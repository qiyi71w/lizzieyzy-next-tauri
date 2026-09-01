use app_model::{EngineBackend, EngineProfileDto};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const DEFAULT_ENGINE_PROFILE_ID: &str = "default";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedEngineProfile {
    pub profile_id: String,
    pub profile: EngineProfileDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineProfileRecord {
    pub id: String,
    pub profile: EngineProfileDto,
    pub max_visits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineProfilesSettings {
    pub selected_profile_id: String,
    #[serde(default)]
    pub autoload_profile_id: Option<String>,
    pub profiles: Vec<EngineProfileRecord>,
}

#[derive(Debug, Clone, Deserialize)]
struct LegacyEngineProfileSettings {
    profile: EngineProfileDto,
    max_visits: u32,
}

pub trait EngineProfileCatalog: Send + Sync {
    fn get(&self, profile_id: &str) -> Option<SavedEngineProfile>;
    fn autoload_profile_id(&self) -> Option<String> {
        None
    }
}

#[derive(Clone, Default)]
pub struct InMemoryEngineProfileCatalog {
    profiles: Arc<Mutex<BTreeMap<String, SavedEngineProfile>>>,
    autoload_profile_id: Arc<Mutex<Option<String>>>,
}

impl InMemoryEngineProfileCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&self, profile: SavedEngineProfile) {
        self.profiles
            .lock()
            .expect("engine profile catalog lock")
            .insert(profile.profile_id.clone(), profile);
    }

    pub fn set_autoload_profile_id(&self, profile_id: Option<String>) {
        *self
            .autoload_profile_id
            .lock()
            .expect("engine profile catalog lock") = profile_id;
    }
}

impl EngineProfileCatalog for InMemoryEngineProfileCatalog {
    fn get(&self, profile_id: &str) -> Option<SavedEngineProfile> {
        self.profiles
            .lock()
            .expect("engine profile catalog lock")
            .get(profile_id)
            .cloned()
    }

    fn autoload_profile_id(&self) -> Option<String> {
        self.autoload_profile_id
            .lock()
            .expect("engine profile catalog lock")
            .clone()
    }
}

pub fn default_engine_profiles_settings() -> EngineProfilesSettings {
    EngineProfilesSettings {
        selected_profile_id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
        autoload_profile_id: None,
        profiles: vec![default_engine_profile_record()],
    }
}

pub fn default_engine_profile_record() -> EngineProfileRecord {
    EngineProfileRecord {
        id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
        profile: EngineProfileDto {
            name: "Local KataGo".to_string(),
            engine_path: String::new(),
            model_path: None,
            config_path: None,
            working_dir: None,
            backend: EngineBackend::KataGoAnalysis,
        },
        max_visits: 800,
    }
}

pub fn load_engine_profiles(path: &Path) -> Result<EngineProfilesSettings, String> {
    match fs::read_to_string(path) {
        Ok(contents) => parse_engine_profiles(&contents)
            .map_err(|err| format!("failed to parse {}: {err}", path.display())),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(default_engine_profiles_settings()),
        Err(err) => Err(format!("failed to read {}: {err}", path.display())),
    }
}

pub fn parse_engine_profiles(contents: &str) -> Result<EngineProfilesSettings, String> {
    serde_json::from_str::<EngineProfilesSettings>(contents)
        .or_else(|_| {
            serde_json::from_str::<LegacyEngineProfileSettings>(contents).map(|settings| {
                EngineProfilesSettings {
                    selected_profile_id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
                    autoload_profile_id: None,
                    profiles: vec![EngineProfileRecord {
                        id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
                        profile: settings.profile,
                        max_visits: settings.max_visits,
                    }],
                }
            })
        })
        .map_err(|err| err.to_string())
        .and_then(normalize_engine_profiles)
}

pub fn save_engine_profiles(
    path: &Path,
    settings: EngineProfilesSettings,
) -> Result<EngineProfilesSettings, String> {
    let settings = normalize_engine_profiles(settings)?;
    replace_json_file(path, &settings)?;
    Ok(settings)
}

