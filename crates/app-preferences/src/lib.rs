use app_model::ContinuousAnalysisBudgetDto;
use serde::ser::Serialize as SerTrait;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const APP_PREFERENCES_FILE: &str = "lizzieyzy-next-app-preferences.json";
pub const UNREADABLE_RECOVERY_MESSAGE: &str = "Unreadable preferences isolated; restored defaults.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferencesDto {
    #[serde(default = "default_continuous_analysis_enabled")]
    pub continuous_analysis_enabled: bool,
    #[serde(flatten)]
    pub continuous_budget: ContinuousAnalysisBudgetDto,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferencesRecoveryDto {
    pub isolated_path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPreferencesLoadResultDto {
    pub preferences: AppPreferencesDto,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery: Option<AppPreferencesRecoveryDto>,
}

pub fn default_app_preferences() -> AppPreferencesDto {
    AppPreferencesDto {
        continuous_analysis_enabled: default_continuous_analysis_enabled(),
        continuous_budget: ContinuousAnalysisBudgetDto::default(),
        show_ownership: default_show_ownership(),
        show_policy: default_show_policy(),
        show_candidates: default_show_candidates(),
        candidate_limit: default_candidate_limit(),
        default_max_visits: default_max_visits(),
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
    }
}

pub fn normalize_app_preferences(mut preferences: AppPreferencesDto) -> AppPreferencesDto {
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
    preferences
}

pub fn load_from_path(path: &Path) -> Result<AppPreferencesLoadResultDto, String> {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<AppPreferencesDto>(&contents) {
            Ok(preferences) if preferences.continuous_budget.validate().is_ok() => {
                Ok(AppPreferencesLoadResultDto {
                    preferences: normalize_app_preferences(preferences),
                    recovery: None,
                })
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
    preferences.continuous_budget.validate()?;
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
            continuous_analysis_enabled: false,
            continuous_budget: ContinuousAnalysisBudgetDto {
                continuous_time_limit_enabled: false,
                continuous_time_limit_seconds: 7,
                continuous_visits_limit_enabled: true,
                continuous_visits_limit: 123,
                continuous_stop_on_empty_board: true,
            },
            show_ownership: false,
            show_policy: false,
            show_candidates: false,
            candidate_limit: 3,
            default_max_visits: 200,
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
        }
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
}
