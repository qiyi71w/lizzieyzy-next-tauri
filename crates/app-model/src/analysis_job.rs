use crate::{AnalysisFrameDto, EngineFailureDto, NodePath};
use serde::{Deserialize, Serialize};

impl EngineFailureDto {
    pub fn with_job_id(mut self, job_id: impl Into<String>) -> Self {
        self.job_id = Some(job_id.into());
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisJobLaneDto {
    SelectedNode,
    WholeGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisJobOutcomeDto {
    Started,
    Completed,
    Cancelled,
    Superseded,
    Timeout,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisJobStartedDto {
    pub run_id: String,
    pub job_id: String,
    pub lane: AnalysisJobLaneDto,
    pub generation: u64,
    pub node_path: NodePath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnalysisPublicationScopeDto {
    pub run_id: String,
    pub job_id: String,
    pub generation: u64,
    pub node_path: NodePath,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisJobEventDto {
    pub run_id: String,
    pub job_id: String,
    pub lane: AnalysisJobLaneDto,
    pub generation: u64,
    pub node_path: NodePath,
    pub outcome: AnalysisJobOutcomeDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<AnalysisFrameDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<EngineFailureDto>,
}

impl AnalysisJobStartedDto {
    pub fn publication_scope(&self) -> AnalysisPublicationScopeDto {
        AnalysisPublicationScopeDto {
            run_id: self.run_id.clone(),
            job_id: self.job_id.clone(),
            generation: self.generation,
            node_path: self.node_path.clone(),
        }
    }
}

impl AnalysisJobEventDto {
    pub fn publication_scope(&self) -> AnalysisPublicationScopeDto {
        AnalysisPublicationScopeDto {
            run_id: self.run_id.clone(),
            job_id: self.job_id.clone(),
            generation: self.generation,
            node_path: self.node_path.clone(),
        }
    }
}

pub fn admits_analysis_publication(
    event: &AnalysisJobEventDto,
    current: &AnalysisPublicationScopeDto,
) -> bool {
    event.outcome == AnalysisJobOutcomeDto::Completed
        && event.frame.is_some()
        && event.run_id == current.run_id
        && event.job_id == current.job_id
        && event.generation == current.generation
        && event.node_path == current.node_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AnalysisFrameDto, EngineFailureKind, EngineOperationDto, ForegroundEngineEventDto, MoveVertex,
        PointDto,
    };
    use uuid::Uuid;

    fn sample_event(
        run_id: &str,
        job_id: &str,
        generation: u64,
        indices: Vec<u32>,
        outcome: AnalysisJobOutcomeDto,
        with_frame: bool,
    ) -> AnalysisJobEventDto {
        AnalysisJobEventDto {
            run_id: run_id.into(),
            job_id: job_id.into(),
            lane: AnalysisJobLaneDto::SelectedNode,
            generation,
            node_path: NodePath { indices },
            outcome,
            frame: with_frame.then(|| AnalysisFrameDto {
                job_id: Uuid::nil(),
                game_id: None,
                node_id: None,
                turn: 0,
                visits: 2,
                winrate_black: 0.5,
                score_mean_black: 0.0,
                score_stdev: None,
                candidates: vec![],
                ownership: None,
                policy: None,
            }),
            failure: None,
        }
    }

    #[test]
    fn analysis_job_wire_keeps_snake_case_identities() {
        let started = AnalysisJobStartedDto {
            run_id: "run-1".into(),
            job_id: "job-1".into(),
            lane: AnalysisJobLaneDto::SelectedNode,
            generation: 7,
            node_path: NodePath { indices: vec![0, 1] },
        };
        let event = sample_event(
            "run-1",
            "job-1",
            7,
            vec![0, 1],
            AnalysisJobOutcomeDto::Completed,
            true,
        );
        let wrapped = ForegroundEngineEventDto::Job { job: event.clone() };

        let started_json = serde_json::to_value(&started).unwrap();
        let event_json = serde_json::to_value(&wrapped).unwrap();

        assert_eq!(started_json["run_id"], "run-1");
        assert_eq!(started_json["job_id"], "job-1");
        assert_eq!(started_json["lane"], "selected_node");
        assert_eq!(started_json["generation"], 7);
        assert_eq!(started_json["node_path"]["indices"], serde_json::json!([0, 1]));
        assert_eq!(event_json["type"], "job");
        assert_eq!(event_json["job"]["run_id"], "run-1");
        assert_eq!(event_json["job"]["job_id"], "job-1");
        assert_eq!(event_json["job"]["outcome"], "completed");
        assert_eq!(
            event_json["job"]["node_path"]["indices"],
            serde_json::json!([0, 1])
        );
        assert!(event_json["job"]["frame"].is_object());

        let decoded_started: AnalysisJobStartedDto = serde_json::from_value(started_json).unwrap();
        let decoded_event: ForegroundEngineEventDto = serde_json::from_value(event_json).unwrap();
        assert_eq!(decoded_started, started);
        match decoded_event {
            ForegroundEngineEventDto::Job { job } => {
                assert_eq!(job.run_id, "run-1");
                assert_eq!(job.outcome, AnalysisJobOutcomeDto::Completed);
            }
            other => panic!("expected job event, got {other:?}"),
        }
        let _ = MoveVertex::Point(PointDto { x: 0, y: 0 });
        let _ = EngineOperationDto::Job;
        let _ = EngineFailureKind::InvalidState;
    }

    #[test]
    fn publication_fence_requires_complete_identity_match() {
        let current = AnalysisPublicationScopeDto {
            run_id: "run-1".into(),
            job_id: "job-1".into(),
            generation: 3,
            node_path: NodePath { indices: vec![0] },
        };
        let completed = sample_event(
            "run-1",
            "job-1",
            3,
            vec![0],
            AnalysisJobOutcomeDto::Completed,
            true,
        );
        assert!(admits_analysis_publication(&completed, &current));
        assert!(!admits_analysis_publication(
            &sample_event(
                "run-2",
                "job-1",
                3,
                vec![0],
                AnalysisJobOutcomeDto::Completed,
                true
            ),
            &current
        ));
        assert!(!admits_analysis_publication(
            &sample_event(
                "run-1",
                "job-2",
                3,
                vec![0],
                AnalysisJobOutcomeDto::Completed,
                true
            ),
            &current
        ));
        assert!(!admits_analysis_publication(
            &sample_event(
                "run-1",
                "job-1",
                4,
                vec![0],
                AnalysisJobOutcomeDto::Completed,
                true
            ),
            &current
        ));
        assert!(!admits_analysis_publication(
            &sample_event(
                "run-1",
                "job-1",
                3,
                vec![1],
                AnalysisJobOutcomeDto::Completed,
                true
            ),
            &current
        ));
        assert!(!admits_analysis_publication(
            &sample_event(
                "run-1",
                "job-1",
                3,
                vec![0],
                AnalysisJobOutcomeDto::Cancelled,
                true
            ),
            &current
        ));
        assert!(!admits_analysis_publication(
            &sample_event(
                "run-1",
                "job-1",
                3,
                vec![0],
                AnalysisJobOutcomeDto::Completed,
                false
            ),
            &current
        ));
    }
}
