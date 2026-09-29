use app_model::{AnalysisStageConditionsDto, AnalysisSwingCriteriaDto, ContinuousAnalysisBudgetDto, WorkspaceSharesDto};
use serde::ser::Serialize as SerTrait;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod recent;
pub use recent::recent_game_paths;
pub mod window_geometry;

pub const APP_PREFERENCES_FILE: &str = "lizzieyzy-next-app-preferences.json";
pub const UNREADABLE_RECOVERY_MESSAGE: &str = "Unreadable preferences isolated; restored defaults.";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferencesDto {
    #[serde(default)]
    pub workspace_shares: Option<WorkspaceSharesDto>,
    #[serde(default, deserialize_with = "deserialize_window_geometry")]
    pub window_geometry: Option<app_model::WindowGeometryDto>,
    pub workspace_visibility: app_model::WorkspaceVisibilityDto,
    pub main_window_always_on_top: bool,
    #[serde(default = "default_continuous_analysis_enabled")]
    pub continuous_analysis_enabled: bool,
    #[serde(flatten)]
    pub continuous_budget: ContinuousAnalysisBudgetDto,
    #[serde(default = "default_show_coordinates")]
    pub show_coordinates: bool,
    #[serde(default)]
    pub show_move_numbers: bool,
    #[serde(default = "default_show_ownership")]
    pub show_ownership: bool,
    #[serde(default = "default_show_policy")]
    pub show_policy: bool,
    #[serde(default = "default_show_candidates")]
    pub show_candidates: bool,
    #[serde(default = "default_candidate_limit")]
    pub candidate_limit: u32,
    #[serde(default = "default_max_visits")]
    pub default_max_visits: u32,
    #[serde(default, alias = "taskConditions", skip_serializing_if = "Option::is_none")]
    pub task_single_stage_conditions: Option<AnalysisStageConditionsDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_overview_conditions: Option<AnalysisStageConditionsDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_deep_conditions: Option<AnalysisStageConditionsDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_swing_overview_conditions: Option<AnalysisStageConditionsDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_swing_deep_conditions: Option<AnalysisStageConditionsDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_swing_criteria: Option<AnalysisSwingCriteriaDto>,
    #[serde(default = "default_review_mode")]
    pub review_mode: String,
    #[serde(default = "default_board_theme")]
    pub board_theme: String,
    #[serde(default = "default_graph_perspective")]
    pub graph_perspective: String,
    #[serde(default = "default_winrate_line")]
    pub winrate_line: bool,
    #[serde(default = "default_score_lead_line")]
    pub score_lead_line: bool,
    #[serde(default = "default_blunder_bar")]
    pub blunder_bar: bool,
    #[serde(default = "default_graph_hover")]
    pub graph_hover: bool,
    #[serde(default = "default_score_lead_scale")]
    pub score_lead_scale: u32,
    #[serde(default = "default_next_move_review_marker")]
    pub next_move_review_marker: String,
    #[serde(default = "default_sub_board_content_mode")]
    pub sub_board_content_mode: String,
    #[serde(default = "default_variation_replay_enabled")]
    pub variation_replay_enabled: bool,
    #[serde(default = "default_variation_replay_interval_ms")]
    pub variation_replay_interval_ms: u32,
    #[serde(default = "default_restore_last_session")]
    pub restore_last_session: bool,
    #[serde(default = "default_sound_enabled")]
    pub sound_enabled: bool,
    #[serde(default = "default_board_width")]
    pub default_board_width: u8,
    #[serde(default = "default_board_height")]
    pub default_board_height: u8,
    #[serde(default = "default_komi")]
    pub default_komi: f32,
    #[serde(default = "default_scoring_rule")]
    pub scoring_rule: String,
    #[serde(default)]
    pub recent_game_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferencesRecoveryDto {
    pub isolated_path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferencesLoadResultDto {
    pub preferences: AppPreferencesDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery: Option<AppPreferencesRecoveryDto>,
}

fn default_show_coordinates() -> bool {
    true
}

// Malformed geometry belongs to the geometry recovery owner, not file quarantine.
fn deserialize_window_geometry<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<app_model::WindowGeometryDto>, D::Error> {
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(value.map(|value| serde_json::from_value(value).unwrap_or_default()))
}

pub fn default_app_preferences() -> AppPreferencesDto {
    AppPreferencesDto {
        workspace_shares: None,
        window_geometry: None,
        workspace_visibility: app_model::WorkspaceVisibilityDto::default(),
        main_window_always_on_top: false,
        continuous_analysis_enabled: default_continuous_analysis_enabled(),
        continuous_budget: ContinuousAnalysisBudgetDto::default(),
        show_coordinates: default_show_coordinates(),
        show_move_numbers: false,
        show_ownership: default_show_ownership(),
        show_policy: default_show_policy(),
        show_candidates: default_show_candidates(),
        candidate_limit: default_candidate_limit(),
        default_max_visits: default_max_visits(),
        task_single_stage_conditions: Some(default_task_single_stage_conditions(default_max_visits())),
        task_overview_conditions: Some(default_task_overview_conditions()),
        task_deep_conditions: Some(default_task_deep_conditions(default_max_visits())),
        task_swing_overview_conditions: Some(default_task_overview_conditions()),
        task_swing_deep_conditions: Some(default_task_swing_deep_conditions()),
        task_swing_criteria: Some(AnalysisSwingCriteriaDto::default()),
        review_mode: default_review_mode(),
        board_theme: default_board_theme(),
        graph_perspective: default_graph_perspective(),
        winrate_line: default_winrate_line(),
        score_lead_line: default_score_lead_line(),
        blunder_bar: default_blunder_bar(),
        graph_hover: default_graph_hover(),
        score_lead_scale: default_score_lead_scale(),
        next_move_review_marker: default_next_move_review_marker(),
        sub_board_content_mode: default_sub_board_content_mode(),
        variation_replay_enabled: default_variation_replay_enabled(),
        variation_replay_interval_ms: default_variation_replay_interval_ms(),
        restore_last_session: default_restore_last_session(),
        sound_enabled: default_sound_enabled(),
        default_board_width: default_board_width(),
        default_board_height: default_board_height(),
        default_komi: default_komi(),
        scoring_rule: default_scoring_rule(),
        recent_game_paths: Vec::new(),
    }
}

pub fn normalize_app_preferences(mut preferences: AppPreferencesDto) -> AppPreferencesDto {
    if preferences.workspace_shares.is_some_and(|shares| shares.validate().is_err()) {
        preferences.workspace_shares = None;
    }
    preferences.candidate_limit = preferences.candidate_limit.clamp(1, 20);
    preferences.default_max_visits = preferences.default_max_visits.clamp(1, 1_000_000);
    preferences.variation_replay_interval_ms = preferences.variation_replay_interval_ms.clamp(100, 5000);
    if preferences.review_mode != "deep" {
        preferences.review_mode = default_review_mode();
    }
    if preferences.board_theme != "high-contrast" {
        preferences.board_theme = default_board_theme();
    }
    if preferences.graph_perspective != "sideToPlay" {
        preferences.graph_perspective = default_graph_perspective();
    }
    if !preferences.winrate_line && !preferences.score_lead_line {
        preferences.winrate_line = true;
    }
    if preferences.score_lead_scale == 0 {
        preferences.score_lead_scale = default_score_lead_scale();
    } else {
        preferences.score_lead_scale = preferences.score_lead_scale.clamp(1, 1000);
    }
    if preferences.next_move_review_marker != "off" && preferences.next_move_review_marker != "graded" {
        preferences.next_move_review_marker = default_next_move_review_marker();
    }
    if preferences.sub_board_content_mode != "raw" {
        preferences.sub_board_content_mode = default_sub_board_content_mode();
    }
    if preferences.scoring_rule != "area" && preferences.scoring_rule != "territory" {
        preferences.scoring_rule = default_scoring_rule();
    }
    let single_stage = preferences
        .task_single_stage_conditions
        .get_or_insert_with(|| default_task_single_stage_conditions(preferences.default_max_visits))
        .clone();
    preferences
        .task_overview_conditions
        .get_or_insert_with(default_task_overview_conditions);
    preferences.task_deep_conditions.get_or_insert_with(|| {
        let mut deep = single_stage;
        deep.total_visits.value = deep.total_visits.value.max(500);
        deep
    });
    preferences
        .task_swing_overview_conditions
        .get_or_insert_with(default_task_overview_conditions);
    preferences
        .task_swing_deep_conditions
        .get_or_insert_with(default_task_swing_deep_conditions);
    preferences
        .task_swing_criteria
        .get_or_insert_with(AnalysisSwingCriteriaDto::default);
    preferences
}

pub fn load_from_path(path: &Path) -> Result<AppPreferencesLoadResultDto, String> {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<AppPreferencesDto>(&contents) {
            Ok(preferences) if preferences.continuous_budget.validate().is_ok() => {
                let preferences = normalize_app_preferences(preferences);
                let valid = preferences
                    .task_single_stage_conditions
                    .as_ref()
                    .is_some_and(|conditions| conditions.validate_single_stage().is_ok())
                    && preferences
                        .task_overview_conditions
                        .as_ref()
                        .zip(preferences.task_deep_conditions.as_ref())
                        .is_some_and(|(overview, deep)| {
                            AnalysisStageConditionsDto::validate_all_positions_two_stage(overview, deep)
                                .is_ok()
                        })
                    && preferences
                        .task_swing_overview_conditions
                        .as_ref()
                        .zip(preferences.task_swing_deep_conditions.as_ref())
                        .is_some_and(|(overview, deep)| {
                            overview.validate_single_stage().is_ok() && deep.validate_single_stage().is_ok()
                        })
                    && preferences
                        .task_swing_criteria
                        .as_ref()
                        .is_some_and(|criteria| criteria.validate().is_ok());
                if valid {
                    Ok(AppPreferencesLoadResultDto {
                        preferences,
                        recovery: None,
                    })
                } else {
                    recover_unreadable(path)
                }
            }
            _ => recover_unreadable(path),
        },
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(AppPreferencesLoadResultDto {
            preferences: default_app_preferences(),
            recovery: None,
        }),
        Err(_) => recover_unreadable(path),
    }
}

pub fn save_to_path(path: &Path, preferences: AppPreferencesDto) -> Result<AppPreferencesDto, String> {
    if let Some(shares) = preferences.workspace_shares {
        shares.validate()?;
    }
    if !(2..=25).contains(&preferences.default_board_width) {
        return Err(format!(
            "Default board width must be between 2 and 25, got {}.",
            preferences.default_board_width
        ));
    }
    if !(2..=25).contains(&preferences.default_board_height) {
        return Err(format!(
            "Default board height must be between 2 and 25, got {}.",
            preferences.default_board_height
        ));
    }
    if !preferences.default_komi.is_finite() {
        return Err("Default komi must be finite.".to_string());
    }
    preferences.continuous_budget.validate()?;
    let single_stage = preferences
        .task_single_stage_conditions
        .as_ref()
        .ok_or_else(|| "Single-stage task conditions are required.".to_string())?;
    single_stage.validate_single_stage()?;
    let (overview, deep) = preferences
        .task_overview_conditions
        .as_ref()
        .zip(preferences.task_deep_conditions.as_ref())
        .ok_or_else(|| "Both overview and deep task conditions are required.".to_string())?;
    AnalysisStageConditionsDto::validate_all_positions_two_stage(overview, deep)?;
    let (swing_overview, swing_deep) = preferences
        .task_swing_overview_conditions
        .as_ref()
        .zip(preferences.task_swing_deep_conditions.as_ref())
        .ok_or_else(|| "Both swing overview and deep task conditions are required.".to_string())?;
    swing_overview.validate_single_stage()?;
    swing_deep.validate_single_stage()?;
    preferences
        .task_swing_criteria
        .as_ref()
        .ok_or_else(|| "Swing selection criteria are required.".to_string())?
        .validate()?;
    let preferences = normalize_app_preferences(preferences);
    replace_json_file(path, &preferences)?;
    Ok(preferences)
}

fn recover_unreadable(path: &Path) -> Result<AppPreferencesLoadResultDto, String> {
    let isolated_path = isolate_unreadable(path)?;
    Ok(AppPreferencesLoadResultDto {
        preferences: default_app_preferences(),
        recovery: Some(AppPreferencesRecoveryDto {
            isolated_path: isolated_path.display().to_string(),
            message: UNREADABLE_RECOVERY_MESSAGE.to_string(),
        }),
    })
}

fn isolate_unreadable(path: &Path) -> Result<PathBuf, String> {
    let isolated = isolated_path(path);
    fs::rename(path, &isolated).map_err(|err| {
        format!(
            "failed to isolate unreadable preferences {}: {err}",
            path.display()
        )
    })?;
    Ok(isolated)
}

fn isolated_path(path: &Path) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| APP_PREFERENCES_FILE.to_string());
    path.with_file_name(format!("{file_name}.unreadable-{unique}"))
}

