use super::*;
use app_model::{LocalFocusActionDto, LocalFocusOutputDto, LocalFocusPointDto, LocalFocusPointStatusDto as Status, LocalFocusRequestDto, LocalFocusSnapshotDto, MoveVertex};

struct Point { value: LocalFocusPointDto, progressed: Instant }
#[derive(Default)]
pub(super) struct Owner {
    identity: Option<AnalysisJobStartedDto>,
    points: Vec<Point>,
    pub(super) revision: u64,
    pub(super) applied: u64,
    pub(super) output: Option<LocalFocusOutputDto>,
    pub(super) lineage: u64,
    pub(super) height: u8,
    pub(super) color: &'static str,
    pending: bool,
}
impl Owner {
    pub(super) fn owns_job(&self, run_id: &str, job_id: &str) -> bool {
        self.identity.as_ref().is_some_and(|id| id.run_id == run_id && id.job_id == job_id)
    }
    pub(super) fn matches(&self, started: &AnalysisJobStartedDto) -> bool {
        self.identity.as_ref().is_some_and(|id| id.run_id == started.run_id && id.job_id == started.job_id && id.generation == started.generation && id.node_path == started.node_path)
    }
    pub(super) fn bind(&mut self, started: &AnalysisJobStartedDto) {
        if self.matches(started) { return; }
        let same_position = self.identity.as_ref().is_some_and(|id| id.run_id == started.run_id && id.generation == started.generation && id.node_path == started.node_path);
        let points = if same_position { std::mem::take(&mut self.points) } else { Vec::new() };
        *self = Self { identity: Some(started.clone()), points, ..Self::default() };
    }
    pub(super) fn snapshot(&self) -> LocalFocusSnapshotDto {
        LocalFocusSnapshotDto {
            run_id: self.identity.as_ref().map(|id| id.run_id.clone()),
            job_id: self.identity.as_ref().map(|id| id.job_id.clone()),
            generation: self.identity.as_ref().map(|id| id.generation),
            node_path: self.identity.as_ref().map(|id| id.node_path.clone()),
            points: self.points.iter().map(|point| point.value.clone()).collect(),
            withdrawal_pending: self.pending,
        }
    }
    fn change(&mut self) {
        self.revision += 1;
        self.lineage = 1;
        self.pending = true;
        // Parser and gateway reject the old output as soon as user intent changes.
        self.output = None;
    }
    pub(super) fn active(&self) -> impl Iterator<Item = &MoveVertex> {
        self.points.iter().filter(|point| point.value.status == Status::Active).map(|point| &point.value.vertex)
    }
    pub(super) fn command(&self, color: &str) -> String {
        let mut command = format!("kata-analyze {color} interval 10 rootInfo true ownership true");
        let count = self.active().count();
        if count > 0 {
            let probability = 0.5 / count as f64;
            for vertex in self.active() {
                let vertex = match vertex {
                    MoveVertex::Pass => "pass".into(),
                    MoveVertex::Point(point) => format!("{}{}", (b'A' + point.x + u8::from(point.x >= 8)) as char, self.height - point.y),
                };
                command.push_str(&format!(" focus {vertex} {probability}"));
            }
        }
        command
    }
    pub(super) fn observe(&mut self, frame: Option<&app_model::AnalysisFrameDto>, now: Instant) -> bool {
        let mut changed = false;
        for point in &mut self.points {
            if point.value.status != Status::Active { continue; }
            if let Some(candidate) = frame.and_then(|frame| frame.candidates.iter().find(|candidate| candidate.vertex == point.value.vertex)) {
                if candidate.visits > point.value.visits {
                    point.value.visits = candidate.visits;
                    point.progressed = now;
                }
                if candidate.visits >= point.value.target_visits {
                    point.value.status = Status::Completed;
                    changed = true;
                    continue;
                }
            }
            if now.saturating_duration_since(point.progressed) > Duration::from_secs(8) {
                point.value.status = Status::Expired;
                changed = true;
            }
        }
        if changed {
            self.revision += 1;
            self.lineage = 1;
            self.pending = true;
        }
        changed
    }
    pub(super) fn applied(&mut self, command: u32, revision: u64) {
        self.applied = revision;
        if self.revision == revision {
            self.output = Some(LocalFocusOutputDto { command, lineage: self.lineage });
            self.pending = false;
        }
    }
    pub(super) fn drained(&mut self) { self.pending = false; }
}

