use app_model::{AnalysisJobModeDto, NodePath, SelectedNodeSnapshotDto, WorkspaceSharesDto};
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
        preferences.workspace_shares = latest.workspace_shares;
        preferences.window_geometry = latest.window_geometry;
        preferences.workspace_visibility = latest.workspace_visibility;
        preferences.main_window_always_on_top = latest.main_window_always_on_top;
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

    pub fn update_workspace_shares(
        &self,
        path: &Path,
        shares: Option<WorkspaceSharesDto>,
    ) -> Result<Option<WorkspaceSharesDto>, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed.as_ref()
            .ok_or("Preferences must finish loading before updating workspace shares.")?
            .preferences.clone();
        preferences.workspace_shares = shares;
        let saved = app_preferences::save_to_path(path, preferences)?;
        let shares = saved.workspace_shares;
        *committed = Some(AppPreferencesLoadResultDto { preferences: saved, recovery: None });
        Ok(shares)
    }

    pub fn update_window_geometry(
        &self,
        path: &Path,
        geometry: app_model::WindowGeometryDto,
    ) -> Result<(), String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed.as_ref()
            .ok_or("Preferences must finish loading before updating window geometry.")?
            .preferences.clone();
        preferences.window_geometry = Some(geometry);
        let saved = app_preferences::save_to_path(path, preferences)?;
        committed.as_mut().unwrap().preferences = saved;
        Ok(())
    }

    pub fn pin_intent(&self) -> Result<bool, String> {
        let committed = self.0.lock().expect("preferences transaction");
        Ok(committed.as_ref().ok_or("Preferences must finish loading before pinning.")?
            .preferences.main_window_always_on_top)
    }

    pub fn save_pin(&self, path: &Path, value: bool) -> Result<(), String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed.as_ref()
            .ok_or("Preferences must finish loading before pinning.")?.preferences.clone();
        preferences.main_window_always_on_top = value;
        let saved = app_preferences::save_to_path(path, preferences)?;
        *committed = Some(AppPreferencesLoadResultDto { preferences: saved, recovery: None });
        Ok(())
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
    fn geometry_autosave_preserves_unconsumed_preference_recovery() {
        let directory = std::env::temp_dir().join(format!("geometry-recovery-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        std::fs::write(&path, b"not json").unwrap();
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()), ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let initial = state.load(&path, &manager).unwrap();
        assert!(initial.recovery.is_some());
        state.update_window_geometry(&path, app_model::WindowGeometryDto {
            x: Some(40.0), y: Some(50.0), width: 1100.0, height: 720.0, scale_factor: 1.0, maximized: false,
        }).unwrap();
        assert_eq!(state.load(&path, &manager).unwrap().recovery, initial.recovery);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn workspace_updates_merge_with_other_owners_and_fail_atomically() {
        let directory = std::env::temp_dir().join(format!("workspace-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()), ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let mut stale = state.load(&path, &manager).unwrap().preferences;
        let shares = Some(WorkspaceSharesDto { left: 0.2, right: 0.3 });
        let before = manager.snapshot();
        state.update_workspace_shares(&path, shares).unwrap();
        let geometry = app_model::WindowGeometryDto {
            x: Some(-1200.0), y: Some(50.0), width: 1100.0, height: 720.0,
            scale_factor: 1.25, maximized: true,
        };
        state.update_window_geometry(&path, geometry).unwrap();
        assert_eq!(manager.snapshot(), before);
        stale.board_theme = "high-contrast".into();
        stale.continuous_budget.continuous_time_limit_seconds = 123;
        let saved = state.save(&path, &manager, stale).unwrap();
        assert_eq!(saved.workspace_shares, shares);
        assert_eq!(saved.window_geometry, Some(geometry));
        state.update_recent_history(&path, Some("/games/new.sgf")).unwrap();
        let before_reset = state.load(&path, &manager).unwrap().preferences;
        let durable = std::fs::read(&path).unwrap();
        assert!(state.update_workspace_shares(&directory, None).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        assert_eq!(state.load(&path, &manager).unwrap().preferences, before_reset);
        assert!(state.update_workspace_shares(&path, Some(WorkspaceSharesDto { left: 0.6, right: 0.4 })).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        let changed = app_model::WindowGeometryDto { x: Some(40.0), maximized: false, ..geometry };
        assert!(state.update_window_geometry(&directory, changed).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        assert_eq!(state.load(&path, &manager).unwrap().preferences, before_reset);
        state.update_workspace_shares(&path, None).unwrap();
        let restarted = PreferencesState::default().load(&path, &manager).unwrap().preferences;
        assert_eq!(restarted, AppPreferencesDto { workspace_shares: None, ..before_reset });
        state.update_window_geometry(&path, changed).unwrap();
        let retried = PreferencesState::default().load(&path, &manager).unwrap().preferences;
        assert_eq!(retried, AppPreferencesDto { window_geometry: Some(changed), ..restarted });
        std::fs::remove_dir_all(directory).unwrap();
    }

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