fn replace_json_file<T: SerTrait>(path: &Path, value: &T) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|err| format!("failed to serialize {}: {err}", path.display()))?;
    atomic_replace_file(path, &json)
}

fn atomic_replace_file(path: &Path, contents: &str) -> Result<(), String> {
    atomic_replace_file_with(path, contents, |from, to| fs::rename(from, to))
}

fn atomic_replace_file_with(
    path: &Path,
    contents: &str,
    rename: impl FnOnce(&Path, &Path) -> io::Result<()>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create app preferences directory {}: {err}",
                parent.display()
            )
        })?;
    }
    let tmp = tmp_path(path);
    fs::write(&tmp, contents).map_err(|err| format!("failed to write {}: {err}", tmp.display()))?;
    match rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = fs::remove_file(&tmp);
            Err(format!("failed to replace {}: {err}", path.display()))
        }
    }
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut tmp = path.as_os_str().to_os_string();
    tmp.push(".tmp");
    PathBuf::from(tmp)
}

fn default_continuous_analysis_enabled() -> bool {
    true
}

fn default_show_ownership() -> bool {
    true
}

fn default_show_policy() -> bool {
    true
}

fn default_show_candidates() -> bool {
    true
}

fn default_candidate_limit() -> u32 {
    8
}

fn default_max_visits() -> u32 {
    800
}