pub fn normalize_engine_profiles(
    mut settings: EngineProfilesSettings,
) -> Result<EngineProfilesSettings, String> {
    if settings.profiles.is_empty() {
        settings.profiles.push(default_engine_profile_record());
    }

    let mut seen_ids = HashSet::new();
    let mut normalized_profiles = Vec::new();
    for mut record in settings.profiles {
        record.id = record.id.trim().to_string();
        if record.id.is_empty() {
            return Err("engine profile id is required".to_string());
        }
        if !seen_ids.insert(record.id.clone()) {
            return Err(format!("duplicate engine profile id: {}", record.id));
        }
        if record.max_visits == 0 {
            return Err("max_visits must be greater than 0".to_string());
        }
        if record.profile.name.trim().is_empty() {
            return Err("engine profile name is required".to_string());
        }
        normalized_profiles.push(record);
    }

    if !seen_ids.contains(DEFAULT_ENGINE_PROFILE_ID) {
        normalized_profiles.insert(0, default_engine_profile_record());
        seen_ids.insert(DEFAULT_ENGINE_PROFILE_ID.to_string());
    }

    settings.selected_profile_id = settings.selected_profile_id.trim().to_string();
    if !seen_ids.contains(&settings.selected_profile_id) {
        settings.selected_profile_id = DEFAULT_ENGINE_PROFILE_ID.to_string();
    }

    settings.autoload_profile_id = settings.autoload_profile_id.and_then(|profile_id| {
        let profile_id = profile_id.trim().to_string();
        if profile_id.is_empty() || !seen_ids.contains(&profile_id) {
            None
        } else {
            Some(profile_id)
        }
    });
    settings.profiles = normalized_profiles;
    Ok(settings)
}

pub fn replace_json_file<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
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

#[cfg(test)]
mod persist_failure_tests {
    use super::*;
    use serde::ser::{Error as SerError, Serialize, Serializer};

    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S: Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
            Err(S::Error::custom("serialization boom"))
        }
    }

    fn temp_catalog() -> (PathBuf, PathBuf) {
        let unique = format!(
            "{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let dir = std::env::temp_dir()
            .join("lizzieyzy-engine-profiles-persist-failure")
            .join(unique);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("catalog.json");
        (dir, path)
    }

    fn sample_settings() -> EngineProfilesSettings {
        EngineProfilesSettings {
            selected_profile_id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
            autoload_profile_id: Some(DEFAULT_ENGINE_PROFILE_ID.to_string()),
            profiles: vec![default_engine_profile_record()],
        }
    }

    #[test]
    fn serialize_failure_keeps_previous_durable_catalog() {
        let (dir, path) = temp_catalog();
        save_engine_profiles(&path, sample_settings()).unwrap();
        let before = fs::read_to_string(&path).unwrap();

        let err = replace_json_file(&path, &FailingSerialize).unwrap_err();
        assert!(err.contains("serialize"));
        assert_eq!(fs::read_to_string(&path).unwrap(), before);
        assert!(!tmp_path(&path).exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn write_failure_keeps_previous_durable_catalog() {
        let (dir, path) = temp_catalog();
        save_engine_profiles(&path, sample_settings()).unwrap();
        let before = fs::read_to_string(&path).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&dir).unwrap().permissions();
            permissions.set_mode(0o555);
            fs::set_permissions(&dir, permissions).unwrap();
            let err = save_engine_profiles(
                &path,
                EngineProfilesSettings {
                    autoload_profile_id: None,
                    ..sample_settings()
                },
            );
            let mut permissions = fs::metadata(&dir).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&dir, permissions).unwrap();
            assert!(err.is_err());
        }

        let loaded = load_engine_profiles(&path).unwrap();
        assert_eq!(
            loaded.autoload_profile_id.as_deref(),
            Some(DEFAULT_ENGINE_PROFILE_ID)
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), before);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn atomic_replace_failure_keeps_previous_durable_catalog() {
        let (dir, path) = temp_catalog();
        save_engine_profiles(&path, sample_settings()).unwrap();
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
