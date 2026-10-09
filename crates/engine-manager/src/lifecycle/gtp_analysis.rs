//! Named KataGo GTP analysis on the existing Run, reader, command IDs and exact restore.
use super::*;
use app_model::{AnalysisFrameDto, ComputeBudgetDto, EngineGtpFactsDto, GameMoveRequestDto};
use std::sync::mpsc::RecvTimeoutError;

pub(super) fn capabilities(
    run: &EngineRunDto,
    facts: &EngineGtpFactsDto,
) -> Option<EngineAnalysisCapabilitiesDto> {
    // Discovery alone is not implementation support: this codec is qualified against 1.18.2.
    let version = facts.version.split('+').next()?;
    if run.adapter_kind != EngineBackend::KataGoGtp
        || facts.name != "KataGo"
        || version != "1.18.2"
        || !["kata-analyze", "stop"]
            .iter()
            .all(|command| facts.commands.iter().any(|value| value == command))
    {
        return None;
    }
    Some(EngineAnalysisCapabilitiesDto {
        selected_node_analysis: true,
        continuous_analysis: true,
        whole_game_analysis: false,
        candidates: true,
        pv: true,
        winrate: true,
        root_score: true,
        ownership: true,
        policy: false,
        visits_limit: true,
        protocol_cancel: true,
    })
}

struct Command {
    id: u32,
    response: Receiver<String>,
    written: Receiver<Result<(), EngineFailureDto>>,
}

struct AnalysisError {
    kind: EngineFailureKind,
    message: AnalysisFailureText,
}
enum AnalysisFailureText {
    Public(FailureText),
    Dynamic(String),
    Context(&'static str, String),
}
impl From<String> for AnalysisError {
    fn from(message: String) -> Self {
        Self {
            kind: EngineFailureKind::Protocol,
            message: AnalysisFailureText::Dynamic(message),
        }
    }
}
impl From<&'static str> for AnalysisError {
    fn from(message: &'static str) -> Self {
        Self { kind: EngineFailureKind::Protocol, message: AnalysisFailureText::Public(message.into()) }
    }
}
fn response_error(error: RecvTimeoutError, context: &'static str) -> AnalysisError {
    AnalysisError {
        kind: if error == RecvTimeoutError::Timeout {
            EngineFailureKind::Timeout
        } else {
            EngineFailureKind::Protocol
        },
        message: AnalysisFailureText::Context(context, error.to_string()),
    }
}

impl ForegroundEngineManager {
    // Run IDs are fresh per process and have precisely one immutable stdout pump. A retired
    // Run cannot register again: both registration and writing require that live incarnation.
    fn begin_analysis_command(
        &self,
        started: &AnalysisJobStartedDto,
        body: &str,
        stream: bool,
    ) -> Result<Command, String> {
        let (id, response) = {
            let mut state = self.lock();
            if owned_live(&state, &started.run_id).is_none() {
                return Err("analysis Run retired".into());
            }
            state.gtp_command_seq = state
                .gtp_command_seq
                .checked_add(1)
                .ok_or("GTP command IDs exhausted")?;
            let id = state.gtp_command_seq;
            let response = state
                .gtp_dispatch
                .register(&started.run_id, id, &started.job_id, stream)?;
            (id, response)
        };
        let (sender, written) = mpsc::sync_channel(1);
        let manager = self.clone();
        let run_id = started.run_id.clone();
        let payload = format!("{id} {body}");
        thread::spawn(move || {
            let result = manager.write_live_jsonl(&run_id, &payload);
            let _ = sender.send(result);
        });
        Ok(Command {
            id,
            response,
            written,
        })
    }

