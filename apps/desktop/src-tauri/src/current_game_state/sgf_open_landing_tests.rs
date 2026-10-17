use super::*;
use app_model::{DocumentDepartureAdmissionDto, GameFileFormatDto as Format, GameLoadSourceDto as Source};

const BRANCHING: &str = "(;GM[1]FF[4]SZ[9]KM[6.5]AB[aa]PL[W];W[];B[bb](;W[cc];C[main tail])(;W[dd]))";

fn id(admission: DocumentDepartureAdmissionDto) -> u64 {
    match admission {
        DocumentDepartureAdmissionDto::Ready { departure_id }
        | DocumentDepartureAdmissionDto::NeedsDecision { departure_id } => departure_id,
    }
}

fn commit(state: &CurrentGameState, departure_id: u64) -> CurrentGameResultDto {
    state.begin_protected_commit(departure_id, &[]).unwrap();
    state.commit_replacement(departure_id).unwrap().current.unwrap()
}

#[test]
fn local_sgf_entries_choose_exact_root_or_first_child_leaf_without_changing_content() {
    for source in [Source::Open, Source::Recent, Source::FileDelivery] {
        for last in [false, true] {
            let state = CurrentGameState::default();
            let departure_id = id(state
                .prepare_loaded_replacement(BRANCHING, None, source, Some(Format::Sgf), last)
                .unwrap());
            let opened = commit(&state, departure_id);
            let expected = if last { vec![0, 0, 0, 0] } else { vec![] };
            assert_eq!(opened.selected_path.indices, expected);
            assert!(!opened.dirty);
            assert!(!opened.can_undo);
            assert!(!opened.can_redo);
            assert_eq!(opened.snapshot.position.board_width, 9);
            if last {
                assert_eq!(opened.snapshot.personal_comment, "main tail");
                assert_eq!(opened.snapshot.position.move_number, 3);
            } else {
                assert_eq!(opened.snapshot.position.to_play, PlayerColor::White);
                assert_eq!(
                    opened.snapshot.position.stones,
                    vec![StoneDto {
                        x: 0,
                        y: 0,
                        color: PlayerColor::Black
                    }]
                );
            }
            let original = CurrentSgfDocument::open(BRANCHING).unwrap().serialize().unwrap();
            assert_eq!(state.serialize().unwrap(), original);
            let recovery = state.take_due_recovery_write(u64::MAX).unwrap();
            assert_eq!(recovery.selected_path, opened.selected_path);
            let restored = CurrentGameState::default().restore_envelope(recovery).unwrap();
            assert_eq!(restored.selected_path, opened.selected_path);
            let sibling = state
                .select_path(
                    NodePath {
                        indices: vec![0, 0, 1],
                    },
                    opened.generation,
                )
                .unwrap();
            assert_eq!(sibling.snapshot.position.move_number, 3);
        }
    }
}

#[test]
fn root_only_and_excluded_sources_retain_their_existing_selection() {
    for last in [false, true] {
        for (source, format, text, expected) in [
            (Source::Open, Some(Format::Sgf), "(;SZ[9]AB[aa]PL[W])", vec![]),
            (Source::Open, Some(Format::Gib), BRANCHING, vec![0, 0, 0, 0]),
            (Source::Recent, Some(Format::Gib), BRANCHING, vec![0, 0, 0, 0]),
            (
                Source::FileDelivery,
                Some(Format::Gib),
                BRANCHING,
                vec![0, 0, 0, 0],
            ),
            (Source::Text, None, BRANCHING, vec![0, 0, 0, 0]),
            (Source::Clipboard, None, BRANCHING, vec![0, 0, 0, 0]),
            (Source::Excluded, None, BRANCHING, vec![0, 0, 0, 0]),
        ] {
            let state = CurrentGameState::default();
            let departure_id = id(state
                .prepare_loaded_replacement(text, None, source, format, last)
                .unwrap());
            assert_eq!(commit(&state, departure_id).selected_path.indices, expected);
        }
    }
}

