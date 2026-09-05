use serde::{Deserialize, Serialize};

use crate::CurrentGameResultDto;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DocumentDepartureAdmissionDto {
    NeedsDecision { departure_id: u64 },
    Ready { departure_id: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentDepartureActionDto {
    Save,
    Discard,
    Cancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationExitActionDto {
    Save,
    Discard,
    Cancel,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicationExitDispositionDto {
    CleanCompleted,
    ExplicitDiscard,
    ExitIncomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ApplicationTeardownAttemptDto {
    Completed,
    TimedOut { outstanding: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentDepartureOutcomeDto {
    pub committed: bool,
    pub analysis_stopped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<CurrentGameResultDto>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplicationExitOutcomeDto {
    pub committed: bool,
    pub analysis_stopped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<CurrentGameResultDto>,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disposition: Option<ApplicationExitDispositionDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub teardown: Option<ApplicationTeardownAttemptDto>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_departure_wire_uses_snake_case() {
        let admission = DocumentDepartureAdmissionDto::NeedsDecision { departure_id: 3 };
        let json = serde_json::to_value(&admission).unwrap();
        assert_eq!(json["status"], "needs_decision");
        assert_eq!(json["departure_id"], 3);

        let action = serde_json::to_value(DocumentDepartureActionDto::Save).unwrap();
        assert_eq!(action, "save");
    }

    #[test]
    fn application_exit_wire_uses_snake_case() {
        let action = serde_json::to_value(ApplicationExitActionDto::Continue).unwrap();
        assert_eq!(action, "continue");

        let disposition = serde_json::to_value(ApplicationExitDispositionDto::CleanCompleted).unwrap();
        assert_eq!(disposition, "clean_completed");
        let discarded = serde_json::to_value(ApplicationExitDispositionDto::ExplicitDiscard).unwrap();
        assert_eq!(discarded, "explicit_discard");
        let incomplete = serde_json::to_value(ApplicationExitDispositionDto::ExitIncomplete).unwrap();
        assert_eq!(incomplete, "exit_incomplete");

        let timed_out = serde_json::to_value(ApplicationTeardownAttemptDto::TimedOut {
            outstanding: vec!["foreground engine".to_string()],
        })
        .unwrap();
        assert_eq!(timed_out["status"], "timed_out");
        assert_eq!(timed_out["outstanding"][0], "foreground engine");
    }
}
