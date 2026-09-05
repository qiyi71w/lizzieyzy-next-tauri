use crate::current_game_state::{recovery, CurrentGameState};
use app_model::{
    ApplicationExitDispositionDto, CurrentGameError, CurrentGameResultDto, RecoveryProtectionDto,
    RecoveryStartupDto,
};
use current_game_recovery::{FileRecoveryStore, RECOVERY_FILE};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

pub const RECOVERY_PROTECTION_EVENT: &str = "current-game-recovery://protection";

pub fn spawn_recovery_writer(handle: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(50));
        let Some(state) = handle.try_state::<CurrentGameState>() else {
            continue;
        };
        let Some(store) = handle.try_state::<Mutex<FileRecoveryStore>>() else {
            continue;
        };
        let protection = {
            let Ok(store) = store.lock() else {
                continue;
            };
            let Some(envelope) = state.take_due_recovery_write(now_ms()) else {
                continue;
            };
            let result = store.replace(&envelope);
            state.finish_recovery_write(envelope, result);
            state.recovery_protection()
        };
        let _ = handle.emit(RECOVERY_PROTECTION_EVENT, protection);
    });
}

#[tauri::command]
pub fn inspect_current_game_recovery(
    state: State<CurrentGameState>,
    store: State<Mutex<FileRecoveryStore>>,
) -> RecoveryStartupDto {
    let store = store.lock().expect("recovery store");
    recovery::inspect_and_seed(&state, &store)
}

#[tauri::command]
pub fn restore_current_game_recovery(
    state: State<CurrentGameState>,
    store: State<Mutex<FileRecoveryStore>>,
) -> Result<CurrentGameResultDto, CurrentGameError> {
    let store = store.lock().expect("recovery store");
    state.restore_from_store(&store)
}

#[tauri::command]
pub fn discard_current_game_recovery(
    state: State<CurrentGameState>,
    store: State<Mutex<FileRecoveryStore>>,
) -> Result<(), String> {
    let store = store.lock().expect("recovery store");
    persist_explicit_discard(&state, &store)
}

#[tauri::command]
pub fn retry_current_game_recovery(
    state: State<CurrentGameState>,
    store: State<Mutex<FileRecoveryStore>>,
) -> RecoveryProtectionDto {
    let store = store.lock().expect("recovery store");
    state.persist_retry(&store, now_ms())
}

#[tauri::command]
pub fn current_game_recovery_protection(state: State<CurrentGameState>) -> RecoveryProtectionDto {
    state.recovery_protection()
}

pub fn persist_explicit_discard(state: &CurrentGameState, store: &FileRecoveryStore) -> Result<(), String> {
    let loaded = store.load()?;
    let Some(envelope) = loaded else {
        return Ok(());
    };
    if matches!(
        envelope.disposition,
        ApplicationExitDispositionDto::ExplicitDiscard
    ) {
        state.mark_discard_committed(envelope);
        return Ok(());
    }
    let mut discarded = envelope;
    discarded.disposition = ApplicationExitDispositionDto::ExplicitDiscard;
    recovery::persist_recovery_envelope(store, &discarded)?;
    state.mark_discard_committed(discarded);
    Ok(())
}

