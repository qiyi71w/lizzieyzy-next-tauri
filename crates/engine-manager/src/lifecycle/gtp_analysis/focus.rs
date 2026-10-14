use super::*;
use app_model::FocusCapabilityDto;

impl ForegroundEngineManager {
    // Called only after this selected job's exact-position confirmation, before its
    // ordinary stream. The job retains the same exclusive admission during the probe.
    pub(super) fn probe_focus(
        &self,
        started: &AnalysisJobStartedDto,
        request: &SelectedNodeJobRequest,
    ) -> Result<(), AnalysisError> {
        {
            let state = self.lock();
            if !probe_current(&state, started) {
                return Ok(());
            }
            let run = admitting_run(&state.phase, &started.run_id).unwrap();
            if !run
                .qualified_resource
                .as_deref()
                .is_some_and(crate::thread_policy::direct_local)
                || state.match_reservation.is_some()
                || !state.match_residents.is_empty()
                || state.runtime_control.is_some()
                || continuation::occupied(&state)
                || batch::occupied(&state.batch)
                || state.continuous_departing
                || run.capability_snapshot.as_ref().is_some_and(|caps| {
                    matches!(
                        caps.focus,
                        FocusCapabilityDto::Supported | FocusCapabilityDto::Unsupported
                    )
                })
            {
                return Ok(());
            }
        }
        let result = self.run_focus_probe(started, request);
        let mut state = self.lock();
        // Late completion drains the wire, but has no authority over capability,
        // a replacement binding, Pause, or ordinary analysis admission.
        if probe_current(&state, started) {
            if let Phase::Ready(run) = &mut state.phase {
                if let Some(caps) = &mut run.capability_snapshot {
                    caps.focus = match &result {
                        Ok(value) => *value,
                        Err(_) => FocusCapabilityDto::Uncertain,
                    };
                }
            }
            publish_snapshot(&mut state);
        }
        result.map(|_| ())
    }

    fn run_focus_probe(
        &self,
        started: &AnalysisJobStartedDto,
        request: &SelectedNodeJobRequest,
    ) -> Result<FocusCapabilityDto, AnalysisError> {
        let position = request.exact_position.as_ref().map_err(|s| s.clone())?;
        let color = match position.dto().to_play {
            PlayerColor::Black => "B",
            PlayerColor::White => "W",
        };
        // Frozen KataGo 47aadc0 parseAnalyzeCommand: focus <moves[:weight]> <probability>.
        // Pass is legal and probability zero introduces no focus search bias.
        let stream = self.begin_analysis_command(
            started,
            &format!("kata-analyze {color} interval 10 rootInfo true focus pass 0"),
            true,
        )?;
        let deadline = Instant::now() + Duration::from_secs(5);
        wait_written(&stream, deadline)?;
        let first = receive(&stream, deadline, "focus startup")?;
        if first.starts_with('?') {
            let mut decoder = crate::gtp::ResponseDecoder::new(stream.id);
            decoder.push(&first)?;
            loop {
                if let Some(reply) = decoder.push(&receive(&stream, deadline, "focus rejection final")?)? {
                    if !reply.success && reply.body.starts_with("Could not parse analyze arguments") {
                        return Ok(FocusCapabilityDto::Unsupported);
                    }
                    return Err("focus probe rejected without a syntax qualification".into());
                }
            }
        }
        require_stream_header(stream.id, &first)?;
        let mut saw_root = false;
        while !saw_root {
            let line = receive(&stream, deadline, "focus rootInfo startup")?;
            if line.is_empty() {
                return Err("focus probe ended before Stop".into());
            }
            katago_protocol::parse_gtp_analysis(
                &line,
                Uuid::parse_str(&started.job_id).map_err(|e| e.to_string())?,
                request.board_width,
                request.board_height,
                position.dto().moves.len() as u32,
            )?;
            saw_root = true;
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        let stop = self.begin_analysis_command(started, "stop", false)?;
        wait_written(&stop, deadline)?;
        while !receive(&stream, deadline, "focus stream final")?.is_empty() {}
        let mut decoder = crate::gtp::ResponseDecoder::new(stop.id);
        loop {
            if let Some(reply) = decoder.push(&receive(&stop, deadline, "focus Stop final")?)? {
                if !reply.success || !reply.body.trim().is_empty() {
                    return Err("focus Stop did not confirm complete cleanup".into());
                }
                return Ok(FocusCapabilityDto::Supported);
            }
        }
    }
}

fn receive(command: &Command, deadline: Instant, context: &'static str) -> Result<String, AnalysisError> {
    command
        .response
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|error| response_error(error, context))
}

fn probe_current(state: &ManagerState, started: &AnalysisJobStartedDto) -> bool {
    matches!(&state.phase, Phase::Ready(run) if run.run_id == started.run_id)
        && state.jobs.iter().any(|job| {
            job.run_id == started.run_id
                && job.job_id == started.job_id
                && !job.terminal
                && job.disposition == JobDisposition::Running
        })
}