fn default_task_single_stage_conditions(total_visits: u32) -> AnalysisStageConditionsDto {
    AnalysisStageConditionsDto {
        total_visits: app_model::AnalysisTaskLimitDto {
            enabled: true,
            value: total_visits,
        },
        ..AnalysisStageConditionsDto::default()
    }
}

fn default_task_overview_conditions() -> AnalysisStageConditionsDto {
    AnalysisStageConditionsDto {
        time_seconds: app_model::AnalysisTaskLimitDto {
            enabled: false,
            value: 10,
        },
        total_visits: app_model::AnalysisTaskLimitDto {
            enabled: true,
            value: 32,
        },
        leading_candidate_visits: app_model::AnalysisTaskLimitDto {
            enabled: false,
            value: 32,
        },
    }
}

fn default_task_deep_conditions(total_visits: u32) -> AnalysisStageConditionsDto {
    AnalysisStageConditionsDto {
        total_visits: app_model::AnalysisTaskLimitDto {
            enabled: true,
            value: total_visits.max(500),
        },
        ..AnalysisStageConditionsDto::default()
    }
}

fn default_task_swing_deep_conditions() -> AnalysisStageConditionsDto {
    AnalysisStageConditionsDto {
        time_seconds: app_model::AnalysisTaskLimitDto {
            enabled: true,
            value: 10,
        },
        total_visits: app_model::AnalysisTaskLimitDto {
            enabled: false,
            value: 800,
        },
        leading_candidate_visits: app_model::AnalysisTaskLimitDto {
            enabled: false,
            value: 500,
        },
    }
}

