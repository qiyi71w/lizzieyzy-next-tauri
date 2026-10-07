use super::*;
use app_model::{PointDto, SgfAuthoringActionDto as Action, SgfTransformDto};

fn point(x: u8, y: u8) -> PointDto {
    PointDto { x, y }
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
    assert_eq!(reopened.snapshot(&selected).unwrap().primary_analysis.unwrap().visits, 200);
}
