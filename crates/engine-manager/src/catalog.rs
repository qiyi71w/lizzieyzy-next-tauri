use app_model::{EngineAdapterSettings, EngineProfileDto, EngineStartupPolicyDto, KataGoSettings};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub const DEFAULT_ENGINE_PROFILE_ID: &str = "default";
pub const ENGINE_PROFILES_VERSION: u32 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedEngineProfile {
    pub profile_id: String,
    pub profile: EngineProfileDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineProfileRecord {
    pub id: String,
    pub profile: EngineProfileDto,
    #[serde(default)]
    pub preload: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineProfilesSettings {
    pub version: u32,
    pub selected_profile_id: String,
    pub startup: EngineStartupPolicyDto,
    #[serde(default)]
    pub last_primary_profile_id: Option<String>,
    #[serde(default)]
    pub startup_evaluation: app_model::StartupEvaluationSettingsDto,
    pub profiles: Vec<EngineProfileRecord>,
}

impl EngineProfilesSettings {
    pub fn startup_profile_id(&self) -> Option<&str> {
        match &self.startup {
            EngineStartupPolicyDto::Off => None,
            EngineStartupPolicyDto::Fixed { profile_id } => Some(profile_id),
            EngineStartupPolicyDto::LastPrimary => self.last_primary_profile_id.as_deref(),
        }
    }

    pub fn startup_evaluation_target(&self) -> Result<Option<SavedEngineProfile>, String> {
        if !self.startup_evaluation.enabled {
            return Ok(None);
        }
        let target = self
            .startup_evaluation
            .target_profile_id
            .as_deref()
            .and_then(|id| self.profiles.iter().find(|record| record.id == id))
            .ok_or("Startup evaluation unavailable: select an existing saved target")?;
        Ok(Some(SavedEngineProfile {
            profile_id: target.id.clone(),
            profile: target.profile.clone(),
        }))
    }
}

fn legacy_startup(profile_id: Option<String>) -> EngineStartupPolicyDto {
    profile_id.map_or(EngineStartupPolicyDto::Off, |profile_id| {
        EngineStartupPolicyDto::Fixed { profile_id }
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionOneEngineProfilesSettings {
    version: u32,
    selected_profile_id: String,
    #[serde(default)]
    autoload_profile_id: Option<String>,
    profiles: Vec<EngineProfileRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyEngineProfile {
    name: String,
    engine_path: String,
    model_path: Option<String>,
    config_path: Option<String>,
    working_dir: Option<String>,
    backend: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyEngineProfileSettings {
    profile: LegacyEngineProfile,
    max_visits: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyEngineProfileRecord {
    id: String,
    profile: LegacyEngineProfile,
    max_visits: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyEngineProfilesSettings {
    selected_profile_id: String,
    #[serde(default)]
    autoload_profile_id: Option<String>,
    profiles: Vec<LegacyEngineProfileRecord>,
}

impl LegacyEngineProfile {
    fn migrate(self, max_visits: u32) -> Result<EngineProfileDto, String> {
        if self.backend != "kata_go_analysis" {
            return Err(format!("unsupported legacy engine backend: {}", self.backend));
        }
        Ok(EngineProfileDto {
            name: self.name,
            program: self.engine_path,
            argv: Vec::new(),
            working_dir: self.working_dir,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: self.model_path,
                config_path: self.config_path,
                max_visits,
            }),
        })
    }
}

pub trait EngineProfileCatalog: Send + Sync {
    fn get(&self, profile_id: &str) -> Option<SavedEngineProfile>;
    fn preload_profiles(&self) -> Vec<SavedEngineProfile> {
        Vec::new()
    }

    fn startup_evaluation_target(&self) -> Result<Option<SavedEngineProfile>, String> {
        Ok(None)
    }
    fn autoload_profile_id(&self) -> Option<String> {
        None
    }
}

#[derive(Clone, Default)]
pub struct InMemoryEngineProfileCatalog {
    profiles: Arc<Mutex<BTreeMap<String, SavedEngineProfile>>>,
    autoload_profile_id: Arc<Mutex<Option<String>>>,
    preload_ids: Arc<Mutex<HashSet<String>>>,
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

    pub fn set_preload(&self, profile_id: &str, enabled: bool) {
        let mut ids = self.preload_ids.lock().expect("engine preload catalog lock");
        if enabled {
            ids.insert(profile_id.to_owned());
        } else {
            ids.remove(profile_id);
        }
    }

    pub fn set_autoload_profile_id(&self, profile_id: Option<String>) {
        *self
            .autoload_profile_id
            .lock()
            .expect("engine profile catalog lock") = profile_id;
    }
}

impl EngineProfileCatalog for InMemoryEngineProfileCatalog {
    fn preload_profiles(&self) -> Vec<SavedEngineProfile> {
        let ids = self.preload_ids.lock().expect("engine preload catalog lock");
        let profiles = self.profiles.lock().expect("engine profile catalog lock");
        ids.iter().filter_map(|id| profiles.get(id).cloned()).collect()
    }

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
        version: ENGINE_PROFILES_VERSION,
        selected_profile_id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
        startup: EngineStartupPolicyDto::Off,
        last_primary_profile_id: None,
        startup_evaluation: Default::default(),
        profiles: vec![default_engine_profile_record()],
    }
}

pub fn default_engine_profile_record() -> EngineProfileRecord {
    EngineProfileRecord {
        id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
        preload: false,
        profile: EngineProfileDto {
            name: "Local KataGo".to_string(),
            program: String::new(),
            argv: Vec::new(),
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: None,
                config_path: None,
                max_visits: 800,
            }),
        },
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
    let value: serde_json::Value = serde_json::from_str(contents).map_err(|err| err.to_string())?;
    let settings = if let Some(version) = value.get("version") {
        match version.as_u64() {
            Some(1) => {
                let old: VersionOneEngineProfilesSettings =
                    serde_json::from_value(value).map_err(|err| err.to_string())?;
                debug_assert_eq!(old.version, 1);
                EngineProfilesSettings {
                    version: ENGINE_PROFILES_VERSION,
                    selected_profile_id: old.selected_profile_id,
                    startup: legacy_startup(old.autoload_profile_id),
                    last_primary_profile_id: None,
                    startup_evaluation: Default::default(),
                    profiles: old.profiles,
                }
            }
            Some(2) => {
                serde_json::from_value::<EngineProfilesSettings>(value).map_err(|err| err.to_string())?
            }
            _ => return Err(format!("unsupported engine profiles version: {version}")),
        }
    } else if value.get("profiles").is_some() {
        let legacy =
            serde_json::from_value::<LegacyEngineProfilesSettings>(value).map_err(|err| err.to_string())?;
        EngineProfilesSettings {
            version: ENGINE_PROFILES_VERSION,
            selected_profile_id: legacy.selected_profile_id,
            startup: legacy_startup(legacy.autoload_profile_id),
            last_primary_profile_id: None,
            startup_evaluation: Default::default(),
            profiles: legacy
                .profiles
                .into_iter()
                .map(|record| {
                    Ok(EngineProfileRecord {
                        id: record.id,
                        preload: false,
                        profile: record.profile.migrate(record.max_visits)?,
                    })
                })
                .collect::<Result<_, String>>()?,
        }
    } else {
        let legacy =
            serde_json::from_value::<LegacyEngineProfileSettings>(value).map_err(|err| err.to_string())?;
        EngineProfilesSettings {
            version: ENGINE_PROFILES_VERSION,
            selected_profile_id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
            startup: EngineStartupPolicyDto::Off,
            last_primary_profile_id: None,
            startup_evaluation: Default::default(),
            profiles: vec![EngineProfileRecord {
                id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
                preload: false,
                profile: legacy.profile.migrate(legacy.max_visits)?,
            }],
        }
    };
    normalize_engine_profiles(settings)
}

pub fn save_engine_profiles(
    path: &Path,
    settings: EngineProfilesSettings,
) -> Result<EngineProfilesSettings, String> {
    let settings = normalize_engine_profiles(settings)?;
    replace_json_file(path, &settings)?;
    Ok(settings)
}

/// Save only a successfully established primary, after a confirmed graceful departure.
/// A session that never established a primary leaves the previous durable identity intact.
pub fn persist_last_primary(
    path: &Path,
    mut current: EngineProfilesSettings,
    primary: Option<String>,
) -> Result<EngineProfilesSettings, String> {
    let Some(primary) = primary else {
        return Ok(current);
    };
    if current.last_primary_profile_id.as_ref() == Some(&primary) {
        return Ok(current);
    }
    current.last_primary_profile_id = Some(primary);
    save_engine_profiles(path, current)
}

pub fn prepare_engine_profiles_save(
    current: &EngineProfilesSettings,
    settings: EngineProfilesSettings,
) -> Result<EngineProfilesSettings, String> {
    let mut settings = normalize_engine_profiles(settings)?;
    // This identity belongs to graceful exit, never a stale settings form.
    settings.last_primary_profile_id = current.last_primary_profile_id.clone();
    let mut destination = 0;
    for existing in &current.profiles {
        if let Some(offset) = settings.profiles[destination..]
            .iter()
            .position(|record| record.id == existing.id)
        {
            settings.profiles[destination..=destination + offset].rotate_right(1);
            destination += 1;
        }
    }
    Ok(settings)
}

pub fn reorder_engine_profiles(
    path: &Path,
    current: EngineProfilesSettings,
    request: &app_model::EngineProfileOrderRequestDto,
) -> Result<EngineProfilesSettings, String> {
    reorder_engine_profiles_with(path, current, request, replace_json_file)
}

fn reorder_engine_profiles_with(
    path: &Path,
    mut current: EngineProfilesSettings,
    request: &app_model::EngineProfileOrderRequestDto,
    persist: impl FnOnce(&Path, &EngineProfilesSettings) -> Result<(), String>,
) -> Result<EngineProfilesSettings, String> {
    if !current
        .profiles
        .iter()
        .map(|record| &record.id)
        .eq(request.expected_profile_ids.iter())
    {
        return Err("engine profile catalog order is stale; reload profiles before reordering".into());
    }
    if request.profile_ids.len() != current.profiles.len() {
        return Err("engine profile order must contain the complete catalog".into());
    }
    let mut positions = BTreeMap::new();
    for (index, id) in request.profile_ids.iter().enumerate() {
        if positions.insert(id.as_str(), index).is_some() {
            return Err(format!("duplicate engine profile id: {id}"));
        }
    }
    if current
        .profiles
        .iter()
        .any(|record| !positions.contains_key(record.id.as_str()))
    {
        return Err("engine profile order contains unknown or missing profile IDs".into());
    }
    if request.profile_ids == request.expected_profile_ids {
        return Ok(current);
    }
    current
        .profiles
        .sort_unstable_by_key(|record| positions[record.id.as_str()]);
    persist(path, &current)?;
    Ok(current)
}

pub fn normalize_engine_profiles(settings: EngineProfilesSettings) -> Result<EngineProfilesSettings, String> {
    if settings.version != ENGINE_PROFILES_VERSION {
        return Err(format!(
            "unsupported engine profiles version: {}",
            settings.version
        ));
    }
    if settings.profiles.is_empty() {
        return Err("engine profiles must contain at least one record".to_string());
    }
    let mut seen_ids = HashSet::new();
    for record in &settings.profiles {
        if record.id.trim().is_empty() || record.id.contains('\0') {
            return Err("engine profile id is required and must not contain NUL".to_string());
        }
        if !seen_ids.insert(record.id.as_str()) {
            return Err(format!("duplicate engine profile id: {}", record.id));
        }
        crate::validate_engine_profile(&record.profile)?;
    }
    if !seen_ids.contains(settings.selected_profile_id.as_str()) {
        return Err("selected engine profile does not exist".to_string());
    }
    if let EngineStartupPolicyDto::Fixed { profile_id } = &settings.startup {
        if profile_id.trim().is_empty() || profile_id.contains('\0') {
            return Err("fixed startup profile ID is invalid".to_string());
        }
    }
    if settings
        .last_primary_profile_id
        .as_deref()
        .is_some_and(|id| id.trim().is_empty() || id.contains('\0'))
    {
        return Err("last primary profile ID is invalid".to_string());
    }
    if settings
        .startup_evaluation
        .target_profile_id
        .as_deref()
        .is_some_and(|id| id.trim().is_empty() || id.contains('\0'))
    {
        return Err("startup evaluation target profile ID is invalid".to_string());
    }
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
            version: ENGINE_PROFILES_VERSION,
            selected_profile_id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
            startup: EngineStartupPolicyDto::Fixed {
                profile_id: DEFAULT_ENGINE_PROFILE_ID.to_string(),
            },
            last_primary_profile_id: None,
            startup_evaluation: Default::default(),
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
                    startup: EngineStartupPolicyDto::Off,
                    ..sample_settings()
                },
            );
            let mut permissions = fs::metadata(&dir).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&dir, permissions).unwrap();
            assert!(err.is_err());
        }

        let loaded = load_engine_profiles(&path).unwrap();
        assert_eq!(loaded.startup_profile_id(), Some(DEFAULT_ENGINE_PROFILE_ID));
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

    #[test]
    fn startup_evaluation_atomic_failure_preserves_authorization_bytes() {
        let (dir, path) = temp_catalog();
        let original = save_engine_profiles(&path, sample_settings()).unwrap();
        let before = fs::read(&path).unwrap();
        let mut next = original.clone();
        next.startup_evaluation = app_model::StartupEvaluationSettingsDto {
            enabled: true,
            target_profile_id: Some("default".into()),
        };
        let json = serde_json::to_string_pretty(&normalize_engine_profiles(next).unwrap()).unwrap();
        assert!(
            atomic_replace_file_with(&path, &json, |_, _| Err(io::Error::other("rename denied"))).is_err()
        );
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(load_engine_profiles(&path).unwrap(), original);
        assert!(!tmp_path(&path).exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reordered_catalog_replace_failure_retains_original_order_and_identities() {
        let (dir, path) = temp_catalog();
        let mut current = sample_settings();
        let mut second = default_engine_profile_record();
        second.id = "second".into();
        second.profile.name = "Second".into();
        current.profiles.push(second);
        save_engine_profiles(&path, current.clone()).unwrap();
        let before = fs::read(&path).unwrap();
        let request = app_model::EngineProfileOrderRequestDto {
            expected_profile_ids: vec!["default".into(), "second".into()],
            profile_ids: vec!["second".into(), "default".into()],
        };
        let error = reorder_engine_profiles_with(&path, current.clone(), &request, |path, next| {
            let json = serde_json::to_string_pretty(next).unwrap();
            atomic_replace_file_with(path, &json, |_, _| Err(io::Error::other("rename boom")))
        })
        .unwrap_err();
        assert!(error.contains("replace"));
        assert_eq!(fs::read(&path).unwrap(), before);
        assert_eq!(load_engine_profiles(&path).unwrap(), current);
        assert!(!tmp_path(&path).exists());
        fs::remove_dir_all(dir).unwrap();
    }
}