#[test]
fn cancelled_and_invalid_candidates_preserve_old_document_and_cursor() {
    let state = CurrentGameState::default();
    state.replace(BRANCHING, None).unwrap();
    let selected = state
        .select_path(
            NodePath {
                indices: vec![0, 0, 1],
            },
            state.inspect().0,
        )
        .unwrap();
    let edited = state
        .set_personal_comment(selected.selected_path.clone(), "keep me".into())
        .unwrap();
    let before = state.serialize().unwrap();
    assert!(state
        .prepare_loaded_replacement("broken", None, Source::Open, Some(Format::Sgf), false)
        .is_err());
    let departure_id = id(state
        .prepare_loaded_replacement("(;SZ[13])", None, Source::Open, Some(Format::Sgf), false)
        .unwrap());
    state.cancel_replacement(departure_id).unwrap();
    assert_eq!(state.serialize().unwrap(), before);
    let holder = state.holder.lock().unwrap();
    assert_eq!(holder.selected_path, edited.selected_path);
    assert!(holder.dirty);
    assert_eq!(holder.generation, edited.generation);
}

#[test]
fn committed_preference_is_captured_once_and_save_does_not_move_the_current_game() {
    use crate::continuous_analysis::PreferencesState;
    use engine_manager::{ForegroundEngineConfig, ForegroundEngineManager, InMemoryEngineProfileCatalog};
    let directory = std::env::temp_dir().join(format!("sgf-landing-order-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&directory).unwrap();
    let path = directory.join("preferences.json");
    let manager = ForegroundEngineManager::new(
        std::sync::Arc::new(InMemoryEngineProfileCatalog::new()),
        ForegroundEngineConfig::for_tests(),
    );
    let preferences = PreferencesState::default();
    assert!(preferences.load_sgf_last().is_err());
    let initial = preferences.load(&path, &manager).unwrap().preferences;
    let state = CurrentGameState::default();
    state.connect_analysis_manager(manager.clone());
    let old = state.replace(BRANCHING, None).unwrap();
    let mut pending = initial.clone();
    pending.load_sgf_last = true;
    assert!(!preferences.load_sgf_last().unwrap());
    assert!(preferences.save(&directory, &manager, pending.clone()).is_err());
    assert!(!preferences.load_sgf_last().unwrap());
    let admitted = id(state
        .prepare_loaded_replacement(
            BRANCHING,
            None,
            Source::Open,
            Some(Format::Sgf),
            preferences.load_sgf_last().unwrap(),
        )
        .unwrap());
    preferences.save(&path, &manager, pending).unwrap();
    assert_eq!(state.holder.lock().unwrap().selected_path, old.selected_path);
    let opened = commit(&state, admitted);
    assert_eq!(opened.selected_path.indices, Vec::<u32>::new());
    assert_eq!(
        state.holder.lock().unwrap().analysis_target,
        Some((opened.generation, opened.selected_path.clone()))
    );
    let recovery = state.take_due_recovery_write(u64::MAX).unwrap();
    assert_eq!(recovery.selected_path, opened.selected_path);
    state.finish_recovery_write(recovery, Ok(()));
    assert!(manager.snapshot().selected_node_job.is_none());
    let next = id(state
        .prepare_loaded_replacement(
            BRANCHING,
            None,
            Source::Recent,
            Some(Format::Sgf),
            preferences.load_sgf_last().unwrap(),
        )
        .unwrap());
    let opened = commit(&state, next);
    assert_eq!(opened.selected_path.indices, vec![0, 0, 0, 0]);
    assert_eq!(
        state.holder.lock().unwrap().analysis_target,
        Some((opened.generation, opened.selected_path.clone()))
    );
    assert_eq!(
        state.take_due_recovery_write(u64::MAX).unwrap().selected_path,
        opened.selected_path
    );
    assert!(manager.snapshot().selected_node_job.is_none());
    std::fs::remove_dir_all(directory).unwrap();
}
