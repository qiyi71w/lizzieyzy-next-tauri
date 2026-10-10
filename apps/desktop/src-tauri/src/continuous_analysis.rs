use app_model::{
    AnalysisJobModeDto, NodePath, ReadboardSyncPreferencesDto, SelectedNodeSnapshotDto, WorkspaceSharesDto,
};
use app_preferences::{AppPreferencesDto, AppPreferencesLoadResultDto};
use engine_manager::{ContinuousPrimaryAction, ForegroundEngineManager, SelectedNodeJobRequest};
use katago_protocol::{analysis_query_from_position, AnalysisQueryOptions};
use std::{path::Path, sync::Mutex};
use tauri::{AppHandle, State};

// Serializes durable writes with primary actions; failed writes never reach the manager.
#[derive(Default)]
pub struct PreferencesState(
    Mutex<Option<AppPreferencesLoadResultDto>>,
    provider_core::network::NetworkState,
);

impl PreferencesState {
    pub fn network(&self) -> &provider_core::network::NetworkState {
        &self.1
    }

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
        self.network()
            .commit(loaded.preferences.network.clone(), || Ok(()))?;
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
        let latest = &committed
            .as_ref()
            .ok_or("Preferences must finish loading before saving.")?
            .preferences;
        preferences.recent_game_paths = latest.recent_game_paths.clone();
        preferences.recent_image_export_directory = latest.recent_image_export_directory.clone();
        preferences.workspace_shares = latest.workspace_shares;
        preferences.window_geometry = latest.window_geometry;
        preferences.workspace_visibility = latest.workspace_visibility;
        preferences.main_window_always_on_top = latest.main_window_always_on_top;
        preferences.match_defaults = latest.match_defaults.clone();
        preferences.network = latest.network.clone();
        preferences.yike_locator = latest.yike_locator.clone();
        preferences.yike_sync = latest.yike_sync.clone();
        preferences.readboard_sync = latest.readboard_sync;
        preferences.tencent_history = latest.tencent_history.clone();
        preferences.readboard_executable_path = latest.readboard_executable_path.clone();
        preferences.fox_kifu = latest.fox_kifu.clone();
        manager.commit_continuous_preferences(
            preferences.continuous_analysis_enabled,
            preferences.continuous_budget,
            || {
                let saved = app_preferences::save_to_path(path, preferences)?;
                *committed = Some(AppPreferencesLoadResultDto {
                    preferences: saved.clone(),
                    recovery: None,
                });
                Ok(saved)
            },
        )
    }

    pub fn save_network(
        &self,
        path: &Path,
        settings: app_model::NetworkSettingsDto,
    ) -> Result<app_model::NetworkSnapshotDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before changing network settings.")?
            .preferences
            .clone();
        preferences.network = settings.clone();
        self.network().commit(settings, || {
            let saved = app_preferences::save_to_path(path, preferences)?;
            *committed = Some(AppPreferencesLoadResultDto {
                preferences: saved,
                recovery: None,
            });
            Ok(())
        })
    }

    pub fn yike_locator(&self) -> Result<Option<String>, String> {
        let committed = self.0.lock().expect("preferences transaction");
        let locator = &committed
            .as_ref()
            .ok_or("Preferences must finish loading before reading the Yike locator.")?
            .preferences
            .yike_locator;
        Ok(locator
            .as_deref()
            .and_then(|value| provider_yike::canonical_yike_locator(value).ok()))
    }

    pub fn save_yike_locator(&self, path: &Path, locator: Option<String>) -> Result<Option<String>, String> {
        let locator = locator
            .as_deref()
            .map(provider_yike::canonical_yike_locator)
            .transpose()
            .map_err(|_| {
                "Enter a supported public Yike locator without credentials or private query parameters."
                    .to_string()
            })?;
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before saving the Yike locator.")?
            .preferences
            .clone();
        preferences.yike_locator = locator.clone();
        let saved = app_preferences::save_to_path(path, preferences)?;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(locator)
    }

    pub fn yike_sync_preferences(&self) -> Result<app_model::YikeSyncPreferencesDto, String> {
        let committed = self.0.lock().expect("preferences transaction");
        let mut value = committed
            .as_ref()
            .ok_or("Preferences must finish loading before synchronization.")?
            .preferences
            .yike_sync
            .clone();
        value.locator = value
            .locator
            .as_deref()
            .and_then(|locator| provider_yike::canonical_yike_locator(locator).ok());
        Ok(value)
    }

    pub fn save_yike_sync_preferences(
        &self,
        path: &Path,
        mut value: app_model::YikeSyncPreferencesDto,
    ) -> Result<app_model::YikeSyncPreferencesDto, String> {
        if value.interval_seconds == 0 {
            return Err("The sync interval must be a positive whole number of seconds.".into());
        }
        value.locator = value
            .locator
            .as_deref()
            .map(provider_yike::canonical_yike_locator)
            .transpose()
            .map_err(|error| error.message)?;
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before synchronization.")?
            .preferences
            .clone();
        preferences.yike_sync = value.clone();
        let saved = app_preferences::save_to_path(path, preferences)?;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(value)
    }

    pub fn readboard_sync_preferences(&self) -> Result<ReadboardSyncPreferencesDto, String> {
        let committed = self.0.lock().expect("preferences transaction");
        let value = committed
            .as_ref()
            .ok_or("Preferences must finish loading before synchronization.")?
            .preferences
            .readboard_sync;
        Ok(value)
    }

    pub fn save_readboard_sync_preferences(
        &self,
        path: &Path,
        value: ReadboardSyncPreferencesDto,
    ) -> Result<ReadboardSyncPreferencesDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before synchronization.")?
            .preferences
            .clone();
        preferences.readboard_sync = value;
        let saved = app_preferences::save_to_path(path, preferences)?;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(value)
    }

    pub fn tencent_history(&self) -> Result<app_model::TencentHistoryDto, String> {
        let committed = self.0.lock().expect("preferences transaction");
        Ok(committed
            .as_ref()
            .ok_or("Preferences must finish loading before reading Tencent history.")?
            .preferences
            .tencent_history
            .clone())
    }

    pub fn save_tencent_query(
        &self,
        path: &Path,
        query: Option<app_model::TencentQueryDto>,
    ) -> Result<app_model::TencentHistoryDto, String> {
        let query = query
            .map(|mut query| {
                query.value = query.value.trim().to_string();
                if query.value.is_empty() || query.value.chars().any(char::is_control) {
                    return Err("Enter a non-empty public Tencent username or chessId.".to_string());
                }
                Ok(query)
            })
            .transpose()?;
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before saving Tencent history.")?
            .preferences
            .clone();
        let history = &mut preferences.tencent_history;
        if let Some(query) = query {
            history.recent.retain(|previous| previous != &query);
            history.recent.insert(0, query.clone());
            history.recent.truncate(8);
            history.last_query = Some(query);
        } else {
            *history = app_model::TencentHistoryDto::default();
        }
        let result = history.clone();
        let saved = app_preferences::save_to_path(path, preferences)?;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(result)
    }

    pub fn readboard_path(&self) -> Result<Option<String>, String> {
        let committed = self.0.lock().expect("preferences transaction");
        Ok(committed
            .as_ref()
            .ok_or("Preferences must finish loading before readboard configuration.")?
            .preferences
            .readboard_executable_path
            .clone())
    }

    pub fn save_readboard_path(&self, path: &Path, executable: String) -> Result<String, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let loaded = committed
            .as_mut()
            .ok_or("Preferences must finish loading before readboard configuration.")?;
        let mut preferences = loaded.preferences.clone();
        preferences.readboard_executable_path = Some(executable.clone());
        let saved = app_preferences::save_to_path(path, preferences)?;
        loaded.preferences = saved;
        Ok(executable)
    }

    pub fn fox_kifu(&self) -> Result<app_model::FoxKifuStateDto, String> {
        let committed = self.0.lock().expect("preferences transaction");
        let state = &committed
            .as_ref()
            .ok_or("Preferences must finish loading before reading Fox recents.")?
            .preferences
            .fox_kifu;
        Ok(provider_fox::sanitize_state(state))
    }

    /// Narrow atomic write of the Fox owner field. A failed write keeps the previous durable state.
    pub fn update_fox_kifu(
        &self,
        path: &Path,
        update: impl FnOnce(&app_model::FoxKifuStateDto) -> Result<app_model::FoxKifuStateDto, String>,
    ) -> Result<app_model::FoxKifuStateDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before saving Fox recents.")?
            .preferences
            .clone();
        preferences.fox_kifu =
            provider_fox::sanitize_state(&update(&provider_fox::sanitize_state(&preferences.fox_kifu))?);
        let saved = app_preferences::save_to_path(path, preferences)?;
        let state = saved.fox_kifu.clone();
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(state)
    }

    /// Persists the match defaults derived from the saved ones, then installs the match with the
    /// defaults actually written. Nothing is installed when the write fails.
    pub(crate) fn commit_match<T>(
        &self,
        path: &Path,
        manager: &ForegroundEngineManager,
        owner: &str,
        defaults: impl FnOnce(&app_model::MatchDefaultsDto) -> app_model::MatchDefaultsDto,
        install: impl FnOnce(&app_model::MatchDefaultsDto) -> T,
    ) -> crate::EngineCommandResult<T> {
        // Same lock order as preference saving: preferences, then manager.
        let mut committed = self.0.lock().expect("preferences transaction");
        manager
            .commit_reserved_match(owner, || {
                let mut preferences = committed
                    .as_ref()
                    .ok_or("Preferences must finish loading before starting a match.")?
                    .preferences
                    .clone();
                preferences.match_defaults = defaults(&preferences.match_defaults);
                let saved = app_preferences::save_to_path(path, preferences)?;
                let installed = install(&saved.match_defaults);
                *committed = Some(AppPreferencesLoadResultDto {
                    preferences: saved,
                    recovery: None,
                });
                Ok(installed)
            })
            .map_err(Box::new)
    }

    pub fn update_workspace_visibility(
        &self,
        path: &Path,
        left: Option<bool>,
        right: Option<bool>,
    ) -> Result<app_model::WorkspaceVisibilityDto, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before changing rail visibility.")?
            .preferences
            .clone();
        if let Some(value) = left {
            preferences.workspace_visibility.left = value;
        }
        if let Some(value) = right {
            preferences.workspace_visibility.right = value;
        }
        let saved = app_preferences::save_to_path(path, preferences)?;
        let visibility = saved.workspace_visibility;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(visibility)
    }

    pub fn update_recent_history(
        &self,
        path: &Path,
        opened_path: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before updating recent history.")?
            .preferences
            .clone();
        preferences.recent_game_paths = match opened_path {
            Some(opened) => app_preferences::recent_game_paths(&preferences.recent_game_paths, opened),
            None => Vec::new(),
        };
        let saved = app_preferences::save_to_path(path, preferences)?;
        let paths = saved.recent_game_paths.clone();
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(paths)
    }

    pub fn update_workspace_shares(
        &self,
        path: &Path,
        shares: Option<WorkspaceSharesDto>,
    ) -> Result<Option<WorkspaceSharesDto>, String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before updating workspace shares.")?
            .preferences
            .clone();
        preferences.workspace_shares = shares;
        let saved = app_preferences::save_to_path(path, preferences)?;
        let shares = saved.workspace_shares;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(shares)
    }

    pub fn update_window_geometry(
        &self,
        path: &Path,
        geometry: app_model::WindowGeometryDto,
    ) -> Result<(), String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before updating window geometry.")?
            .preferences
            .clone();
        preferences.window_geometry = Some(geometry);
        let saved = app_preferences::save_to_path(path, preferences)?;
        committed.as_mut().unwrap().preferences = saved;
        Ok(())
    }

    pub fn pin_intent(&self) -> Result<bool, String> {
        let committed = self.0.lock().expect("preferences transaction");
        Ok(committed
            .as_ref()
            .ok_or("Preferences must finish loading before pinning.")?
            .preferences
            .main_window_always_on_top)
    }

    pub fn save_pin(&self, path: &Path, value: bool) -> Result<(), String> {
        let mut committed = self.0.lock().expect("preferences transaction");
        let mut preferences = committed
            .as_ref()
            .ok_or("Preferences must finish loading before pinning.")?
            .preferences
            .clone();
        preferences.main_window_always_on_top = value;
        let saved = app_preferences::save_to_path(path, preferences)?;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery: None,
        });
        Ok(())
    }

    pub fn image_export_directory(&self) -> Result<Option<std::path::PathBuf>, String> {
        let committed = self.0.lock().expect("preferences transaction");
        let value = committed
            .as_ref()
            .ok_or("Preferences must finish loading before image export.")?
            .preferences
            .recent_image_export_directory
            .as_deref();
        Ok(value.map(std::path::PathBuf::from).filter(|path| path.is_dir()))
    }

    pub fn record_image_export(&self, path: &Path, target: &Path) -> Result<(), String> {
        let directory = target
            .parent()
            .ok_or("Image target has no parent directory.")?
            .canonicalize()
            .map_err(|error| format!("failed to resolve image export directory: {error}"))?;
        let mut committed = self.0.lock().expect("preferences transaction");
        let loaded = committed
            .as_ref()
            .ok_or("Preferences must finish loading before image export.")?;
        let mut preferences = loaded.preferences.clone();
        preferences.recent_image_export_directory = Some(directory.to_string_lossy().into_owned());
        let recovery = loaded.recovery.clone();
        let saved = app_preferences::save_to_path(path, preferences)?;
        *committed = Some(AppPreferencesLoadResultDto {
            preferences: saved,
            recovery,
        });
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
                preferences =
                    manager.commit_continuous_preferences(start, preferences.continuous_budget, || {
                        let saved = app_preferences::save_to_path(path, preferences)?;
                        *committed = Some(AppPreferencesLoadResultDto {
                            preferences: saved.clone(),
                            recovery: None,
                        });
                        Ok(saved)
                    })?;
                if start && !finite {
                    manager.authorize_continuous_start();
                }
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
        exact_position: Err("Exact GTP history is unavailable for this analysis surface.".into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::{ContinuousAnalysisPhaseDto, ForegroundEngineLifecycleDto};
    use engine_manager::{ForegroundEngineConfig, InMemoryEngineProfileCatalog};
    use std::sync::Arc;

    #[test]
    fn image_export_directory_survives_stale_form_failure_and_restart() {
        let directory = std::env::temp_dir().join(format!("image-export-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let mut stale = state.load(&path, &manager).unwrap().preferences;
        assert_eq!(state.image_export_directory().unwrap(), None);
        state
            .record_image_export(&path, &directory.join("board.png"))
            .unwrap();
        stale.show_coordinates = false;
        state.save(&path, &manager, stale).unwrap();
        let expected = directory.canonicalize().unwrap();
        assert_eq!(state.image_export_directory().unwrap(), Some(expected.clone()));
        let durable = std::fs::read(&path).unwrap();
        assert!(state
            .record_image_export(&directory, &std::env::temp_dir().join("other.png"))
            .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        assert_eq!(state.image_export_directory().unwrap(), Some(expected.clone()));
        let restarted = PreferencesState::default();
        restarted.load(&path, &manager).unwrap();
        assert_eq!(restarted.image_export_directory().unwrap(), Some(expected));
        std::fs::remove_dir_all(&directory).unwrap();
        assert_eq!(restarted.image_export_directory().unwrap(), None);
    }

    #[test]
    fn readboard_path_survives_stale_save_restart_and_failed_write() {
        let directory = std::env::temp_dir().join(format!("readboard-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let mut stale = state.load(&path, &manager).unwrap().preferences;
        let executable = r"C:\Readboard Tools\readboard.exe";
        state.save_readboard_path(&path, executable.into()).unwrap();
        stale.show_coordinates = false;
        state.save(&path, &manager, stale).unwrap();
        assert!(state
            .save_readboard_path(&directory, r"C:\Other\readboard.exe".into())
            .is_err());
        assert_eq!(state.readboard_path().unwrap().as_deref(), Some(executable));
        let restarted = PreferencesState::default()
            .load(&path, &manager)
            .unwrap()
            .preferences;
        assert_eq!(restarted.readboard_executable_path.as_deref(), Some(executable));
        assert!(!restarted.show_coordinates);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn geometry_autosave_preserves_unconsumed_preference_recovery() {
        let directory = std::env::temp_dir().join(format!("geometry-recovery-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        std::fs::write(&path, b"not json").unwrap();
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let initial = state.load(&path, &manager).unwrap();
        assert!(initial.recovery.is_some());
        state
            .update_window_geometry(
                &path,
                app_model::WindowGeometryDto {
                    x: Some(40.0),
                    y: Some(50.0),
                    width: 1100.0,
                    height: 720.0,
                    scale_factor: 1.0,
                    maximized: false,
                },
            )
            .unwrap();
        assert_eq!(state.load(&path, &manager).unwrap().recovery, initial.recovery);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn workspace_updates_merge_with_other_owners_and_fail_atomically() {
        let directory = std::env::temp_dir().join(format!("workspace-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let mut stale = state.load(&path, &manager).unwrap().preferences;
        let shares = Some(WorkspaceSharesDto {
            left: 0.2,
            right: 0.3,
        });
        let before = manager.snapshot();
        state.update_workspace_shares(&path, shares).unwrap();
        let geometry = app_model::WindowGeometryDto {
            x: Some(-1200.0),
            y: Some(50.0),
            width: 1100.0,
            height: 720.0,
            scale_factor: 1.25,
            maximized: true,
        };
        state.update_window_geometry(&path, geometry).unwrap();
        assert_eq!(manager.snapshot(), before);
        stale.board_theme = "high-contrast".into();
        stale.continuous_budget.continuous_time_limit_seconds = 123;
        let saved = state.save(&path, &manager, stale).unwrap();
        assert_eq!(saved.workspace_shares, shares);
        assert_eq!(saved.window_geometry, Some(geometry));
        state
            .update_recent_history(&path, Some("/games/new.sgf"))
            .unwrap();
        let before_reset = state.load(&path, &manager).unwrap().preferences;
        let durable = std::fs::read(&path).unwrap();
        assert!(state.update_workspace_shares(&directory, None).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        assert_eq!(state.load(&path, &manager).unwrap().preferences, before_reset);
        assert!(state
            .update_workspace_shares(
                &path,
                Some(WorkspaceSharesDto {
                    left: 0.6,
                    right: 0.4
                })
            )
            .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        let changed = app_model::WindowGeometryDto {
            x: Some(40.0),
            maximized: false,
            ..geometry
        };
        assert!(state.update_window_geometry(&directory, changed).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        assert_eq!(state.load(&path, &manager).unwrap().preferences, before_reset);
        state.update_workspace_shares(&path, None).unwrap();
        let restarted = PreferencesState::default()
            .load(&path, &manager)
            .unwrap()
            .preferences;
        assert_eq!(
            restarted,
            AppPreferencesDto {
                workspace_shares: None,
                ..before_reset
            }
        );
        state.update_window_geometry(&path, changed).unwrap();
        let retried = PreferencesState::default()
            .load(&path, &manager)
            .unwrap()
            .preferences;
        assert_eq!(
            retried,
            AppPreferencesDto {
                window_geometry: Some(changed),
                ..restarted
            }
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn network_preferences_fail_atomically_restore_and_preserve_other_owner_writes() {
        let directory = std::env::temp_dir().join(format!("network-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let stale = state.load(&path, &manager).unwrap().preferences;
        let before = state.network().snapshot();
        let identity = state.network().begin(before.policy_revision, 0).unwrap();
        let lease = state.network().operation(&identity).unwrap().lease();
        let manual = app_model::NetworkSettingsDto {
            mode: app_model::NetworkModeDto::Manual,
            manual_host: "localhost".into(),
            manual_port: 8123,
        };
        assert!(state.save_network(&directory, manual.clone()).is_err());
        assert_eq!(state.network().snapshot(), before);
        assert!(lease.check().is_ok());
        let saved = state.save_network(&path, manual.clone()).unwrap();
        assert_eq!(saved.policy_revision, before.policy_revision + 1);
        assert_eq!(
            lease.check().unwrap_err().kind,
            app_model::ProviderErrorKind::Cancelled
        );
        state.save(&path, &manager, stale).unwrap();
        let restarted = PreferencesState::default();
        assert_eq!(
            restarted.load(&path, &manager).unwrap().preferences.network,
            manual
        );
        assert_eq!(restarted.network().snapshot().settings, manual);
        let secret = app_model::NetworkSettingsDto {
            manual_host: "user:secret@proxy".into(),
            ..manual
        };
        assert!(state.save_network(&path, secret).is_err());
        assert!(!std::fs::read_to_string(&path).unwrap().contains("secret"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn yike_locator_preserves_owner_writes_and_rejects_secrets_atomically() {
        let directory = std::env::temp_dir().join(format!("yike-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let stale = state.load(&path, &manager).unwrap().preferences;
        let locator = "https://home.yikeweiqi.com/#/live/new-room/186031";
        let saved = state.save_yike_locator(&path, Some(locator.into())).unwrap();
        assert_eq!(saved.as_deref(), Some(locator));
        state.save(&path, &manager, stale).unwrap();
        let durable = std::fs::read(&path).unwrap();
        assert!(state.save_yike_locator(&directory, None).is_err());
        assert_eq!(state.yike_locator().unwrap(), saved);
        assert!(state
            .save_yike_locator(&path, Some(format!("{locator}?token=private")))
            .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        let restarted = PreferencesState::default();
        restarted.load(&path, &manager).unwrap();
        assert_eq!(restarted.yike_locator().unwrap(), saved);
        state.save_yike_locator(&path, None).unwrap();
        assert_eq!(state.yike_locator().unwrap(), None);
        assert_eq!(
            app_preferences::load_from_path(&path)
                .unwrap()
                .preferences
                .yike_locator,
            None
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn external_sync_preferences_survive_stale_save_without_changing_global_sound() {
        let directory = std::env::temp_dir().join(format!("sync-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let stale = state.load(&path, &manager).unwrap().preferences;
        let preferences = app_model::YikeSyncPreferencesDto {
            interval_seconds: 3,
            locator: Some("https://home.yikeweiqi.com/#/unite/79438407".into()),
            jump_to_last: true,
            mute: true,
        };
        state
            .save_yike_sync_preferences(&path, preferences.clone())
            .unwrap();
        let saved = state.save(&path, &manager, stale.clone()).unwrap();
        assert_eq!(saved.yike_sync, preferences);
        let mut expected = stale;
        expected.yike_sync = preferences.clone();
        assert_eq!(saved, expected);
        assert!(state
            .save_yike_sync_preferences(&directory, app_model::YikeSyncPreferencesDto::default())
            .is_err());
        assert!(state
            .save_yike_sync_preferences(
                &path,
                app_model::YikeSyncPreferencesDto {
                    interval_seconds: 0,
                    ..preferences.clone()
                }
            )
            .is_err());
        assert_eq!(state.yike_sync_preferences().unwrap(), preferences);
        let restarted = PreferencesState::default();
        restarted.load(&path, &manager).unwrap();
        assert_eq!(restarted.yike_sync_preferences().unwrap(), preferences);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn readboard_sync_preferences_survive_stale_save_without_changing_global_sound() {
        let directory = std::env::temp_dir().join(format!("readboard-sync-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        assert_eq!(
            state.readboard_sync_preferences().unwrap_err(),
            "Preferences must finish loading before synchronization."
        );
        let stale = state.load(&path, &manager).unwrap().preferences;
        let preferences = ReadboardSyncPreferencesDto {
            always_sync: false,
            focus: false,
            mute: false,
            jump_to_last: true,
        };
        state.save_readboard_sync_preferences(&path, preferences).unwrap();
        let saved = state.save(&path, &manager, stale.clone()).unwrap();
        assert_eq!(saved.readboard_sync, preferences);
        assert_eq!(saved.sound_enabled, stale.sound_enabled);
        let mut expected = stale;
        expected.readboard_sync = preferences;
        assert_eq!(saved, expected);
        assert!(state
            .save_readboard_sync_preferences(&directory, ReadboardSyncPreferencesDto::default())
            .is_err());
        assert_eq!(state.readboard_sync_preferences().unwrap(), preferences);
        let restarted = PreferencesState::default();
        restarted.load(&path, &manager).unwrap();
        assert_eq!(restarted.readboard_sync_preferences().unwrap(), preferences);
        assert!(restarted.load(&path, &manager).unwrap().preferences.sound_enabled);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn tencent_history_bounds_deduplicates_and_survives_stale_saves_and_failed_clear() {
        use app_model::{TencentHistoryDto, TencentQueryDto, TencentQueryKindDto};
        let directory = std::env::temp_dir().join(format!("tencent-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let stale = state.load(&path, &manager).unwrap().preferences;
        let locator = "https://home.yikeweiqi.com/#/live/new-room/186031";
        state.save_yike_locator(&path, Some(locator.into())).unwrap();
        for number in 0..10 {
            state
                .save_tencent_query(
                    &path,
                    Some(TencentQueryDto {
                        kind: TencentQueryKindDto::Username,
                        value: format!(" player-{number} "),
                    }),
                )
                .unwrap();
        }
        let newest = TencentQueryDto {
            kind: TencentQueryKindDto::Username,
            value: "player-5".into(),
        };
        let saved = state.save_tencent_query(&path, Some(newest.clone())).unwrap();
        assert_eq!(saved.last_query, Some(newest.clone()));
        assert_eq!(
            saved
                .recent
                .iter()
                .map(|query| query.value.as_str())
                .collect::<Vec<_>>(),
            [
                "player-5", "player-9", "player-8", "player-7", "player-6", "player-4", "player-3",
                "player-2"
            ]
        );
        state.save(&path, &manager, stale).unwrap();
        assert!(state.save_tencent_query(&directory, None).is_err());
        assert_eq!(state.tencent_history().unwrap(), saved);
        assert_eq!(state.yike_locator().unwrap().as_deref(), Some(locator));
        let restarted = PreferencesState::default();
        restarted.load(&path, &manager).unwrap();
        assert_eq!(restarted.tencent_history().unwrap(), saved);
        assert!(state
            .save_tencent_query(
                &path,
                Some(TencentQueryDto {
                    kind: TencentQueryKindDto::ChessId,
                    value: "  ".into(),
                })
            )
            .is_err());
        assert_eq!(state.tencent_history().unwrap(), saved);
        let other = TencentQueryDto {
            kind: TencentQueryKindDto::ChessId,
            value: "player-5".into(),
        };
        let both = state.save_tencent_query(&path, Some(other.clone())).unwrap();
        assert_eq!(both.recent[0], other);
        assert_eq!(both.recent[1], newest);
        assert_eq!(
            state.save_tencent_query(&path, None).unwrap(),
            TencentHistoryDto::default()
        );
        assert_eq!(
            app_preferences::load_from_path(&path)
                .unwrap()
                .preferences
                .tencent_history,
            TencentHistoryDto::default()
        );
        assert_eq!(state.yike_locator().unwrap().as_deref(), Some(locator));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn fox_recents_persist_atomically_survive_stale_saves_and_clear_separately() {
        let directory = std::env::temp_dir().join(format!("fox-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let stale = state.load(&path, &manager).unwrap().preferences;
        let lookup = app_model::FoxLookupDto {
            kind: app_model::FoxLookupKindDto::Nickname,
            value: "绝艺".into(),
        };
        let account = app_model::FoxAccountDto {
            uid: "8772065".into(),
            nickname: "绝艺".into(),
        };
        let remember = |current: &app_model::FoxKifuStateDto| {
            provider_fox::remember_lookup(current, &lookup, Some(&account)).map_err(|error| error.message)
        };
        let saved = state.update_fox_kifu(&path, remember).unwrap();
        assert_eq!(saved.recents, vec![account.clone()]);
        assert_eq!(saved.last_query.as_ref(), Some(&lookup));
        state.save(&path, &manager, stale).unwrap();
        let durable = std::fs::read(&path).unwrap();
        assert!(state
            .update_fox_kifu(&directory, |_| Ok(Default::default()))
            .is_err());
        assert_eq!(state.fox_kifu().unwrap(), saved);
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        let restarted = PreferencesState::default();
        restarted.load(&path, &manager).unwrap();
        assert_eq!(restarted.fox_kifu().unwrap(), saved);
        let cleared = state
            .update_fox_kifu(&path, |current| {
                Ok(app_model::FoxKifuStateDto {
                    recents: Vec::new(),
                    last_query: current.last_query.clone(),
                })
            })
            .unwrap();
        assert!(cleared.recents.is_empty());
        assert_eq!(cleared.last_query.as_ref(), Some(&lookup));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("8772065") && !text.contains("chesslist"));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rail_visibility_preserves_other_owners_and_failed_writes_do_not_commit() {
        let directory = std::env::temp_dir().join(format!("rail-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let mut stale = state.load(&path, &manager).unwrap().preferences;
        state
            .update_workspace_visibility(&path, Some(false), None)
            .unwrap();
        stale.show_coordinates = false;
        state.save(&path, &manager, stale).unwrap();
        state
            .update_recent_history(&path, Some("/games/test.sgf"))
            .unwrap();
        state
            .update_workspace_visibility(&path, None, Some(false))
            .unwrap();
        assert!(state
            .update_workspace_visibility(&directory, Some(true), None)
            .is_err());
        let loaded = state.load(&path, &manager).unwrap().preferences;
        assert_eq!(
            loaded.workspace_visibility,
            app_model::WorkspaceVisibilityDto {
                left: false,
                right: false
            }
        );
        assert!(!loaded.show_coordinates);
        assert_eq!(loaded.recent_game_paths, ["/games/test.sgf"]);
        let restarted = PreferencesState::default()
            .load(&path, &manager)
            .unwrap()
            .preferences;
        assert_eq!(restarted, loaded);
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn recent_history_failure_clear_restart_and_stale_preferences_are_atomic() {
        let directory = std::env::temp_dir().join(format!("recent-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let state = PreferencesState::default();
        let mut before_open = state.load(&path, &manager).unwrap().preferences;
        before_open.restore_last_session = true;
        state.save(&path, &manager, before_open.clone()).unwrap();
        state
            .update_recent_history(&path, Some("/games/old.sgf"))
            .unwrap();
        let durable = std::fs::read(&path).unwrap();
        assert!(state
            .update_recent_history(&directory, Some("/games/new.gib"))
            .is_err());
        assert!(state.update_recent_history(&directory, None).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), durable);
        assert_eq!(
            state.load(&path, &manager).unwrap().preferences.recent_game_paths,
            ["/games/old.sgf"]
        );
        let saved = state.save(&path, &manager, before_open).unwrap();
        assert_eq!(saved.recent_game_paths, ["/games/old.sgf"]);
        state
            .update_recent_history(&path, Some("/games/new.gib"))
            .unwrap();
        let reloaded = PreferencesState::default()
            .load(&path, &manager)
            .unwrap()
            .preferences;
        assert_eq!(reloaded.recent_game_paths, ["/games/new.gib", "/games/old.sgf"]);
        state.update_recent_history(&path, None).unwrap();
        let cleared = app_preferences::load_from_path(&path).unwrap().preferences;
        assert_eq!(
            cleared,
            AppPreferencesDto {
                recent_game_paths: Vec::new(),
                ..reloaded
            }
        );
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn no_engine_primary_refusal_preserves_durable_intent() {
        let directory = std::env::temp_dir().join(format!("continuous-prefs-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("preferences.json");
        let manager = ForegroundEngineManager::new(
            Arc::new(InMemoryEngineProfileCatalog::new()),
            ForegroundEngineConfig::for_tests(),
        );
        let preferences = PreferencesState::default();
        let loaded = preferences.load(&path, &manager).unwrap().preferences;
        preferences.save(&path, &manager, loaded.clone()).unwrap();
        let durable = std::fs::read(&path).unwrap();
        let before = manager.snapshot();
        assert!(preferences.primary(&path, &manager).is_err());
        assert_eq!(manager.snapshot(), before);
        assert_eq!(std::fs::read(&path).unwrap(), durable);

        // Explicit preferences can still change durable intent without a Run.
        preferences
            .save(
                &path,
                &manager,
                AppPreferencesDto {
                    continuous_analysis_enabled: false,
                    ..loaded
                },
            )
            .unwrap();
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
        std::fs::remove_dir_all(directory).unwrap();
    }
}
