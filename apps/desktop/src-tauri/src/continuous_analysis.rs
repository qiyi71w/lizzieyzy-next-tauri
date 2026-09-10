use app_model::{AnalysisJobModeDto, NodePath, SelectedNodeSnapshotDto};
use app_preferences::{AppPreferencesDto, AppPreferencesLoadResultDto};
use engine_manager::{ContinuousPrimaryAction, ForegroundEngineManager, SelectedNodeJobRequest};
use katago_protocol::{analysis_query_from_position, AnalysisQueryOptions};
use std::{path::Path, sync::Mutex};
use tauri::{AppHandle, State};

// Serializes durable writes with primary actions; failed writes never reach the manager.
#[derive(Default)]
pub struct PreferencesState(Mutex<Option<AppPreferencesLoadResultDto>>);

impl PreferencesState {
    pub fn load(
        &self,
        path: &Path,
        manager: &ForegroundEngineManager,
    ) -> Result<AppPreferencesLoadResultDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        if let Some(loaded) = committed.as_ref() {
            return Ok(loaded.clone());
        }
        let loaded = app_preferences::load_from_path(path)?;
        manager.set_continuous_preferences(
            loaded.preferences.continuous_analysis_enabled,
            loaded.preferences.continuous_budget,
        )?;
        *committed = Some(loaded.clone());
        Ok(loaded)
    }

    pub fn save(
        &self,
        path: &Path,
        manager: &ForegroundEngineManager,
        preferences: AppPreferencesDto,
    ) -> Result<AppPreferencesDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let saved = app_preferences::save_to_path(path, preferences)?;
        manager.set_continuous_preferences(saved.continuous_analysis_enabled, saved.continuous_budget)?;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved.clone(),
            recovery: None,
        });
        Ok(saved)
    }

    pub fn primary(
        &self,
        path: &Path,
        manager: &ForegroundEngineManager,
    ) -> Result<AppPreferencesDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let loaded = committed
            .as_ref()
            .ok_or("Preferences must finish loading before analysis actions.")?;
        let mut preferences = loaded.preferences.clone();
        match manager
            .continuous_primary_action()
            .map_err(|failure| failure.message)?
        {
            ContinuousPrimaryAction::Resume => {
                manager.resume_continuous().map_err(|failure| failure.message)?;
            }
            action => {
                let start = matches!(action, ContinuousPrimaryAction::Start);
                let finite = manager
                    .snapshot()
                    .selected_node_job
                    .is_some_and(|job| job.mode == AnalysisJobModeDto::Finite);
                preferences.continuous_analysis_enabled = start;
                preferences = app_preferences::save_to_path(path, preferences)?;
                manager.set_continuous_preferences(start, preferences.continuous_budget)?;
                if start && !finite {
                    manager.authorize_continuous_start();
                }
                *committed = Some(AppPreferencesLoadResultDto {
                    preferences: preferences.clone(),
                    recovery: None,
                });
            }
        }
        Ok(preferences)
    }
}

#[tauri::command]
pub fn foreground_engine_continuous_action(
    app: AppHandle,
    preferences: State<PreferencesState>,
    manager: State<ForegroundEngineManager>,
    current_game: State<crate::current_game_state::CurrentGameState>,
) -> Result<AppPreferencesDto, String> {
    let path = super::app_preferences_path(&app)?;
    current_game.run_analysis_action(|| preferences.primary(&path, &manager))
}

pub fn position_request(
    generation: u64,
    node_path: NodePath,
    snapshot: SelectedNodeSnapshotDto,
    board_size: u8,
    komi: f32,
    rules: String,
) -> Result<SelectedNodeJobRequest, String> {
    let query = analysis_query_from_position(
        board_size,
        komi,
        &snapshot.position.stones,
        snapshot.position.to_play,
        AnalysisQueryOptions {
            id: "pending".to_string(),
            rules,
            turn: snapshot.position.move_number,
            max_visits: None,
            include_ownership: Some(true),
            include_policy: Some(true),
        },
    )
    .map_err(|error| error.to_string())?;
    Ok(SelectedNodeJobRequest {
        run_id: String::new(),
        mode: AnalysisJobModeDto::Continuous,
        generation,
        node_path,
        query,
        board_size,
        position_empty: snapshot.position.stones.is_empty(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::{ContinuousAnalysisPhaseDto, ForegroundEngineLifecycleDto};
    use engine_manager::{ForegroundEngineConfig, InMemoryEngineProfileCatalog};
    use std::sync::Arc;

    #[test]
    fn durable_primary_write_failure_preserves_intent_and_restart_choice() {
        let directory = std::env::temp_dir().join(format!("continuous-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let preferences = PreferencesState::default();
        assert_eq!(manager.snapshot().continuous.enabled, None);
        assert!(
            preferences
                .load(&path, &manager)
                .unwrap()
                .preferences
                .continuous_analysis_enabled
        );
        assert_eq!(
            manager.snapshot().continuous.phase,
            ContinuousAnalysisPhaseDto::Waiting
        );
        // Replacing a directory as a file is a deterministic persistence failure.
        assert!(preferences.primary(&directory, &manager).is_err());
        assert_eq!(manager.snapshot().continuous.enabled, Some(true));
        assert!(
            !preferences
                .primary(&path, &manager)
                .unwrap()
                .continuous_analysis_enabled
        );
        assert_eq!(
            manager.snapshot().continuous.phase,
            ContinuousAnalysisPhaseDto::Off
        );
        assert!(preferences.primary(&directory, &manager).is_err());
        assert_eq!(manager.snapshot().continuous.enabled, Some(false));

        let restarted = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let reloaded = PreferencesState::default().load(&path, &restarted).unwrap();
        assert!(!reloaded.preferences.continuous_analysis_enabled);
        assert_eq!(
            restarted.snapshot().continuous.phase,
            ContinuousAnalysisPhaseDto::Off
        );
        assert!(matches!(
            restarted.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::NoEngine { .. }
        ));
        assert!(
            preferences
                .primary(&path, &manager)
                .unwrap()
                .continuous_analysis_enabled
        );
        assert!(matches!(
            manager.snapshot().lifecycle,
            ForegroundEngineLifecycleDto::NoEngine { .. }
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
