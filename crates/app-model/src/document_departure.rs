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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentDepartureOutcomeDto {
    pub committed: bool,
    pub analysis_stopped: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<CurrentGameResultDto>,
    pub message: String,
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
}
