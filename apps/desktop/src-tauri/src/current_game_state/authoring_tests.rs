use super::*;
use app_model::{PointDto, SgfAuthoringActionDto as Action, SgfTransformDto};

fn point(x: u8, y: u8) -> PointDto {
    PointDto { x, y }
}

#[test]
fn authoring_add_reuse_navigates_without_editing_and_preserves_undo_redo() {
    use engine_manager::{ForegroundEngineConfig, ForegroundEngineManager, InMemoryEngineProfileCatalog};
    use std::sync::Arc;

    let source = "(;SZ[9](;B[aa]C[other])(;B[cc]C[keep]XX[value];W[dd]C[descendant]))";
    for color in [Some(PlayerColor::Black), None] {
        for history_state in ["clean", "undo", "redo"] {
            let manager = ForegroundEngineManager::new(
                Arc::new(InMemoryEngineProfileCatalog::new()),
                ForegroundEngineConfig::for_tests(),
            );
            let mut state = CurrentGameState::default();
            state.connect_analysis_manager(manager);
            let opened = state.replace(source, Some("source.sgf".into())).unwrap();
            let mut before = state.select_path(NodePath::default(), opened.generation).unwrap();
            if history_state != "clean" {
                before = state
                    .set_personal_comment(NodePath::default(), "root edit".into())
                    .unwrap();
                if history_state == "redo" {
                    before = state.undo(before.generation).unwrap();
                }
            }
            let serialized = state.serialize().unwrap();
            let identity = state.document_identity();
            let revisions = {
                let holder = state.holder.get_mut().expect("exclusive fixture");
                (
                    holder.undo_revisions.clone(),
                    holder.redo_revisions.clone(),
                    holder.next_edit_revision,
                    holder.edit_revision,
                    holder.nonhistory_revision,
                )
            };
            let pending = state.take_due_recovery_write(u64::MAX).unwrap();
            state.finish_recovery_write(pending, Ok(()));
            let reused = state
                .author(
                    before.generation,
                    before.selected_path.clone(),
                    Action::Add {
                        point: point(2, 2),
                        color,
                        insert: false,
                    },
                )
                .unwrap();
            assert_eq!(reused.selected_path.indices, vec![1]);
            assert_eq!(reused.snapshot.path, reused.selected_path);
            assert_eq!(reused.snapshot.personal_comment, "keep");
            assert_eq!(reused.generation, before.generation);
            assert!(reused.snapshot_seq > before.snapshot_seq);
            assert_eq!(state.document_identity(), identity);
            assert_eq!(reused.dirty, before.dirty);
            assert_eq!(reused.can_undo, before.can_undo);
            assert_eq!(reused.can_redo, before.can_redo);
            assert_eq!(reused.native_path, before.native_path);
            assert_eq!(state.serialize().unwrap(), serialized);
            {
                let holder = state.holder.get_mut().expect("exclusive fixture");
                assert_eq!(
                    (
                        holder.undo_revisions.clone(),
                        holder.redo_revisions.clone(),
                        holder.next_edit_revision,
                        holder.edit_revision,
                        holder.nonhistory_revision,
                    ),
                    revisions
                );
                assert_eq!(
                    holder.analysis_target,
                    Some((reused.generation, reused.selected_path.clone()))
                );
            }
            assert_eq!(
                state
                    .select_path(reused.selected_path.clone(), reused.generation)
                    .unwrap(),
                reused
            );
            let recovery = state.take_due_recovery_write(u64::MAX).expect("cursor recovery");
            assert_eq!(recovery.selected_path, reused.selected_path);
            assert_eq!(recovery.sgf_text, serialized);
            assert_eq!(recovery.dirty, before.dirty);
            match history_state {
                "undo" => {
                    let undone = state.undo(reused.generation).unwrap();
                    assert_eq!(undone.selected_path, NodePath::default());
                    assert!(!undone.dirty);
                    assert!(!undone.can_undo);
                    assert!(undone.can_redo);
                    assert_eq!(state.serialize().unwrap(), source);
                }
                "redo" => {
                    let redone = state.redo(reused.generation).unwrap();
                    assert_eq!(redone.selected_path, NodePath::default());
                    assert!(redone.dirty);
                    assert!(redone.can_undo);
                    assert!(!redone.can_redo);
                    assert_eq!(redone.snapshot.personal_comment, "root edit");
                }
                "clean" => {}
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn authoring_same_path_noop_does_not_advance_snapshot_or_schedule_recovery() {
    let state = CurrentGameState::default();
    let before = state.replace("(;SZ[9];B[cc]C[keep])", None).unwrap();
    let serialized = state.serialize().unwrap();
    let pending = state.take_due_recovery_write(u64::MAX).unwrap();
    state.finish_recovery_write(pending, Ok(()));
    let after = state
        .author(
            before.generation,
            before.selected_path.clone(),
            Action::Drag {
                from: point(2, 2),
                to: point(2, 2),
            },
        )
        .unwrap();
    assert_eq!(after, before);
    assert_eq!(state.serialize().unwrap(), serialized);
    assert!(state.take_due_recovery_write(u64::MAX).is_none());
}

#[test]
fn authoring_stale_cursor_and_reversible_history_are_nondestructive() {
    let state = CurrentGameState::default();
    let before = state
        .replace("(;SZ[5:3];B[aa]C[keep](;W[bb])(;W[cc]))", None)
        .unwrap();
    let action = Action::Drag {
        from: point(0, 0),
        to: point(4, 2),
    };
    let serialized = state.serialize().unwrap();
    assert!(state
        .author(
            before.generation - 1,
            before.selected_path.clone(),
            action.clone()
        )
        .is_err());
    assert!(state
        .author(before.generation, NodePath::default(), action.clone())
        .is_err());
    assert_eq!(state.serialize().unwrap(), serialized);
    let latest = state
        .select_path(before.selected_path.clone(), before.generation)
        .unwrap();
    assert_eq!(latest, before);
    let edited = state
        .author(before.generation, before.selected_path.clone(), action)
        .unwrap();
    assert!(edited.dirty);
    assert!(edited.can_undo);
    assert!(edited.generation > before.generation);
    assert_eq!(edited.selected_path, before.selected_path);
    let undo = state.undo(edited.generation).unwrap();
    assert_eq!(state.serialize().unwrap(), serialized);
    assert!(!undo.can_undo);
    assert!(!undo.dirty);
    let redo = state.redo(undo.generation).unwrap();
    assert!(redo.dirty);
    assert_eq!(redo.selected_path, before.selected_path);
}

#[test]
fn authoring_rectangular_transform_rejection_preserves_holder_and_history() {
    let state = CurrentGameState::default();
    let before = state.replace("(;SZ[5:3];B[aa];W[bb])", None).unwrap();
    assert!(state
        .author(
            before.generation,
            before.selected_path.clone(),
            Action::Transform {
                transform: SgfTransformDto::RotateClockwise
            }
        )
        .is_err());
    assert_eq!(
        state
            .select_path(before.selected_path.clone(), before.generation)
            .unwrap(),
        before
    );
    let after = state
        .author(
            before.generation,
            before.selected_path.clone(),
            Action::Transform {
                transform: SgfTransformDto::MirrorHorizontal,
            },
        )
        .unwrap();
    assert!(after.generation > before.generation);
    assert_eq!(state.serialize().unwrap(), "(;SZ[5:3];B[ea];W[db])");
    let undo = state.undo(after.generation).unwrap();
    assert_eq!(state.serialize().unwrap(), "(;SZ[5:3];B[aa];W[bb])");
    assert_eq!(undo.selected_path, before.selected_path);
}

#[test]
fn authoring_match_trial_external_sync_and_departure_guards_preserve_document() {
    for guard in ["match", "trial", "external", "departure"] {
        let mut state = CurrentGameState::default();
        let before = state.replace("(;SZ[5];B[aa];W[bb])", None).unwrap();
        let serialized = state.serialize().unwrap();
        match guard {
            "match" => {
                state
                    .holder
                    .get_mut()
                    .expect("exclusive fixture")
                    .human_match
                    .snapshot
                    .resources_held = true
            }
            "trial" => {
                state.enter_trial().unwrap();
            }
            "external" => {
                let network = provider_core::network::NetworkState::default();
                state
                    .begin_external_start(
                        &network,
                        "https://home.yikeweiqi.com/#/unite/owner-room".into(),
                        app_model::YikeSyncPreferencesDto::default(),
                    )
                    .unwrap();
            }
            "departure" => {
                state.prepare_replacement("(;SZ[9])", None).unwrap();
            }
            _ => unreachable!(),
        }
        for action in [
            Action::Add {
                point: point(2, 2),
                color: None,
                insert: true,
            },
            Action::Drag {
                from: point(0, 0),
                to: point(2, 2),
            },
            Action::Transform {
                transform: SgfTransformDto::MirrorHorizontal,
            },
            Action::ContinueLadder,
        ] {
            assert!(
                state
                    .author(before.generation, before.selected_path.clone(), action)
                    .is_err(),
                "{guard}"
            );
            assert_eq!(state.serialize().unwrap(), serialized, "{guard}");
        }
        let holder = state.holder.get_mut().expect("exclusive fixture");
        assert_eq!(holder.generation, before.generation);
        assert_eq!(holder.selected_path, before.selected_path);
        assert!(!holder.history.can_undo());
    }
}

#[test]
fn authoring_history_exchanges_legitimate_analysis_without_resetting_source_or_revisions() {
    let state = CurrentGameState::default();
    let opened = state
        .replace("(;SZ[5];B[aa]C[keep])", Some("source.sgf".into()))
        .unwrap();
    let selected = opened.selected_path.clone();
    let payload = |visits| SgfAnalysisPayload {
        engine_name: "KataGo".into(),
        visits,
        winrate_black: 0.55,
        score_mean_black: Some(1.5),
        score_stdev: None,
        pda: None,
        candidates: vec![],
        ownership: None,
    };
    let edited = state
        .author(
            opened.generation,
            selected.clone(),
            Action::Drag {
                from: point(0, 0),
                to: point(4, 4),
            },
        )
        .unwrap();
    let analyzed = state
        .attach_primary_analysis(edited.generation, selected.clone(), payload(100))
        .unwrap();
    assert_eq!(analyzed.generation, edited.generation);
    assert!(analyzed.snapshot_seq > edited.snapshot_seq);
    let edited_sgf = state.serialize().unwrap();
    let undone = state.undo(analyzed.generation).unwrap();
    assert!(undone.generation > analyzed.generation);
    assert!(undone.snapshot_seq > analyzed.snapshot_seq);
    assert_eq!(undone.selected_path, selected);
    assert_eq!(undone.native_path, opened.native_path);
    assert_eq!(undone.snapshot.personal_comment, "keep");
    assert!(undone.snapshot.primary_analysis.is_none());
    assert!(undone.dirty);
    assert!(!undone.can_undo);
    assert!(undone.can_redo);
    assert_eq!(state.serialize().unwrap(), "(;SZ[5];B[aa]C[keep])");
    assert!(state
        .attach_primary_analysis(analyzed.generation, selected.clone(), payload(999))
        .is_err());
    let reanalyzed = state
        .attach_primary_analysis(undone.generation, selected.clone(), payload(200))
        .unwrap();
    let original_sgf = state.serialize().unwrap();
    let redone = state.redo(reanalyzed.generation).unwrap();
    assert!(redone.generation > reanalyzed.generation);
    assert_eq!(redone.selected_path, selected);
    assert_eq!(redone.native_path, opened.native_path);
    assert_eq!(redone.snapshot.primary_analysis.unwrap().visits, 100);
    assert!(!redone.can_redo);
    assert_eq!(state.serialize().unwrap(), edited_sgf);
    let undone_again = state.undo(redone.generation).unwrap();
    assert_eq!(undone_again.snapshot.primary_analysis.unwrap().visits, 200);
    assert_eq!(state.serialize().unwrap(), original_sgf);
    let reopened = CurrentSgfDocument::open(&original_sgf).unwrap();
    assert_eq!(
        reopened
            .snapshot(&selected)
            .unwrap()
            .primary_analysis
            .unwrap()
            .visits,
        200
    );
}
