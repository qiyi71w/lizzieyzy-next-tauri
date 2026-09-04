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
    #[serde(default = "default_show_ownership")]
    pub show_ownership: bool,
    #[serde(default = "default_show_policy")]
    pub show_policy: bool,
    #[serde(default = "default_show_candidates")]
    pub show_candidates: bool,
    #[serde(default = "default_candidate_limit")]
    pub candidate_limit: u32,
    #[serde(default = "default_auto_load_cache")]
    pub auto_load_cache: bool,
    #[serde(default = "default_auto_save_analysis")]
    pub auto_save_analysis: bool,
    #[serde(default = "default_max_visits")]
    pub default_max_visits: u32,
    #[serde(default = "default_review_mode")]
    pub review_mode: String,
    #[serde(default = "default_board_theme")]
    pub board_theme: String,
    #[serde(default = "default_sub_board_content_mode")]
    pub sub_board_content_mode: String,
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
        show_ownership: default_show_ownership(),
        show_policy: default_show_policy(),
        show_candidates: default_show_candidates(),
        candidate_limit: default_candidate_limit(),
        auto_load_cache: default_auto_load_cache(),
        auto_save_analysis: default_auto_save_analysis(),
        default_max_visits: default_max_visits(),
        review_mode: default_review_mode(),
        board_theme: default_board_theme(),
        sub_board_content_mode: default_sub_board_content_mode(),
    }
}

pub fn normalize_app_preferences(mut preferences: AppPreferencesDto) -> AppPreferencesDto {
    preferences.candidate_limit = preferences.candidate_limit.clamp(1, 20);
    preferences.default_max_visits = preferences.default_max_visits.clamp(1, 1_000_000);
    if preferences.review_mode != "deep" {
        preferences.review_mode = default_review_mode();
    }
    if preferences.board_theme != "high-contrast" {
        preferences.board_theme = default_board_theme();
    }
    if preferences.sub_board_content_mode != "raw" {
        preferences.sub_board_content_mode = default_sub_board_content_mode();
    }
    preferences
}

pub fn load_from_path(path: &Path) -> Result<AppPreferencesLoadResultDto, String> {
    match fs::read_to_string(path) {
        Ok(contents) => match serde_json::from_str::<AppPreferencesDto>(&contents) {
            Ok(preferences) => Ok(AppPreferencesLoadResultDto {
                preferences: normalize_app_preferences(preferences),
                recovery: None,
            }),
            Err(_) => recover_unreadable(path),
        },
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(AppPreferencesLoadResultDto {
            preferences: default_app_preferences(),
            recovery: None,
        }),
        Err(_) => recover_unreadable(path),
    }
}

pub fn save_to_path(path: &Path, preferences: AppPreferencesDto) -> Result<AppPreferencesDto, String> {
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

fn default_auto_load_cache() -> bool {
    true
}

fn default_auto_save_analysis() -> bool {
    true
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

fn default_sub_board_content_mode() -> String {
    "variation".to_string()
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
            show_ownership: false,
            show_policy: false,
            show_candidates: false,
            candidate_limit: 3,
            auto_load_cache: false,
            auto_save_analysis: false,
            default_max_visits: 200,
            review_mode: "deep".to_string(),
            board_theme: "high-contrast".to_string(),
            sub_board_content_mode: "raw".to_string(),
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
        assert!(!loaded.preferences.show_candidates);
        assert_eq!(loaded.preferences.candidate_limit, 2);
        assert!(loaded.preferences.show_ownership);
        assert!(loaded.preferences.show_policy);
        assert_eq!(loaded.preferences.review_mode, "quick");
        assert_eq!(loaded.preferences.board_theme, "classic");
        assert_eq!(loaded.preferences.sub_board_content_mode, "variation");
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
}