pub fn recovery_file_path(app_handle: &AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|err| format!("failed to resolve app data directory for current-game recovery: {err}"))?;
    std::fs::create_dir_all(&dir).map_err(|err| {
        format!(
            "failed to create current-game recovery directory {}: {err}",
            dir.display()
        )
    })?;
    Ok(dir.join(RECOVERY_FILE))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::current_game_state::recovery::{persist_committed_exit, persist_replacement_snapshot};
    use app_model::{
        ApplicationExitOutcomeDto, ApplicationTeardownAttemptDto, NodePath, RecoveryEnvelopeDto,
        RecoveryStartupDto,
    };
    use std::fs;

    const EMPTY: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";
    const BRANCH: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白];B[pd])";

    fn temp_store() -> (std::path::PathBuf, FileRecoveryStore) {
        let unique = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        );
        let dir = std::env::temp_dir()
            .join("lizzieyzy-session-recovery")
            .join(unique);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(RECOVERY_FILE);
        (dir, FileRecoveryStore::new(path))
    }

    fn envelope(sgf: &str, disposition: ApplicationExitDispositionDto) -> RecoveryEnvelopeDto {
        RecoveryEnvelopeDto {
            document_seq: 3,
            snapshot_seq: 2,
            sgf_text: sgf.to_string(),
            selected_path: NodePath { indices: vec![] },
            source_path: Some("/tmp/recovered.sgf".to_string()),
            dirty: true,
            disposition,
        }
    }

    #[test]
    fn inspect_seeds_abnormal_candidate_and_restore_installs_it() {
        let (dir, store) = temp_store();
        let stored = envelope(BRANCH, ApplicationExitDispositionDto::ExitIncomplete);
        store.replace(&stored).unwrap();
        let state = CurrentGameState::default();
        state.replace(EMPTY, None).unwrap();
        let startup = recovery::inspect_and_seed(&state, &store);
        assert!(matches!(startup, RecoveryStartupDto::Abnormal { .. }));
        let restored = state.restore_from_store(&store).unwrap();
        assert!(state.serialize().unwrap().contains("B[pd]"));
        assert_eq!(restored.native_path.as_deref(), Some("/tmp/recovered.sgf"));
        assert!(restored.dirty);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn discard_clears_candidate_and_allows_a_newer_document_snapshot() {
        let (dir, store) = temp_store();
        store
            .replace(&envelope(BRANCH, ApplicationExitDispositionDto::ExitIncomplete))
            .unwrap();
        let state = CurrentGameState::default();
        recovery::inspect_and_seed(&state, &store);
        persist_explicit_discard(&state, &store).unwrap();
        assert!(matches!(
            recovery::inspect_recovery_store(&store),
            RecoveryStartupDto::None
        ));
        let replaced = state.replace(EMPTY, Some("/tmp/next.sgf".to_string())).unwrap();
        assert!(!replaced.dirty);
        state
            .persist_flush(&store, ApplicationExitDispositionDto::ExitIncomplete)
            .unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.sgf_text, EMPTY);
        assert!(loaded.document_seq > 3);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_exit_persist_does_not_report_success() {
        let parent = std::env::temp_dir().join(format!(
            "lizzieyzy-recovery-not-dir-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::write(&parent, "not a directory").unwrap();
        let store = FileRecoveryStore::new(parent.join(RECOVERY_FILE));
        let state = CurrentGameState::default();
        state.replace(EMPTY, None).unwrap();
        let outcome = persist_committed_exit(
            &state,
            &store,
            ApplicationExitOutcomeDto {
                committed: true,
                analysis_stopped: true,
                current: None,
                message: "Application exit completed.".to_string(),
                disposition: Some(ApplicationExitDispositionDto::CleanCompleted),
                teardown: Some(ApplicationTeardownAttemptDto::Completed),
                recovery_persist_error: None,
            },
        );
        assert!(outcome.recovery_persist_error.is_some());
        assert!(outcome.message.contains("Failed to persist"));
        let _ = fs::remove_file(parent);
    }

    #[test]
    fn timed_out_exit_anyway_flushes_an_incomplete_envelope() {
        let (dir, store) = temp_store();
        let state = CurrentGameState::default();
        state.replace(EMPTY, None).unwrap();
        persist_committed_exit(
            &state,
            &store,
            ApplicationExitOutcomeDto {
                committed: true,
                analysis_stopped: true,
                current: None,
                message: "Application exit completed.".to_string(),
                disposition: Some(ApplicationExitDispositionDto::ExitIncomplete),
                teardown: Some(ApplicationTeardownAttemptDto::TimedOut {
                    outstanding: vec!["foreground engine".to_string()],
                }),
                recovery_persist_error: None,
            },
        );
        let loaded = store.load().unwrap().expect("exit-anyway envelope");
        assert_eq!(loaded.sgf_text, EMPTY);
        assert_eq!(loaded.disposition, ApplicationExitDispositionDto::ExitIncomplete);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn replacement_flush_writes_the_new_document_immediately() {
        let (dir, store) = temp_store();
        let state = CurrentGameState::default();
        state.replace(EMPTY, None).unwrap();
        persist_replacement_snapshot(&state, &store).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert_eq!(loaded.sgf_text, EMPTY);
        state.replace(BRANCH, None).unwrap();
        persist_replacement_snapshot(&state, &store).unwrap();
        let loaded = store.load().unwrap().unwrap();
        assert!(loaded.sgf_text.contains("B[pd]"));
        let _ = fs::remove_dir_all(dir);
    }
}
