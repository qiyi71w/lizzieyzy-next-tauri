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
        mut preferences: AppPreferencesDto,
    ) -> Result<AppPreferencesDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let latest = &committed.as_ref()
            .ok_or("Preferences must finish loading before saving.")?.preferences;
        preferences.recent_game_paths = latest.recent_game_paths.clone();
        preferences.workspace_visibility = latest.workspace_visibility;
        let saved = app_preferences::save_to_path(path, preferences)?;
        manager.set_continuous_preferences(saved.continuous_analysis_enabled, saved.continuous_budget)?;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved.clone(),
            recovery: None,
        });
        Ok(saved)
    }

    pub fn update_workspace_visibility(
        &self,
        path: &Path,
        left: Option<bool>,
        right: Option<bool>,
    ) -> Result<app_model::WorkspaceVisibilityDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed.as_ref()
            .ok_or("Preferences must finish loading before changing rail visibility.")?
            .preferences.clone();
        if let Some(value) = left { preferences.workspace_visibility.left = value; }
        if let Some(value) = right { preferences.workspace_visibility.right = value; }
        let saved = app_preferences::save_to_path(path, preferences)?;
        let visibility = saved.workspace_visibility;
        *committed = Some(AppPreferencesLoadResultDto { preferences: saved, recovery: None });
        Ok(visibility)
    }

    pub fn update_recent_history(&self, path: &Path, opened_path: Option<&str>) -> Result<Vec<String>, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed.as_ref()
            .ok_or("Preferences must finish loading before updating recent history.")?
            .preferences.clone();
        preferences.recent_game_paths = match opened_path {
            Some(opened) => app_preferences::recent_game_paths(&preferences.recent_game_paths, opened),
            None => Vec::new(),
        };
        let saved = app_preferences::save_to_path(path, preferences)?;
        let paths = saved.recent_game_paths.clone();
        *committed = Some(AppPreferencesLoadResultDto { preferences: saved, recovery: None });
        Ok(paths)
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
    board_width: u8,
    board_height: u8,
    komi: f32,
    rules: String,
) -> Result<SelectedNodeJobRequest, String> {
    let query = analysis_query_from_position(
        board_width,
        board_height,
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
        board_width,
        board_height,
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
    fn rail_visibility_preserves_other_owners_and_failed_writes_do_not_commit() {
        let directory = std::env::temp_dir().join(format!("rail-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()), ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let mut stale = state.load(&path, &manager).unwrap().preferences;
        state.update_workspace_visibility(&path, Some(false), None).unwrap();
        stale.show_coordinates = false;
        state.save(&path, &manager, stale).unwrap();
        state.update_recent_history(&path, Some("/games/test.sgf")).unwrap();
        state.update_workspace_visibility(&path, None, Some(false)).unwrap();
        assert!(state.update_workspace_visibility(&directory, Some(true), None).is_err());
        let loaded = state.load(&path, &manager).unwrap().preferences;
        assert_eq!(loaded.workspace_visibility, app_model::WorkspaceVisibilityDto { left: false, right: false });
        assert!(!loaded.show_coordinates);
        assert_eq!(loaded.recent_game_paths, ["/games/test.sgf"]);
        let restarted = PreferencesState::default().load(&path, &manager).unwrap().preferences;
        assert_eq!(restarted, loaded);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn recent_history_failure_clear_restart_and_stale_preferences_are_atomic() {
        let directory = std::env::temp_dir().join(format!("recent-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()), ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let mut before_open = state.load(&path, &manager).unwrap().preferences;
        before_open.restore_last_session = true;
        state.save(&path, &manager, before_open.clone()).unwrap();
        state.update_recent_history(&path, Some("/games/old.sgf")).unwrap();
        let durable = std::fs::read(&path).unwrap();
        assert!(state.update_recent_history(&directory, Some("/games/new.gib")).is_err());
        assert!(state.update_recent_history(&directory, None).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        assert_eq!(state.load(&path, &manager).unwrap().preferences.recent_game_paths, ["/games/old.sgf"]);
        let saved = state.save(&path, &manager, before_open).unwrap();
        assert_eq!(saved.recent_game_paths, ["/games/old.sgf"]);
        state.update_recent_history(&path, Some("/games/new.gib")).unwrap();
        let reloaded = PreferencesState::default().load(&path, &manager).unwrap().preferences;
        assert_eq!(reloaded.recent_game_paths, ["/games/new.gib", "/games/old.sgf"]);
        state.update_recent_history(&path, None).unwrap();
        let cleared = app_preferences::load_from_path(&path).unwrap().preferences;
        assert_eq!(cleared, AppPreferencesDto { recent_game_paths: Vec::new(), ..reloaded });
        std::fs::remove_dir_all(directory).unwrap();
    }

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
