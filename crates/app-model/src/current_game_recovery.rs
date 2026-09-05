use serde::{Deserialize, Serialize};

use crate::{ApplicationExitDispositionDto, NodePath};

pub const RECOVERY_UNREADABLE_MESSAGE: &str = "Recovery snapshot is unreadable and was not applied.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryEnvelopeDto {
    pub document_seq: u64,
    pub snapshot_seq: u64,
    pub sgf_text: String,
    pub selected_path: NodePath,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    pub dirty: bool,
    pub disposition: ApplicationExitDispositionDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RecoveryProtectionDto {
    Protected,
    Unprotected { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum RecoveryStartupDto {
    None,
    Abnormal { envelope: RecoveryEnvelopeDto },
    Normal { envelope: RecoveryEnvelopeDto },
    Unreadable { message: String },
}

impl RecoveryEnvelopeDto {
    pub fn is_newer_than(&self, other: &RecoveryEnvelopeDto) -> bool {
        self.document_seq > other.document_seq
            || (self.document_seq == other.document_seq && self.snapshot_seq > other.snapshot_seq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovery_wire_uses_snake_case() {
        let envelope = RecoveryEnvelopeDto {
            document_seq: 2,
            snapshot_seq: 4,
            sgf_text: "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])".to_string(),
            selected_path: NodePath { indices: vec![0] },
            source_path: Some("/tmp/game.sgf".to_string()),
            dirty: true,
            disposition: ApplicationExitDispositionDto::ExitIncomplete,
        };
        let json = serde_json::to_value(&envelope).unwrap();
        assert_eq!(json["document_seq"], 2);
        assert_eq!(json["snapshot_seq"], 4);
        assert_eq!(json["selected_path"]["indices"][0], 0);
        assert_eq!(json["source_path"], "/tmp/game.sgf");
        assert_eq!(json["disposition"], "exit_incomplete");

        let unprotected = serde_json::to_value(RecoveryProtectionDto::Unprotected {
            message: "write failed".to_string(),
        })
        .unwrap();
        assert_eq!(unprotected["status"], "unprotected");

        let abnormal = serde_json::to_value(RecoveryStartupDto::Abnormal {
            envelope: envelope.clone(),
        })
        .unwrap();
        assert_eq!(abnormal["status"], "abnormal");
        let normal = serde_json::to_value(RecoveryStartupDto::Normal { envelope }).unwrap();
        assert_eq!(normal["status"], "normal");
        let none = serde_json::to_value(RecoveryStartupDto::None).unwrap();
        assert_eq!(none["status"], "none");
    }
}