    pub(super) fn spawn_gtp_analysis(&self, started: AnalysisJobStartedDto, request: SelectedNodeJobRequest) {
        let manager = self.clone();
        thread::spawn(move || {
            let result = manager.run_gtp_analysis(&started, &request);
            if let Err(error) = result {
                let message = match error.message {
                    AnalysisFailureText::Public(text) => text,
                    AnalysisFailureText::Dynamic(text) => manager.inner.diagnostic_text(&started.run_id, &text),
                    AnalysisFailureText::Context(context, text) => FailureText::from(context).then(": ")
                        .then(manager.inner.diagnostic_text(&started.run_id, &text)),
                };
                manager.inner.fail_analysis_run(&started.run_id, error.kind, message);
            }
            manager
                .lock()
                .gtp_dispatch
                .retire(&started.run_id, &started.job_id);
            manager.reconcile_continuous();
        });
    }

    fn run_gtp_analysis(
        &self,
        started: &AnalysisJobStartedDto,
        request: &SelectedNodeJobRequest,
    ) -> Result<(), AnalysisError> {
        let position = request.exact_position.clone()?;
        let rules_request = GameMoveRequest {
            identity: GameMoveRequestDto {
                run_id: started.run_id.clone(),
                generation: started.generation,
                node_path: started.node_path.clone(),
                budget: ComputeBudgetDto {
                    deadline_ms: 30_000,
                    max_visits: None,
                },
            },
            position: position.clone(),
        };
        // The selected job owns admission throughout this existing ticket09 transaction and
        // the following stream. Other positioning transactions cannot interleave.
        let confirmed = self.confirm_analysis_rules(rules_request, &started.job_id);
        if let Err(error) = confirmed {
            let mut state = self.lock();
            if let Some(job) = state.jobs.iter().find(|job| job.job_id == started.job_id) {
                if job.disposition != JobDisposition::Running {
                    let outcome = stopped_outcome(job.disposition);
                    finish_selected_node_job(&mut state, started, Some(outcome), None, None);
                    publish_snapshot(&mut state);
                    return Ok(());
                }
            }
            state.continuous_safety_hold = true;
            finish_failed_job(&mut state, started, error);
            publish_snapshot(&mut state);
            return Ok(());
        }
        {
            let mut state = self.lock();
            let Some(job) = state
                .jobs
                .iter()
                .find(|job| job.job_id == started.job_id && !job.terminal)
            else {
                return Ok(());
            };
            if job.disposition != JobDisposition::Running {
                let outcome = stopped_outcome(job.disposition);
                finish_selected_node_job(&mut state, started, Some(outcome), None, None);
                publish_snapshot(&mut state);
                return Ok(());
            }
        }
        let color = match position.dto().to_play {
            app_model::PlayerColor::Black => "B",
            app_model::PlayerColor::White => "W",
        };
        let stream = self.begin_analysis_command(
            started,
            &format!("kata-analyze {color} interval 10 rootInfo true ownership true"),
            true,
        )?;
        wait_written(&stream, Instant::now() + Duration::from_secs(5))?;
        let began = Instant::now();
        {
            let mut state = self.lock();
            if let Some(job) = state.jobs.iter_mut().find(|job| job.job_id == started.job_id) {
                job.submitted = true;
                job.submitted_at = began;
            }
        }
        let mut header = false;
        let mut last: Option<AnalysisFrameDto> = None;
        let outcome = loop {
            let state = self.lock();
            let Some(job) = state
                .jobs
                .iter()
                .find(|job| job.job_id == started.job_id && !job.terminal)
            else {
                return Err("analysis job retired before its stream drain".into());
            };
            if job.disposition != JobDisposition::Running {
                break stopped_outcome(job.disposition);
            }
            let visits = last.as_ref().map_or(0, |frame| frame.visits);
            if let Some(budget) = job.continuous_budget {
                if budget.continuous_visits_limit_enabled && visits >= budget.continuous_visits_limit {
                    break AnalysisJobOutcomeDto::VisitsLimited;
                }
                if budget.continuous_time_limit_enabled
                    && began.elapsed() >= Duration::from_secs(u64::from(budget.continuous_time_limit_seconds))
                {
                    break AnalysisJobOutcomeDto::TimeLimited;
                }
            } else if request.query.max_visits.is_some_and(|limit| visits >= limit) {
                break AnalysisJobOutcomeDto::Completed;
            }
            drop(state);
            match stream.response.recv_timeout(Duration::from_millis(20)) {
                Ok(line) => {
                    if !header {
                        require_stream_header(stream.id, &line)?;
                        header = true;
                        continue;
                    }
                    if line.is_empty() {
                        return Err("KataGo analysis ended without the requested stop barrier".into());
                    }
                    let frame = katago_protocol::parse_gtp_analysis(
                        &line,
                        Uuid::parse_str(&started.job_id).map_err(|error| error.to_string())?,
                        request.board_width,
                        request.board_height,
                        position.dto().moves.len() as u32,
                    )?;
                    let mut state = self.lock();
                    if admitting_run(&state.phase, &started.run_id).is_none() {
                        continue;
                    }
                    let Some(job) = state.jobs.iter_mut().find(|job| {
                        job.job_id == started.job_id
                            && !job.terminal
                            && job.disposition == JobDisposition::Running
                    }) else {
                        continue;
                    };
                    job.state = AnalysisJobStateDto::Searching;
                    state.last_activity = Instant::now();
                    publish_event(
                        &mut state,
                        ForegroundEngineEventDto::Job {
                            job: selected_node_job_event(
                                started,
                                AnalysisJobOutcomeDto::Progress,
                                Some(frame.clone()),
                                None,
                            ),
                        },
                    );
                    publish_snapshot(&mut state);
                    last = Some(frame);
                }
                Err(RecvTimeoutError::Timeout) => {
                    if !header && began.elapsed() >= Duration::from_secs(5) {
                        return Err(response_error(
                            RecvTimeoutError::Timeout,
                            "KataGo analysis header",
                        ));
                    }
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err("KataGo analysis response disconnected".into())
                }
            }
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        let stop = self.begin_analysis_command(started, "stop", false)?;
        wait_written(&stop, deadline)?;
        // The stream's delimiter and this Stop's complete numbered reply are independent
        // barriers. Neither an info/play record nor any other request's ACK can release them.
        loop {
            let line = stream
                .response
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|error| response_error(error, "KataGo stream drain"))?;
            if !header {
                require_stream_header(stream.id, &line)?;
                header = true;
            } else if line.is_empty() {
                break;
            }
        }
        let mut decoder = crate::gtp::ResponseDecoder::new(stop.id);
        loop {
            let line = stop
                .response
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(|error| response_error(error, "KataGo Stop acknowledgement"))?;
            if let Some(reply) = decoder.push(&line)? {
                if !reply.success {
                    return Err("KataGo rejected the numbered Stop request".into());
                }
                if !reply.body.trim().is_empty() {
                    return Err("KataGo Stop response contained an unexpected body".into());
                }
                break;
            }
        }
        let mut state = self.lock();
        let Some(job) = state
            .jobs
            .iter()
            .find(|job| job.job_id == started.job_id && !job.terminal)
        else {
            return Ok(());
        };
        let (outcome, frame) = if job.disposition == JobDisposition::Running {
            (outcome, last)
        } else {
            (stopped_outcome(job.disposition), None)
        };
        finish_selected_node_job(&mut state, started, Some(outcome), frame, None);
        state.last_activity = Instant::now();
        publish_snapshot(&mut state);
        Ok(())
    }
}

fn wait_written(command: &Command, deadline: Instant) -> Result<(), AnalysisError> {
    command
        .written
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|error| response_error(error, "GTP analysis write"))?
        .map_err(|error| AnalysisError {
            kind: error.kind,
            message: AnalysisFailureText::Public(FailureText::from_failure(&error)),
        })
}
fn require_stream_header(id: u32, line: &str) -> Result<(), String> {
    if line.trim_end() == format!("={id}") {
        Ok(())
    } else {
        Err("KataGo analysis command did not acknowledge its numbered request".into())
    }
}
fn stopped_outcome(disposition: JobDisposition) -> AnalysisJobOutcomeDto {
    match disposition {
        JobDisposition::Superseded => AnalysisJobOutcomeDto::Superseded,
        JobDisposition::TimedOut => AnalysisJobOutcomeDto::Timeout,
        _ => AnalysisJobOutcomeDto::Cancelled,
    }
}