fn default_review_mode() -> String {
    "quick".to_string()
}

fn default_board_theme() -> String {
    "classic".to_string()
}

fn default_graph_perspective() -> String {
    "black".to_string()
}

fn default_winrate_line() -> bool {
    true
}

fn default_score_lead_line() -> bool {
    true
}

fn default_blunder_bar() -> bool {
    false
}

fn default_graph_hover() -> bool {
    true
}

fn default_score_lead_scale() -> u32 {
    15
}

fn default_next_move_review_marker() -> String {
    "variations".to_string()
}

fn default_sub_board_content_mode() -> String {
    "variation".to_string()
}

fn default_variation_replay_enabled() -> bool {
    false
}

fn default_variation_replay_interval_ms() -> u32 {
    500
}

fn default_restore_last_session() -> bool {
    false
}

fn default_sound_enabled() -> bool {
    true
}

fn default_board_width() -> u8 {
    19
}

fn default_board_height() -> u8 {
    19
}

fn default_komi() -> f32 {
    7.5
}

fn default_scoring_rule() -> String {
    "area".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::ser::{Error as SerError, Serializer};

    struct FailingSerialize;

    impl SerTrait for FailingSerialize {
        fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(SerError::custom("serialization boom"))
        }
    }

    fn temp_prefs() -> (PathBuf, PathBuf) {
        let unique = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        );
        let dir = std::env::temp_dir()
            .join("lizzieyzy-app-preferences")
            .join(unique);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(APP_PREFERENCES_FILE);
        (dir, path)
    }

    fn sample_preferences() -> AppPreferencesDto {
        AppPreferencesDto {
            workspace_shares: None,
            window_geometry: None,
            workspace_visibility: app_model::WorkspaceVisibilityDto { left: false, right: true },
            continuous_analysis_enabled: false,
            continuous_budget: ContinuousAnalysisBudgetDto {
                continuous_time_limit_enabled: false,
                continuous_time_limit_seconds: 7,
                continuous_visits_limit_enabled: true,
                continuous_visits_limit: 123,
                continuous_stop_on_empty_board: true,
            },
            show_coordinates: false,
            show_move_numbers: true,
            show_ownership: false,
            show_policy: false,
            show_candidates: false,
            candidate_limit: 3,
            default_max_visits: 200,
            task_single_stage_conditions: Some(default_task_single_stage_conditions(200)),
            task_overview_conditions: Some(default_task_overview_conditions()),
            task_deep_conditions: Some(default_task_deep_conditions(800)),
            task_swing_overview_conditions: Some(default_task_overview_conditions()),
            task_swing_deep_conditions: Some(default_task_swing_deep_conditions()),
            task_swing_criteria: Some(AnalysisSwingCriteriaDto::default()),
            review_mode: "deep".to_string(),
            board_theme: "high-contrast".to_string(),
            graph_perspective: "sideToPlay".to_string(),
            winrate_line: false,
            score_lead_line: true,
            blunder_bar: true,
            graph_hover: false,
            score_lead_scale: 20,
            next_move_review_marker: "graded".to_string(),
            sub_board_content_mode: "raw".to_string(),
            variation_replay_enabled: true,
            variation_replay_interval_ms: 250,
            restore_last_session: true,
            sound_enabled: true,
            default_board_width: 19,
            default_board_height: 19,
            default_komi: 7.5,
            scoring_rule: default_scoring_rule(),
            recent_game_paths: Vec::new(),
        }
    }

    #[test]
    fn task_stage_presets_roundtrip_migrate_legacy_deep_and_reject_invalid_writes() {
        let (dir, path) = temp_prefs();
        let original = save_to_path(&path, sample_preferences()).unwrap();
        let mut value = serde_json::to_value(&original).unwrap();
        value["taskOverviewConditions"] = serde_json::json!({
            "time_seconds": {"enabled": false, "value": 10},
            "total_visits": {"enabled": true, "value": 32},
            "leading_candidate_visits": {"enabled": false, "value": 32}
        });
        value["taskDeepConditions"] = serde_json::json!({
            "time_seconds": {"enabled": true, "value": 10},
            "total_visits": {"enabled": true, "value": 500},
            "leading_candidate_visits": {"enabled": true, "value": 500}
        });
        save_to_path(&path, serde_json::from_value(value.clone()).unwrap()).unwrap();
        let loaded = serde_json::to_value(load_from_path(&path).unwrap().preferences).unwrap();
        assert_eq!(loaded["taskOverviewConditions"], value["taskOverviewConditions"]);
        assert_eq!(loaded["taskDeepConditions"], value["taskDeepConditions"]);
        let durable = fs::read(&path).unwrap();
        let mut invalid_swing = value.clone();
        invalid_swing["taskSwingCriteria"]["winrate_change_percentage_points"]["enabled"] =
            serde_json::json!(false);
        assert!(save_to_path(&path, serde_json::from_value(invalid_swing).unwrap()).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        value["taskDeepConditions"]["total_visits"]["value"] = serde_json::json!(499);
        assert!(save_to_path(&path, serde_json::from_value(value).unwrap()).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        fs::write(&path, r#"{"defaultMaxVisits":900,"taskConditions":{"time_seconds":{"enabled":false,"value":10},"total_visits":{"enabled":true,"value":900},"leading_candidate_visits":{"enabled":false,"value":500}}}"#).unwrap();
        let migrated = load_from_path(&path).unwrap();
        assert!(migrated.recovery.is_none());
        assert_eq!(
            migrated
                .preferences
                .task_single_stage_conditions
                .as_ref()
                .unwrap()
                .total_visits
                .value,
            900
        );
        assert_eq!(
            migrated
                .preferences
                .task_overview_conditions
                .as_ref()
                .unwrap()
                .total_visits
                .value,
            32
        );
        assert_eq!(
            migrated
                .preferences
                .task_deep_conditions
                .as_ref()
                .unwrap()
                .total_visits
                .value,
            900
        );
        let swing_deep = migrated.preferences.task_swing_deep_conditions.as_ref().unwrap();
        assert!(swing_deep.time_seconds.enabled);
        assert_eq!(swing_deep.time_seconds.value, 10);
        assert!(!swing_deep.total_visits.enabled);
        assert_eq!(
            migrated.preferences.task_swing_criteria,
            Some(AnalysisSwingCriteriaDto::default())
        );

        fs::write(&path, r#"{"continuousAnalysisEnabled":false,"defaultMaxVisits":32,"taskConditions":{"time_seconds":{"enabled":false,"value":10},"total_visits":{"enabled":true,"value":32},"leading_candidate_visits":{"enabled":false,"value":32}}}"#).unwrap();
        let migrated = load_from_path(&path).unwrap();
        assert!(migrated.recovery.is_none());
        assert!(!migrated.preferences.continuous_analysis_enabled);
        assert_eq!(
            migrated
                .preferences
                .task_single_stage_conditions
                .as_ref()
                .unwrap()
                .total_visits
                .value,
            32
        );
        assert_eq!(
            migrated
                .preferences
                .task_deep_conditions
                .as_ref()
                .unwrap()
                .total_visits
                .value,
            500
        );
        let saved = save_to_path(&path, migrated.preferences).unwrap();
        let serialized = serde_json::to_value(saved).unwrap();
        assert!(serialized.get("taskConditions").is_none());
        assert!(serialized.get("taskSingleStageConditions").is_some());
        assert!(serialized.get("taskOverviewConditions").is_some());
        assert!(serialized.get("taskDeepConditions").is_some());
        assert!(serialized.get("taskSwingOverviewConditions").is_some());
        assert!(serialized.get("taskSwingDeepConditions").is_some());
        assert!(serialized.get("taskSwingCriteria").is_some());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn missing_storage_loads_owner_defaults() {
        let (dir, path) = temp_prefs();
        let loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.preferences, default_app_preferences());
        assert!(loaded.recovery.is_none());
        assert!(!path.exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn partial_storage_fills_missing_keys_from_owner_defaults() {
        let (dir, path) = temp_prefs();
        fs::write(&path, r#"{"showCandidates":false,"candidateLimit":2}"#).unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert!(loaded.preferences.show_coordinates);
        assert!(!loaded.preferences.show_move_numbers);
        assert!(loaded.preferences.continuous_analysis_enabled);
        assert!(!loaded.preferences.show_candidates);
        assert_eq!(loaded.preferences.candidate_limit, 2);
        assert!(loaded.preferences.show_ownership);
        assert!(loaded.preferences.show_policy);
        assert_eq!(loaded.preferences.review_mode, "quick");
        assert_eq!(loaded.preferences.board_theme, "classic");
        assert_eq!(loaded.preferences.next_move_review_marker, "variations");
        assert_eq!(loaded.preferences.sub_board_content_mode, "variation");
        assert!(!loaded.preferences.variation_replay_enabled);
        assert_eq!(loaded.preferences.variation_replay_interval_ms, 500);
        assert!(loaded.recovery.is_none());
        assert!(loaded.preferences.sound_enabled);
        assert_eq!(loaded.preferences.default_board_width, 19);
        assert_eq!(loaded.preferences.default_board_height, 19);
        assert_eq!(loaded.preferences.default_komi, 7.5);
        assert_eq!(loaded.preferences.scoring_rule, "area");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_sub_board_content_mode_defaults_to_variation() {
        let (dir, path) = temp_prefs();
        fs::write(&path, r#"{"showCandidates":true}"#).unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.preferences.sub_board_content_mode, "variation");
        assert!(loaded.recovery.is_none());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_variation_replay_defaults_to_off_and_500ms() {
        let (dir, path) = temp_prefs();
        fs::write(&path, r#"{"showCandidates":true}"#).unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert!(!loaded.preferences.variation_replay_enabled);
        assert_eq!(loaded.preferences.variation_replay_interval_ms, 500);
        assert!(loaded.recovery.is_none());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_restore_last_session_defaults_to_off() {
        let (dir, path) = temp_prefs();
        fs::write(&path, r#"{"showCandidates":true}"#).unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert!(!loaded.preferences.restore_last_session);
        assert!(!default_app_preferences().restore_last_session);
        assert!(loaded.recovery.is_none());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn variation_replay_interval_clamps_to_100_5000() {
        let low = normalize_app_preferences(AppPreferencesDto {
            variation_replay_interval_ms: 50,
            ..default_app_preferences()
        });
        let high = normalize_app_preferences(AppPreferencesDto {
            variation_replay_interval_ms: 9000,
            ..default_app_preferences()
        });
        assert_eq!(low.variation_replay_interval_ms, 100);
        assert_eq!(high.variation_replay_interval_ms, 5000);
        assert!(!default_app_preferences().variation_replay_enabled);
        assert_eq!(default_app_preferences().variation_replay_interval_ms, 500);
    }

    #[test]
    fn unreadable_storage_is_isolated_and_loads_defaults() {
        let (dir, path) = temp_prefs();
        fs::write(&path, "{not-json").unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.preferences, default_app_preferences());
        let recovery = loaded.recovery.expect("recovery");
        assert_eq!(recovery.message, UNREADABLE_RECOVERY_MESSAGE);
        let isolated = PathBuf::from(&recovery.isolated_path);
        assert!(isolated.exists());
        assert_eq!(fs::read_to_string(&isolated).unwrap(), "{not-json");
        assert!(!path.exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn successful_write_is_reloadable_as_restart() {
        let (dir, path) = temp_prefs();
        let saved = save_to_path(&path, sample_preferences()).unwrap();
        assert_eq!(saved, sample_preferences());
        assert!(!tmp_path(&path).exists());

        let reloaded = load_from_path(&path).unwrap();
        assert_eq!(reloaded.preferences, sample_preferences());
        assert!(reloaded.recovery.is_none());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn invalid_continuous_budget_preserves_durable_settings() {
        let (dir, path) = temp_prefs();
        let original = save_to_path(&path, sample_preferences()).unwrap();
        for (key, value) in [
            ("continuousTimeLimitSeconds", serde_json::json!(0)),
            ("continuousVisitsLimit", serde_json::json!(0)),
            ("continuousTimeLimitSeconds", serde_json::json!(1.5)),
            ("continuousVisitsLimit", serde_json::json!(4_294_967_296u64)),
        ] {
            let mut input = serde_json::to_value(&original).unwrap();
            input["continuousTimeLimitEnabled"] = serde_json::json!(true);
            input["continuousVisitsLimitEnabled"] = serde_json::json!(true);
            input[key] = value;
            if let Ok(preferences) = serde_json::from_value(input) {
                assert!(
                    save_to_path(&path, preferences).is_err(),
                    "accepted invalid {key}"
                );
            }
            assert_eq!(load_from_path(&path).unwrap().preferences, original);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn serialize_failure_keeps_previous_durable_value() {
        let (dir, path) = temp_prefs();
        save_to_path(&path, sample_preferences()).unwrap();
        let before = fs::read_to_string(&path).unwrap();

        let err = replace_json_file(&path, &FailingSerialize).unwrap_err();
        assert!(err.contains("serialize"));
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert!(!tmp_path(&path).exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn write_failure_keeps_previous_durable_value() {
        let (dir, path) = temp_prefs();
        save_to_path(&path, sample_preferences()).unwrap();
        let before = fs::read_to_string(&path).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&dir).unwrap().permissions();
            permissions.set_mode(0o555);
            fs::set_permissions(&dir, permissions).unwrap();
            let err = save_to_path(&path, default_app_preferences());
            let mut permissions = fs::metadata(&dir).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&dir, permissions).unwrap();
            assert!(err.is_err());
        }

        let loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.preferences, sample_preferences());
        assert_eq!(fs::read_to_string(&path).unwrap(), before);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn replace_failure_keeps_previous_durable_value() {
        let (dir, path) = temp_prefs();
        save_to_path(&path, sample_preferences()).unwrap();
        let before = fs::read_to_string(&path).unwrap();

        let err = atomic_replace_file_with(&path, "{\"replaced\":true}", |_from, _to| {
            Err(io::Error::other("rename boom"))
        })
        .unwrap_err();
        assert!(err.contains("replace"));
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert!(!tmp_path(&path).exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn analysis_cache_preferences_are_absent_from_dto_and_defaults() {
        let defaults = serde_json::to_value(default_app_preferences()).unwrap();
        let object = defaults.as_object().unwrap();
        assert!(!object.contains_key("autoLoadCache"));
        assert!(!object.contains_key("autoSaveAnalysis"));

        let loaded: AppPreferencesDto = serde_json::from_value(serde_json::json!({
            "autoLoadCache": false,
            "autoSaveAnalysis": false,
            "showCandidates": true
        }))
        .unwrap();
        let roundtrip = serde_json::to_value(normalize_app_preferences(loaded)).unwrap();
        let roundtrip_object = roundtrip.as_object().unwrap();
        assert!(!roundtrip_object.contains_key("autoLoadCache"));
        assert!(!roundtrip_object.contains_key("autoSaveAnalysis"));
        assert_eq!(
            roundtrip_object.get("showCandidates"),
            Some(&serde_json::json!(true))
        );
    }

    #[test]
    fn default_board_and_komi_preferences_roundtrip_and_reject_invalid_writes() {
        let (dir, path) = temp_prefs();
        let mut custom = sample_preferences();
        custom.default_board_width = 13;
        custom.default_board_height = 17;
        custom.default_komi = 6.5;

        let saved = save_to_path(&path, custom.clone()).unwrap();
        assert_eq!(saved.default_board_width, 13);
        assert_eq!(saved.default_board_height, 17);
        assert_eq!(saved.default_komi, 6.5);

        let reloaded = load_from_path(&path).unwrap();
        assert_eq!(reloaded.preferences.default_board_width, 13);
        assert_eq!(reloaded.preferences.default_board_height, 17);
        assert_eq!(reloaded.preferences.default_komi, 6.5);

        let serialized = serde_json::to_value(&reloaded.preferences).unwrap();
        assert_eq!(serialized["defaultBoardWidth"], 13);
        assert_eq!(serialized["defaultBoardHeight"], 17);
        assert_eq!(serialized["defaultKomi"], 6.5);

        let durable = fs::read(&path).unwrap();

        // Invalid width (< 2, > 25)
        let mut invalid_width_low = custom.clone();
        invalid_width_low.default_board_width = 1;
        assert!(save_to_path(&path, invalid_width_low).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        let mut invalid_width_high = custom.clone();
        invalid_width_high.default_board_width = 26;
        assert!(save_to_path(&path, invalid_width_high).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        // Invalid height (< 2, > 25)
        let mut invalid_height_low = custom.clone();
        invalid_height_low.default_board_height = 1;
        assert!(save_to_path(&path, invalid_height_low).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        let mut invalid_height_high = custom.clone();
        invalid_height_high.default_board_height = 26;
        assert!(save_to_path(&path, invalid_height_high).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        // Nonfinite komi (NaN, infinity, neg infinity)
        let mut invalid_komi_nan = custom.clone();
        invalid_komi_nan.default_komi = f32::NAN;
        assert!(save_to_path(&path, invalid_komi_nan).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        let mut invalid_komi_inf = custom.clone();
        invalid_komi_inf.default_komi = f32::INFINITY;
        assert!(save_to_path(&path, invalid_komi_inf).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        let mut invalid_komi_neginf = custom.clone();
        invalid_komi_neginf.default_komi = f32::NEG_INFINITY;
        assert!(save_to_path(&path, invalid_komi_neginf).is_err());
        assert_eq!(fs::read(&path).unwrap(), durable);

        // Verify durable preferences still load untouched
        let reloaded_after_rejects = load_from_path(&path).unwrap();
        assert_eq!(reloaded_after_rejects.preferences.default_board_width, 13);
        assert_eq!(reloaded_after_rejects.preferences.default_board_height, 17);
        assert_eq!(reloaded_after_rejects.preferences.default_komi, 6.5);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn sound_enabled_defaults_to_true_and_disabled_survives_roundtrip_and_unrelated_writes() {
        let (dir, path) = temp_prefs();

        // Missing old JSON defaults to soundEnabled: true
        fs::write(&path, r#"{"showCandidates":true}"#).unwrap();
        let loaded = load_from_path(&path).unwrap();
        assert!(loaded.preferences.sound_enabled);
        assert!(default_app_preferences().sound_enabled);

        // Disabling sound survives save and durable reload
        let mut prefs = loaded.preferences;
        prefs.sound_enabled = false;
        let saved = save_to_path(&path, prefs).unwrap();
        assert!(!saved.sound_enabled);

        let reloaded = load_from_path(&path).unwrap();
        assert!(!reloaded.preferences.sound_enabled);

        let serialized = serde_json::to_value(&reloaded.preferences).unwrap();
        assert_eq!(serialized["soundEnabled"], false);

        // Unrelated preference writes retain disabled sound
        let mut updated = reloaded.preferences;
        updated.candidate_limit = 5;
        updated.board_theme = "high-contrast".to_string();
        let saved_updated = save_to_path(&path, updated).unwrap();
        assert!(!saved_updated.sound_enabled);
        assert_eq!(saved_updated.candidate_limit, 5);

        let reloaded_updated = load_from_path(&path).unwrap();
        assert!(!reloaded_updated.preferences.sound_enabled);
        assert_eq!(reloaded_updated.preferences.candidate_limit, 5);
        assert_eq!(reloaded_updated.preferences.board_theme, "high-contrast");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn scoring_rule_defaults_to_area_and_normalizes_unknown() {
        let (dir, path) = temp_prefs();

        // Default preferences first use is "area"
        let defaults = default_app_preferences();
        assert_eq!(defaults.scoring_rule, "area");

        // Serialized defaults contain camelCase scoringRule
        let defaults_json = serde_json::to_value(&defaults).unwrap();
        assert_eq!(defaults_json["scoringRule"], "area");
        // No compensation or handicap values stored
        let obj = defaults_json.as_object().unwrap();
        assert!(!obj.contains_key("compensation"));
        assert!(!obj.contains_key("handicapCompensation"));
        assert!(!obj.contains_key("h"));

        // Loading from nonexistent path defaults to area
        let missing = load_from_path(&path).unwrap();
        assert_eq!(missing.preferences.scoring_rule, "area");

        // Legacy JSON without scoringRule defaults to area
        fs::write(&path, r#"{"showCandidates":true}"#).unwrap();
        let loaded_legacy = load_from_path(&path).unwrap();
        assert_eq!(loaded_legacy.preferences.scoring_rule, "area");

        // Normalization preserves "area" and "territory", maps unknown to "area"
        for allowed in ["area", "territory"] {
            let pref = AppPreferencesDto {
                scoring_rule: allowed.to_string(),
                ..default_app_preferences()
            };
            assert_eq!(normalize_app_preferences(pref).scoring_rule, allowed);
        }

        for unknown in ["", "japanese", "chinese", "AREA", "TERRITORY", "unknown", "aga"] {
            let pref = AppPreferencesDto {
                scoring_rule: unknown.to_string(),
                ..default_app_preferences()
            };
            assert_eq!(normalize_app_preferences(pref).scoring_rule, "area");
        }

        // Loading file with unknown scoringRule normalizes to area
        fs::write(&path, r#"{"showCandidates":true,"scoringRule":"japanese"}"#).unwrap();
        let loaded_unknown = load_from_path(&path).unwrap();
        assert_eq!(loaded_unknown.preferences.scoring_rule, "area");

        // Saving unknown scoringRule normalizes and persists area
        let mut to_save = sample_preferences();
        to_save.scoring_rule = "invalid_rule".to_string();
        let saved = save_to_path(&path, to_save).unwrap();
        assert_eq!(saved.scoring_rule, "area");
        let reloaded = load_from_path(&path).unwrap();
        assert_eq!(reloaded.preferences.scoring_rule, "area");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn scoring_rule_persists_area_and_territory_and_retains_durable_on_failed_replace() {
        let (dir, path) = temp_prefs();
        let mut prefs = sample_preferences();
        prefs.scoring_rule = "territory".to_string();
        let saved = save_to_path(&path, prefs.clone()).unwrap();
        assert_eq!(saved.scoring_rule, "territory");

        let reloaded = load_from_path(&path).unwrap();
        assert_eq!(reloaded.preferences.scoring_rule, "territory");
        let durable_json = fs::read_to_string(&path).unwrap();

        // When atomic replace fails during save, previous durable scoringRule is retained
        let mut new_prefs = reloaded.preferences.clone();
        new_prefs.scoring_rule = "area".to_string();
        let serialized = serde_json::to_string(&new_prefs).unwrap();
        let err = atomic_replace_file_with(&path, &serialized, |_from, _to| {
            Err(io::Error::other("rename failed"))
        })
        .unwrap_err();
        assert!(err.contains("replace"));

        // Verify previous durable file is untouched and still loads territory
        assert_eq!(fs::read_to_string(&path).unwrap(), durable_json);
        assert!(!tmp_path(&path).exists());
        let reloaded_after_failure = load_from_path(&path).unwrap();
        assert_eq!(reloaded_after_failure.preferences.scoring_rule, "territory");

        // Normal successful save of area updates durable value
        let saved_area = save_to_path(&path, new_prefs).unwrap();
        assert_eq!(saved_area.scoring_rule, "area");
        let reloaded_area = load_from_path(&path).unwrap();
        assert_eq!(reloaded_area.preferences.scoring_rule, "area");

        // Unrelated preference writes retain current durable scoring rule
        let mut updated_unrelated = reloaded_area.preferences;
        updated_unrelated.candidate_limit = 7;
        let saved_unrelated = save_to_path(&path, updated_unrelated).unwrap();
        assert_eq!(saved_unrelated.scoring_rule, "area");
        assert_eq!(saved_unrelated.candidate_limit, 7);
        let reloaded_unrelated = load_from_path(&path).unwrap();
        assert_eq!(reloaded_unrelated.preferences.scoring_rule, "area");
        assert_eq!(reloaded_unrelated.preferences.candidate_limit, 7);

        let _ = fs::remove_dir_all(dir);
    }
}

#[cfg(test)]
mod workspace_tests {
    use super::*;

    #[test]
    fn invalid_window_record_preserves_other_preferences() {
        let preferences: AppPreferencesDto = serde_json::from_str(
            r#"{"windowGeometry":{"x":"invalid"},"boardTheme":"high-contrast","soundEnabled":false,"recentGamePaths":["/games/test.sgf"]}"#,
        ).unwrap();
        assert_eq!(preferences.window_geometry, Some(app_model::WindowGeometryDto::default()));
        assert_eq!(preferences.board_theme, "high-contrast");
        assert!(!preferences.sound_enabled);
        assert_eq!(preferences.recent_game_paths, ["/games/test.sgf"]);
        let legacy: AppPreferencesDto = serde_json::from_str(r#"{"boardTheme":"high-contrast"}"#).unwrap();
        assert_eq!(legacy.window_geometry, None);
    }

    #[test]
    fn legacy_and_invalid_shares_preserve_unrelated_preferences() {
        let legacy: AppPreferencesDto = serde_json::from_str(r#"{"boardTheme":"high-contrast","soundEnabled":false}"#).unwrap();
        assert_eq!(legacy.workspace_shares, None);
        let mut invalid = legacy.clone();
        invalid.workspace_shares = Some(WorkspaceSharesDto { left: -0.1, right: 0.2 });
        assert_eq!(normalize_app_preferences(invalid), normalize_app_preferences(legacy));
    }

    #[test]
    fn share_bounds_reject_nonfinite_and_no_center_space() {
        for (left, right) in [(f64::NAN, 0.2), (0.2, f64::INFINITY), (-0.1, 0.2), (0.7, 0.3)] {
            assert!(WorkspaceSharesDto { left, right }.validate().is_err());
        }
        assert!(WorkspaceSharesDto { left: 0.0, right: 0.0 }.validate().is_ok());
    }
}
