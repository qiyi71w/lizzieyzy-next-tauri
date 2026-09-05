use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

mod analysis_job;
pub use analysis_job::{
    admits_analysis_attachment, admits_analysis_publication, AnalysisJobEventDto, AnalysisJobLaneDto,
    AnalysisJobOutcomeDto, AnalysisJobStartedDto, AnalysisPublicationScopeDto,
};
mod document_departure;
pub use document_departure::{
    ApplicationExitActionDto, ApplicationExitDispositionDto, ApplicationExitOutcomeDto,
    ApplicationTeardownAttemptDto, DocumentDepartureActionDto, DocumentDepartureAdmissionDto,
    DocumentDepartureOutcomeDto,
};

pub type GameId = Uuid;
pub type NodeId = Uuid;
pub type AnalysisJobId = Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerColor {
    Black,
    White,
}

impl PlayerColor {
    pub fn opponent(self) -> Self {
        match self {
            PlayerColor::Black => PlayerColor::White,
            PlayerColor::White => PlayerColor::Black,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PointDto {
    pub x: u8,
    pub y: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveVertex {
    Point(PointDto),
    Pass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveDto {
    pub color: PlayerColor,
    pub vertex: MoveVertex,
    pub move_number: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoneDto {
    pub x: u8,
    pub y: u8,
    pub color: PlayerColor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionDto {
    pub board_size: u8,
    pub move_number: u32,
    pub to_play: PlayerColor,
    pub stones: Vec<StoneDto>,
    pub captures_black: u32,
    pub captures_white: u32,
    pub last_move: Option<MoveDto>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSummaryDto {
    pub id: GameId,
    pub board_size: u8,
    pub komi: f32,
    pub black_name: Option<String>,
    pub white_name: Option<String>,
    pub result: Option<String>,
    pub move_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameDto {
    pub summary: GameSummaryDto,
    pub moves: Vec<MoveDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct NodePath {
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SgfPropertyDto {
    pub key: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SgfTreeNodeDto {
    pub properties: Vec<SgfPropertyDto>,
    pub children: Vec<SgfTreeNodeDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectedNodeSnapshotDto {
    pub path: NodePath,
    pub position: PositionDto,
    pub personal_comment: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_information: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_analysis: Option<AnalysisFrameDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary_analysis: Option<AnalysisFrameDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CurrentGameResultDto {
    pub tree: SgfTreeNodeDto,
    pub selected_path: NodePath,
    pub snapshot: SelectedNodeSnapshotDto,
    pub generation: u64,
    pub dirty: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentGameError {
    pub kind: CurrentGameErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurrentGameErrorKind {
    NoCurrentGame,
    InvalidNodePath,
    MalformedSgf,
    UnsupportedBoardSize,
    OccupiedPoint,
    Suicide,
    SimpleKo,
    RootRemoval,
    DepartureInProgress,
    DepartureBlocked,
}

impl std::fmt::Display for CurrentGameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for CurrentGameError {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateMoveDto {
    pub vertex: MoveVertex,
    pub visits: u32,
    pub winrate_black: f32,
    pub score_mean_black: f32,
    pub policy_prior: Option<f32>,
    pub pv: Vec<MoveVertex>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisFrameDto {
    pub job_id: AnalysisJobId,
    pub game_id: Option<GameId>,
    pub node_id: Option<NodeId>,
    pub turn: u32,
    pub visits: u32,
    pub winrate_black: f32,
    pub score_mean_black: f32,
    pub score_stdev: Option<f32>,
    pub candidates: Vec<CandidateMoveDto>,
    pub ownership: Option<Vec<f32>>,
    pub policy: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemMarkerDto {
    pub turn: u32,
    pub severity: ProblemSeverity,
    pub winrate_loss: f32,
    pub score_loss: f32,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemSeverity {
    Info,
    Inaccuracy,
    Mistake,
    Blunder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineProfileDto {
    pub name: String,
    pub engine_path: String,
    pub model_path: Option<String>,
    pub config_path: Option<String>,
    pub working_dir: Option<String>,
    pub backend: EngineBackend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineBackend {
    KataGoAnalysis,
    KataGoGtp,
    GenericGtp,
    ReadboardSidecar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineOperationDto {
    Start,
    Stop,
    Restart,
    Switch,
    Autoload,
    Teardown,
    Job,
    DeleteProfile,
    UnexpectedExit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineFailureKind {
    Start,
    Asset,
    Readiness,
    Protocol,
    NonzeroExit,
    Timeout,
    Cancellation,
    UnsupportedCapability,
    InvalidState,
    Occupied,
    ProfileNotFound,
    ProfileInUse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineFailureDto {
    pub operation: EngineOperationDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub switch_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub job_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    pub kind: EngineFailureKind,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic_summary: Option<String>,
}

impl std::fmt::Display for EngineFailureDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for EngineFailureDto {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineCapabilitySnapshotDto {
    pub adapter_kind: EngineBackend,
    pub selected_node_analysis: bool,
    pub whole_game_analysis: bool,
    pub protocol_cancel: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineRunDto {
    pub run_id: String,
    pub profile_id: String,
    pub adapter_kind: EngineBackend,
    pub profile_snapshot: EngineProfileDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_snapshot: Option<EngineCapabilitySnapshotDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForegroundEngineSnapshotDto {
    pub revision: u64,
    pub lifecycle: ForegroundEngineLifecycleDto,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_node_job: Option<AnalysisJobStartedDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whole_game_job: Option<AnalysisJobStartedDto>,
}

impl ForegroundEngineSnapshotDto {
    pub fn with_lifecycle(revision: u64, lifecycle: ForegroundEngineLifecycleDto) -> Self {
        Self {
            revision,
            lifecycle,
            selected_node_job: None,
            whole_game_job: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ForegroundEngineLifecycleDto {
    NoEngine {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        failure: Option<EngineFailureDto>,
    },
    Starting {
        run: EngineRunDto,
    },
    Ready {
        run: EngineRunDto,
    },
    Switching {
        primary: EngineRunDto,
        candidate: EngineRunDto,
        switch_id: String,
    },
    Stopping {
        run: EngineRunDto,
    },
    Error {
        run: EngineRunDto,
        failure: EngineFailureDto,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ForegroundEngineEventDto {
    Snapshot { snapshot: ForegroundEngineSnapshotDto },
    Failure { failure: EngineFailureDto },
    Job { job: AnalysisJobEventDto },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppHealthDto {
    pub app: String,
    pub architecture: String,
    pub rust_backend_ready: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Yike,
    Fox,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderImportRequest {
    pub provider: ProviderKind,
    pub payload: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default)]
    pub metadata: ProviderGameMetadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderImportResult {
    pub provider: ProviderKind,
    pub sgf_text: String,
    pub summary: ProviderGameSummary,
    pub metadata: ProviderGameMetadata,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderGameSummary {
    pub provider: ProviderKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board_size: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub komi: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handicap: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub black_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub white_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub move_count: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ProviderGameMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_status: Option<String>,
    #[serde(default)]
    pub extra: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderError {
    pub kind: ProviderErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderErrorKind {
    InvalidRequest,
    UnsupportedProvider,
    InvalidUrl,
    InvalidPayload,
    ParseFailed,
    TransportFailed,
    Timeout,
    RuntimeUnavailable,
    NotImplemented,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProviderFetchMethod {
    #[default]
    Get,
    Post,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderFetchRequest {
    pub provider: ProviderKind,
    pub url: String,
    #[serde(default)]
    pub method: ProviderFetchMethod,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderFetchResult {
    pub provider: ProviderKind,
    pub url: String,
    pub status_code: u16,
    pub payload: String,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(default)]
    pub metadata: ProviderGameMetadata,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ReadboardSidecarProbeRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReadboardSidecarProbeResult {
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct ReadboardSidecarSyncSnapshotRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snapshot_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_base64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sgf_text: Option<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ReadboardSidecarSyncSnapshotResult {
    pub snapshot_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<PositionDto>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl ProviderGameSummary {
    pub fn empty(provider: ProviderKind) -> Self {
        Self {
            provider,
            source_id: None,
            board_size: None,
            komi: None,
            handicap: None,
            black_name: None,
            white_name: None,
            result: None,
            date: None,
            move_count: None,
        }
    }
}

#[cfg(test)]
mod provider_tests {
    use super::*;

    #[test]
    fn provider_dtos_serialize_with_snake_case_contract() {
        let request = ProviderImportRequest {
            provider: ProviderKind::Yike,
            payload: "(;GM[1])".to_string(),
            source_url: Some("https://example.test/game".to_string()),
            source_id: Some("123".to_string()),
            metadata: ProviderGameMetadata::default(),
        };

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["provider"], "yike");
        assert_eq!(json["source_url"], "https://example.test/game");
        assert_eq!(json["source_id"], "123");
    }

    #[test]
    fn provider_errors_serialize_kind_for_tauri_commands() {
        let error = ProviderError {
            kind: ProviderErrorKind::InvalidPayload,
            message: "missing SGF".to_string(),
        };

        let json = serde_json::to_value(error).unwrap();

        assert_eq!(json["kind"], "invalid_payload");
        assert_eq!(json["message"], "missing SGF");
    }

    #[test]
    fn provider_fetch_dtos_serialize_with_snake_case_contract() {
        let request = ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: "https://example.test/fetch".to_string(),
            method: ProviderFetchMethod::Post,
            headers: BTreeMap::from([("x-test".to_string(), "1".to_string())]),
            body: Some("{}".to_string()),
            source_url: Some("https://example.test/game".to_string()),
            source_id: Some("game-1".to_string()),
            timeout_ms: Some(500),
        };

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["provider"], "fox");
        assert_eq!(json["method"], "post");
        assert_eq!(json["source_url"], "https://example.test/game");
        assert_eq!(json["timeout_ms"], 500);
    }

    #[test]
    fn readboard_sidecar_dtos_serialize_contract() {
        let request = ReadboardSidecarSyncSnapshotRequest {
            endpoint: Some("http://127.0.0.1:39081".to_string()),
            snapshot_id: Some("snapshot-1".to_string()),
            image_path: Some("/tmp/board.png".to_string()),
            image_base64: None,
            sgf_text: None,
            metadata: BTreeMap::from([("source".to_string(), "test".to_string())]),
            timeout_ms: Some(250),
        };

        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["snapshot_id"], "snapshot-1");
        assert_eq!(json["image_path"], "/tmp/board.png");
        assert_eq!(json["timeout_ms"], 250);
    }
}

#[cfg(test)]
mod current_game_wire {
    use super::*;

    #[test]
    fn current_game_wire_tree_path_snapshot_and_result_use_snake_case() {
        let result = CurrentGameResultDto {
            tree: SgfTreeNodeDto {
                properties: vec![SgfPropertyDto {
                    key: "C".to_string(),
                    values: vec!["root personal".to_string()],
                }],
                children: vec![SgfTreeNodeDto {
                    properties: vec![SgfPropertyDto {
                        key: "W".to_string(),
                        values: vec!["dd".to_string()],
                    }],
                    children: Vec::new(),
                }],
            },
            selected_path: NodePath {
                indices: vec![0, 0, 0],
            },
            snapshot: SelectedNodeSnapshotDto {
                path: NodePath {
                    indices: vec![0, 0, 0],
                },
                position: PositionDto {
                    board_size: 5,
                    move_number: 3,
                    to_play: PlayerColor::Black,
                    stones: vec![StoneDto {
                        x: 0,
                        y: 0,
                        color: PlayerColor::Black,
                    }],
                    captures_black: 0,
                    captures_white: 1,
                    last_move: Some(MoveDto {
                        color: PlayerColor::White,
                        vertex: MoveVertex::Pass,
                        move_number: 3,
                    }),
                    errors: Vec::new(),
                },
                personal_comment: "mainline pass".to_string(),
                generated_information: None,
                primary_analysis: None,
                secondary_analysis: None,
            },
            generation: 1,
            dirty: false,
            native_path: Some("/tmp/game.sgf".to_string()),
        };

        let json = serde_json::to_value(&result).unwrap();
        let decoded: CurrentGameResultDto = serde_json::from_value(json.clone()).unwrap();

        assert_eq!(json["selected_path"]["indices"], serde_json::json!([0, 0, 0]));
        assert_eq!(json["snapshot"]["personal_comment"], "mainline pass");
        assert_eq!(json["snapshot"]["position"]["to_play"], "black");
        assert_eq!(json["snapshot"]["position"]["last_move"]["vertex"], "pass");
        assert_eq!(json["snapshot"]["position"]["captures_white"], 1);
        assert_eq!(json["generation"], 1);
        assert_eq!(json["dirty"], false);
        assert_eq!(json["native_path"], "/tmp/game.sgf");
        assert_eq!(json["tree"]["children"][0]["properties"][0]["key"], "W");
        assert!(json["snapshot"].get("generated_information").is_none());
        assert_eq!(decoded, result);
    }

    #[test]
    fn current_game_wire_typed_errors_use_snake_case_kinds() {
        let kinds = [
            (CurrentGameErrorKind::NoCurrentGame, "no_current_game"),
            (CurrentGameErrorKind::InvalidNodePath, "invalid_node_path"),
            (CurrentGameErrorKind::MalformedSgf, "malformed_sgf"),
            (
                CurrentGameErrorKind::UnsupportedBoardSize,
                "unsupported_board_size",
            ),
            (CurrentGameErrorKind::OccupiedPoint, "occupied_point"),
            (CurrentGameErrorKind::Suicide, "suicide"),
            (CurrentGameErrorKind::SimpleKo, "simple_ko"),
            (CurrentGameErrorKind::RootRemoval, "root_removal"),
            (CurrentGameErrorKind::DepartureInProgress, "departure_in_progress"),
            (CurrentGameErrorKind::DepartureBlocked, "departure_blocked"),
        ];

        for (kind, expected) in kinds {
            let error = CurrentGameError {
                kind,
                message: "user-presentable".to_string(),
            };
            let json = serde_json::to_value(&error).unwrap();
            assert_eq!(json["kind"], expected);
            assert_eq!(json["message"], "user-presentable");
            let decoded: CurrentGameError = serde_json::from_value(json).unwrap();
            assert_eq!(decoded.kind, kind);
        }
    }
}

#[cfg(test)]
mod foreground_engine_wire {
    use super::*;

    fn sample_profile() -> EngineProfileDto {
        EngineProfileDto {
            name: "Local KataGo".into(),
            engine_path: "/bin/katago".into(),
            model_path: Some("/models/model.bin".into()),
            config_path: Some("/configs/analysis.cfg".into()),
            working_dir: Some("/tmp/engine".into()),
            backend: EngineBackend::KataGoAnalysis,
        }
    }

    #[test]
    fn foreground_engine_snapshot_and_failure_keep_snake_case_identities() {
        let snapshot = ForegroundEngineSnapshotDto {
            revision: 4,
            lifecycle: ForegroundEngineLifecycleDto::Ready {
                run: EngineRunDto {
                    run_id: "run-1".into(),
                    profile_id: "profile-1".into(),
                    adapter_kind: EngineBackend::KataGoAnalysis,
                    profile_snapshot: sample_profile(),
                    capability_snapshot: Some(EngineCapabilitySnapshotDto {
                        adapter_kind: EngineBackend::KataGoAnalysis,
                        selected_node_analysis: true,
                        whole_game_analysis: true,
                        protocol_cancel: true,
                    }),
                },
            },
            selected_node_job: Some(AnalysisJobStartedDto {
                run_id: "run-1".into(),
                job_id: "job-selected".into(),
                lane: AnalysisJobLaneDto::SelectedNode,
                generation: 3,
                node_path: NodePath { indices: vec![0] },
            }),
            whole_game_job: Some(AnalysisJobStartedDto {
                run_id: "run-1".into(),
                job_id: "job-whole".into(),
                lane: AnalysisJobLaneDto::WholeGame,
                generation: 3,
                node_path: NodePath { indices: vec![] },
            }),
        };
        let failure = EngineFailureDto {
            operation: EngineOperationDto::Start,
            run_id: Some("run-2".into()),
            switch_id: None,
            job_id: Some("job-9".into()),
            profile_id: Some("profile-1".into()),
            kind: EngineFailureKind::Readiness,
            message: "readiness probe failed".into(),
            diagnostic_summary: Some("stderr trimmed".into()),
        };
        let event = ForegroundEngineEventDto::Failure {
            failure: failure.clone(),
        };

        let snapshot_json = serde_json::to_value(&snapshot).unwrap();
        let failure_json = serde_json::to_value(&failure).unwrap();
        let event_json = serde_json::to_value(&event).unwrap();

        assert_eq!(snapshot_json["revision"], 4);
        assert_eq!(snapshot_json["lifecycle"]["state"], "ready");
        assert_eq!(snapshot_json["selected_node_job"]["lane"], "selected_node");
        assert_eq!(snapshot_json["selected_node_job"]["job_id"], "job-selected");
        assert_eq!(snapshot_json["whole_game_job"]["lane"], "whole_game");
        assert_eq!(snapshot_json["whole_game_job"]["job_id"], "job-whole");
        assert_eq!(snapshot_json["lifecycle"]["run"]["run_id"], "run-1");
        assert_eq!(snapshot_json["lifecycle"]["run"]["profile_id"], "profile-1");
        assert_eq!(
            snapshot_json["lifecycle"]["run"]["adapter_kind"],
            "kata_go_analysis"
        );
        assert_eq!(
            snapshot_json["lifecycle"]["run"]["profile_snapshot"]["backend"],
            "kata_go_analysis"
        );
        assert_eq!(
            snapshot_json["lifecycle"]["run"]["capability_snapshot"]["protocol_cancel"],
            true
        );
        assert_eq!(failure_json["operation"], "start");
        assert_eq!(failure_json["run_id"], "run-2");
        assert_eq!(failure_json["job_id"], "job-9");
        assert_eq!(failure_json["profile_id"], "profile-1");
        assert_eq!(failure_json["kind"], "readiness");
        assert!(failure_json.get("switch_id").is_none());
        assert_eq!(event_json["type"], "failure");
        assert_eq!(event_json["failure"]["kind"], "readiness");

        let decoded_snapshot: ForegroundEngineSnapshotDto = serde_json::from_value(snapshot_json).unwrap();
        let decoded_failure: EngineFailureDto = serde_json::from_value(failure_json).unwrap();
        assert_eq!(decoded_snapshot, snapshot);
        assert_eq!(decoded_failure, failure);
    }

    #[test]
    fn foreground_engine_lifecycle_tagged_union_includes_switching_variant() {
        let run = EngineRunDto {
            run_id: "run-b".into(),
            profile_id: "profile-b".into(),
            adapter_kind: EngineBackend::KataGoAnalysis,
            profile_snapshot: sample_profile(),
            capability_snapshot: None,
        };
        let snapshot = ForegroundEngineSnapshotDto {
            revision: 1,
            lifecycle: ForegroundEngineLifecycleDto::Switching {
                primary: run.clone(),
                candidate: EngineRunDto {
                    run_id: "run-c".into(),
                    ..run
                },
                switch_id: "switch-1".into(),
            },
            selected_node_job: None,
            whole_game_job: None,
        };

        let json = serde_json::to_value(&snapshot).unwrap();
        assert_eq!(json["lifecycle"]["state"], "switching");
        assert_eq!(json["lifecycle"]["switch_id"], "switch-1");
        assert_eq!(json["lifecycle"]["candidate"]["run_id"], "run-c");
        let decoded: ForegroundEngineSnapshotDto = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, snapshot);
    }

    #[test]
    fn no_engine_omits_clean_failure_and_keeps_attempt_scoped_payload() {
        let clean = ForegroundEngineSnapshotDto {
            revision: 1,
            lifecycle: ForegroundEngineLifecycleDto::NoEngine { failure: None },
            selected_node_job: None,
            whole_game_job: None,
        };
        let clean_json = serde_json::to_value(&clean).unwrap();
        assert_eq!(clean_json["lifecycle"]["state"], "no_engine");
        assert!(clean_json["lifecycle"].get("failure").is_none());
        let decoded_clean: ForegroundEngineSnapshotDto = serde_json::from_value(
            serde_json::json!({ "revision": 1, "lifecycle": { "state": "no_engine" } }),
        )
        .unwrap();
        assert_eq!(decoded_clean, clean);

        let failure = EngineFailureDto {
            operation: EngineOperationDto::Autoload,
            run_id: Some("run-attempt".into()),
            switch_id: None,
            job_id: None,
            profile_id: Some("profile-bad".into()),
            kind: EngineFailureKind::Asset,
            message: "required engine assets are missing".into(),
            diagnostic_summary: None,
        };
        let failed = ForegroundEngineSnapshotDto {
            revision: 3,
            lifecycle: ForegroundEngineLifecycleDto::NoEngine {
                failure: Some(failure.clone()),
            },
            selected_node_job: None,
            whole_game_job: None,
        };
        let failed_json = serde_json::to_value(&failed).unwrap();
        assert_eq!(failed_json["lifecycle"]["state"], "no_engine");
        assert_eq!(failed_json["lifecycle"]["failure"]["operation"], "autoload");
        assert_eq!(failed_json["lifecycle"]["failure"]["kind"], "asset");
        assert_eq!(
            failed_json["lifecycle"]["failure"]["message"],
            "required engine assets are missing"
        );
        assert_eq!(failed_json["lifecycle"]["failure"]["profile_id"], "profile-bad");
        let decoded_failed: ForegroundEngineSnapshotDto = serde_json::from_value(failed_json).unwrap();
        assert_eq!(decoded_failed, failed);
    }
}
