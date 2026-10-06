use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YikeSyncPreferencesDto {
    #[serde(default = "default_interval", deserialize_with = "read_interval")]
    pub interval_seconds: u32,
    #[serde(default)]
    pub locator: Option<String>,
    #[serde(default)]
    pub jump_to_last: bool,
    #[serde(default)]
    pub mute: bool,
}
fn default_interval() -> u32 {
    1
}
fn read_interval<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u32, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StoredInterval {
        Seconds(u32),
        Invalid(serde::de::IgnoredAny),
    }
    Ok(match StoredInterval::deserialize(deserializer)? {
        StoredInterval::Seconds(value) if value > 0 => value,
        _ => 10,
    })
}
impl Default for YikeSyncPreferencesDto {
    fn default() -> Self {
        Self {
            interval_seconds: 1,
            locator: None,
            jump_to_last: false,
            mute: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadboardSyncPreferencesDto {
    #[serde(default = "default_readboard_always_sync")]
    pub always_sync: bool,
    #[serde(default = "default_readboard_focus")]
    pub focus: bool,
    #[serde(default = "default_readboard_mute")]
    pub mute: bool,
    #[serde(default = "default_readboard_jump_to_last")]
    pub jump_to_last: bool,
}

fn default_readboard_always_sync() -> bool {
    true
}

fn default_readboard_focus() -> bool {
    true
}

fn default_readboard_mute() -> bool {
    true
}

fn default_readboard_jump_to_last() -> bool {
    false
}

impl Default for ReadboardSyncPreferencesDto {
    fn default() -> Self {
        Self {
            always_sync: default_readboard_always_sync(),
            focus: default_readboard_focus(),
            mute: default_readboard_mute(),
            jump_to_last: default_readboard_jump_to_last(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSyncPhaseDto {
    #[default]
    Idle,
    Starting,
    Syncing,
    Retrying,
    ErrorPaused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSyncSourceDto {
    Yike,
    Readboard,
}

/// Readboard-only session facts; `source_move_number` is snapshot metadata, not a move count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadboardSyncStatusDto {
    pub preferences: ReadboardSyncPreferencesDto,
    pub runtime_generation: Option<u64>,
    pub source_move_number: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExternalSyncSnapshotDto {
    pub revision: u64,
    pub session_id: Option<u64>,
    pub starting_id: Option<u64>,
    pub phase: ExternalSyncPhaseDto,
    pub source: Option<ExternalSyncSourceDto>,
    pub locator: Option<String>,
    pub source_status: Option<String>,
    pub source_tip: Option<crate::NodePath>,
    pub document_identity: Option<u64>,
    pub request_identity: Option<crate::ProviderRequestIdentityDto>,
    pub retry_count: u8,
    pub failure: Option<crate::ProviderError>,
    pub browser_error: Option<String>,
    /// Yike owner preferences; readboard preferences live in `readboard`.
    pub preferences: YikeSyncPreferencesDto,
    pub readboard: Option<ReadboardSyncStatusDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalSyncUpdateDto {
    pub sync: ExternalSyncSnapshotDto,
    pub current: Option<crate::CurrentGameResultDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalSyncStartDto {
    pub start_id: u64,
    pub admission: crate::DocumentDepartureAdmissionDto,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_sync_interval_recovers_invalid_historical_values() {
        for invalid in ["0", "-1", "1.5", "4294967296", "null", "\"invalid\"", "{}", "[]"] {
            let stored = format!(r#"{{"intervalSeconds":{invalid},"mute":true}}"#);
            let preferences: YikeSyncPreferencesDto = serde_json::from_str(&stored).unwrap();
            assert_eq!(preferences.interval_seconds, 10, "{invalid}");
            assert!(preferences.mute);
        }
        let missing: YikeSyncPreferencesDto = serde_json::from_str("{}").unwrap();
        assert_eq!(missing.interval_seconds, 1);
        let maximum: YikeSyncPreferencesDto =
            serde_json::from_str(r#"{"intervalSeconds":4294967295}"#).unwrap();
        assert_eq!(maximum.interval_seconds, u32::MAX);
    }

    #[test]
    fn readboard_sync_preferences_defaults_and_partial_deserialization() {
        let empty: ReadboardSyncPreferencesDto = serde_json::from_str("{}").unwrap();
        assert_eq!(empty, ReadboardSyncPreferencesDto::default());
        assert!(empty.always_sync);
        assert!(empty.focus);
        assert!(empty.mute);
        assert!(!empty.jump_to_last);

        let mute_false: ReadboardSyncPreferencesDto = serde_json::from_str(r#"{"mute":false}"#).unwrap();
        assert_eq!(
            mute_false,
            ReadboardSyncPreferencesDto {
                always_sync: true,
                focus: true,
                mute: false,
                jump_to_last: false,
            }
        );

        let focus_false: ReadboardSyncPreferencesDto = serde_json::from_str(r#"{"focus":false}"#).unwrap();
        assert_eq!(
            focus_false,
            ReadboardSyncPreferencesDto {
                always_sync: true,
                focus: false,
                mute: true,
                jump_to_last: false,
            }
        );

        let sync_false: ReadboardSyncPreferencesDto =
            serde_json::from_str(r#"{"alwaysSync":false}"#).unwrap();
        assert_eq!(
            sync_false,
            ReadboardSyncPreferencesDto {
                always_sync: false,
                focus: true,
                mute: true,
                jump_to_last: false,
            }
        );

        let jump_true: ReadboardSyncPreferencesDto = serde_json::from_str(r#"{"jumpToLast":true}"#).unwrap();
        assert_eq!(
            jump_true,
            ReadboardSyncPreferencesDto {
                always_sync: true,
                focus: true,
                mute: true,
                jump_to_last: true,
            }
        );

        let all_inverted = ReadboardSyncPreferencesDto {
            always_sync: false,
            focus: false,
            mute: false,
            jump_to_last: true,
        };
        let serialized = serde_json::to_string(&all_inverted).unwrap();
        let deserialized: ReadboardSyncPreferencesDto = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, all_inverted);
    }
}
