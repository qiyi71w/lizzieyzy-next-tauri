use super::*;
use app_model::{PointDto, PointSearchScopeDto};

#[test]
fn point_search_is_nonmutating_generation_and_cursor_fenced_then_uses_exact_selection() {
    let state = CurrentGameState::default();
    let opened = state
        .replace(
            "(;SZ[19](;B[aa];W[bb])(;B[aa];W[cc])(;B[dd];W[ee]))",
            Some("source.sgf".into()),
        )
        .unwrap();
    let path = NodePath { indices: vec![2] };
    let selected = state.select_path(path.clone(), opened.generation).unwrap();
    let source = state.serialize().unwrap();
    let point = PointDto { x: 0, y: 0 };
    let full = PointSearchScopeDto::AllBranches;
    let found = state
        .find_recorded_point(path.clone(), selected.generation, point, vec![], full)
        .unwrap()
        .unwrap();
    assert_eq!(found.indices, vec![0]);
    assert_eq!(
        state.select_path(path.clone(), selected.generation).unwrap(),
        selected
    );
    assert_eq!(
        state
            .find_recorded_point(
                path.clone(),
                selected.generation,
                point,
                vec![],
                PointSearchScopeDto::CurrentLine
            )
            .unwrap(),
        None
    );
    for missing in [PointDto { x: 5, y: 5 }, PointDto { x: 19, y: 0 }] {
        assert_eq!(
            state
                .find_recorded_point(path.clone(), selected.generation, missing, vec![], full)
                .unwrap(),
            None
        );
    }
    assert!(state
        .find_recorded_point(path.clone(), selected.generation + 1, point, vec![], full)
        .is_err());
    assert!(state
        .find_recorded_point(NodePath::default(), selected.generation, point, vec![], full)
        .is_err());
    assert_eq!(
        state.select_path(path.clone(), selected.generation).unwrap(),
        selected
    );
    let navigated = state.select_path(found.clone(), selected.generation).unwrap();
    assert_eq!(navigated.selected_path, found);
    assert_eq!(navigated.generation, selected.generation);
    assert_eq!(navigated.dirty, selected.dirty);
    assert_eq!(navigated.native_path, selected.native_path);
    assert_eq!(state.serialize().unwrap(), source);
    state.enter_trial().unwrap();
    assert!(state
        .find_recorded_point(navigated.selected_path, selected.generation, point, vec![], full)
        .is_err());
}
