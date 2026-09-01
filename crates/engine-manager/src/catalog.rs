use app_model::EngineProfileDto;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedEngineProfile {
    pub profile_id: String,
    pub profile: EngineProfileDto,
}

pub trait EngineProfileCatalog: Send + Sync {
    fn get(&self, profile_id: &str) -> Option<SavedEngineProfile>;
}

#[derive(Clone, Default)]
pub struct InMemoryEngineProfileCatalog {
    profiles: Arc<Mutex<BTreeMap<String, SavedEngineProfile>>>,
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
}

impl EngineProfileCatalog for InMemoryEngineProfileCatalog {
    fn get(&self, profile_id: &str) -> Option<SavedEngineProfile> {
        self.profiles
            .lock()
            .expect("engine profile catalog lock")
            .get(profile_id)
            .cloned()
    }
}