impl ForegroundEngineManager {
    pub fn local_focus_action(&self, request: LocalFocusRequestDto) -> Result<LocalFocusSnapshotDto, String> {
        if matches!(request.action, LocalFocusActionDto::Add { .. }) && self.measured_reservation() {
            return Err("Focus is unavailable during measured setup work".into());
        }
        let mut state = self.lock();
        let id = state.local_focus.identity.as_ref().ok_or("No ordinary local analysis owner")?;
        if id.run_id != request.run_id || id.job_id != request.job_id || id.generation != request.generation || id.node_path != request.node_path {
            return Err("Focus position or output owner changed".into());
        }
        match request.action {
            LocalFocusActionDto::Add { vertex, max_visits } => {
                if max_visits == 0 { return Err("Focus visits must be positive".into()); }
                continuation::require_idle(&state).map_err(|error| error.message)?;
                batch::admit(&state, None).map_err(|error| error.message)?;
                if !state.match_residents.is_empty() || matches!(state.auto_load.snapshot.phase, app_model::AutoLoadPhaseDto::Waiting | app_model::AutoLoadPhaseDto::Running) {
                    return Err("Another analysis owner is active".into());
                }
                let run = admitting_run(&state.phase, &request.run_id).ok_or("Local engine is unavailable")?;
                if run.adapter_kind != EngineBackend::KataGoGtp || !run.capability_snapshot.as_ref().is_some_and(|cap| cap.focus == app_model::FocusCapabilityDto::Supported) {
                    return Err("A qualified local focus engine is required".into());
                }
                let target = state.continuous_target.as_ref().ok_or("Ordinary analysis is unavailable")?;
                if state.continuous_departing || state.finite_admission_pending || state.match_reservation.is_some() || state.runtime_control.is_some()
                    || !state.jobs.iter().any(|job| job.job_id == request.job_id && !job.terminal && job.mode == AnalysisJobModeDto::Continuous && job.disposition == JobDisposition::Running && job.submitted) {
                    return Err("Focus requires the current running ordinary analysis owner".into());
                }
                let position = target.exact_position.as_ref().map_err(Clone::clone)?;
                let legal = position.validate_move(&vertex).is_ok();
                if state.local_focus.points.iter().any(|point| point.value.vertex == vertex) { return Ok(state.local_focus.snapshot()); }
                state.local_focus.points.push(Point { value: LocalFocusPointDto { vertex, target_visits: max_visits, visits: 0, status: if legal { Status::Active } else { Status::Blocked } }, progressed: Instant::now() });
                if legal { state.local_focus.change(); }
            }
            LocalFocusActionDto::Remove { vertex } => {
                let active = state.local_focus.points.iter().any(|point| point.value.vertex == vertex && point.value.status == Status::Active);
                state.local_focus.points.retain(|point| point.value.vertex != vertex);
                if active { state.local_focus.change(); }
            }
            LocalFocusActionDto::Clear => {
                let active = state.local_focus.active().next().is_some();
                state.local_focus.points.clear();
                if active { state.local_focus.change(); }
            }
        }
        publish_snapshot(&mut state);
        Ok(state.local_focus.snapshot())
    }

    /// Rechecked at the current-game publication boundary, not just when parsing bytes.
    pub fn local_focus_publication(&self, event: &AnalysisJobEventDto) -> Result<bool, ()> {
        let state = self.lock();
        let Some(output) = event.local_focus_output else {
            return if state.local_focus.identity.as_ref().is_some_and(|id| id.job_id == event.job_id) { Err(()) } else { Ok(false) };
        };
        if !state.jobs.iter().any(|job| job.job_id == event.job_id && job.run_id == event.run_id && job.disposition == JobDisposition::Running
            && (!job.terminal || matches!((job.state, event.outcome),
                (AnalysisJobStateDto::TimeLimited, AnalysisJobOutcomeDto::TimeLimited)
                | (AnalysisJobStateDto::VisitsLimited, AnalysisJobOutcomeDto::VisitsLimited)))) { return Err(()); }
        let owner = &state.local_focus;
        if owner.output != Some(output) || !owner.identity.as_ref().is_some_and(|id| id.run_id == event.run_id && id.job_id == event.job_id && id.generation == event.generation && id.node_path == event.node_path) {
            return Err(());
        }
        Ok(output.lineage > 0)
    }
}

pub(super) fn reconcile(state: &mut ManagerState) {
    let Some(id) = state.local_focus.identity.as_ref() else { return; };
    // The saved target is Run-neutral; the bound owner is checked against the live Run below.
    let same_position = state.continuous_target.as_ref().is_some_and(|target| target.generation == id.generation && target.node_path == id.node_path);
    if !same_position || admitting_run(&state.phase, &id.run_id).is_none() || state.continuous_departing
        || state.jobs.iter().any(|job| !job.terminal && job.mode == AnalysisJobModeDto::Finite) {
        state.local_focus = Owner::default();
        return;
    }
    if !state.jobs.iter().any(|job| job.job_id == id.job_id && !job.terminal && job.disposition == JobDisposition::Running) {
        for point in &mut state.local_focus.points {
            if point.value.status == Status::Active { point.value.status = Status::Stopped; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watched_points_use_independent_strict_highwater_clocks_and_keep_completion() {
        let now = Instant::now();
        let mut owner = Owner { height: 9, ..Owner::default() };
        for (x, status) in [(3, Status::Active), (4, Status::Active), (5, Status::Blocked)] {
            owner.points.push(Point { value: LocalFocusPointDto { vertex: MoveVertex::Point(app_model::PointDto { x, y: 5 }), target_visits: 500, visits: 0, status }, progressed: now });
        }
        assert!(owner.command("B").contains("focus D4 0.25 focus E4 0.25"));
        let frame = katago_protocol::parse_gtp_analysis("info move D4 visits 100 winrate 0.5 order 0 pv D4 rootInfo visits 10000 winrate 0.5", Uuid::nil(), 9, 9, 0).unwrap();
        assert!(!owner.observe(Some(&frame), now + Duration::from_secs(4)));
        assert!(!owner.observe(Some(&frame), now + Duration::from_secs(8)));
        assert!(owner.observe(Some(&frame), now + Duration::from_secs(8) + Duration::from_nanos(1)));
        let snapshot = owner.snapshot();
        assert_eq!(snapshot.points[0].status, Status::Active);
        assert_eq!(snapshot.points[1].status, Status::Expired);
        assert_eq!(snapshot.points[2].status, Status::Blocked);
        assert_eq!(snapshot.points[0].visits, 100);
        assert!(owner.command("B").contains("focus D4 0.5"));
        let completed = katago_protocol::parse_gtp_analysis("info move D4 visits 500 winrate 0.5 order 0 pv D4 rootInfo visits 1000 winrate 0.5", Uuid::nil(), 9, 9, 0).unwrap();
        assert!(owner.observe(Some(&completed), now + Duration::from_secs(9)));
        assert_eq!(owner.snapshot().points[0].status, Status::Completed);
        assert_eq!(owner.snapshot().points.len(), 3);
        assert!(!owner.command("B").contains("focus"));
        owner.observe(None, now + Duration::from_secs(100));
        assert_eq!(owner.snapshot().points[0].status, Status::Completed);
    }

    #[test]
    fn one_complete_result_finishes_multiple_points_without_unwatching_or_completing_blocked() {
        let now = Instant::now();
        let mut owner = Owner { height: 9, ..Owner::default() };
        for (x, status) in [(3, Status::Active), (4, Status::Active), (5, Status::Blocked)] {
            owner.points.push(Point { value: LocalFocusPointDto { vertex: MoveVertex::Point(app_model::PointDto { x, y: 5 }), target_visits: 500, visits: 0, status }, progressed: now });
        }
        let frame = katago_protocol::parse_gtp_analysis("info move D4 visits 500 winrate 0.5 order 0 pv D4 info move E4 visits 600 winrate 0.5 order 1 pv E4 info move F4 visits 900 winrate 0.5 order 2 pv F4 rootInfo visits 3000 winrate 0.5", Uuid::nil(), 9, 9, 0).unwrap();
        assert!(owner.observe(Some(&frame), now));
        assert_eq!(owner.active().count(), 0);
        assert_eq!(owner.snapshot().points.iter().map(|point| point.status.clone()).collect::<Vec<_>>(), vec![Status::Completed, Status::Completed, Status::Blocked]);
        assert_eq!(owner.snapshot().points[2].visits, 0);
        assert!(!owner.command("B").contains("focus"));
    }

    #[test]
    fn unchanged_or_lower_point_visits_and_root_growth_never_extend_deadline() {
        let now = Instant::now();
        let mut owner = Owner::default();
        owner.points.push(Point { value: LocalFocusPointDto { vertex: MoveVertex::Point(app_model::PointDto { x: 3, y: 5 }), target_visits: 500, visits: 100, status: Status::Active }, progressed: now });
        let frame = katago_protocol::parse_gtp_analysis("info move D4 visits 99 winrate 0.5 order 0 pv D4 rootInfo visits 90000 winrate 0.5", Uuid::nil(), 9, 9, 0).unwrap();
        assert!(!owner.observe(Some(&frame), now + Duration::from_secs(8)));
        assert!(owner.observe(None, now + Duration::from_secs(8) + Duration::from_nanos(1)));
        assert_eq!(owner.snapshot().points[0].visits, 100);
        assert_eq!(owner.snapshot().points[0].status, Status::Expired);
    }
}
