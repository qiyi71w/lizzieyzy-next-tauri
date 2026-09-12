use app_model::{
    AnalysisBranchChoiceDto, AnalysisPositionIntervalDto, AnalysisScopeDto, AnalysisScopeModeDto, NodePath,
    PlayerColor,
};
use sgf::CurrentSgfDocument;

fn scope(mode: AnalysisScopeModeDto) -> AnalysisScopeDto {
    AnalysisScopeDto {
        mode,
        current_node: NodePath { indices: vec![0, 0] },
        branch_choices: vec![],
        interval: None,
        to_play: None,
    }
}

const TREE: &str = "(;SZ[5];B[aa]MN[90];C[note](;W[];AB[cc]PL[W];W[dd])(;W[];AB[cc]PL[W];W[ee]))";

#[test]
fn scope_preserves_position_semantics_and_equal_board_nodes() {
    let document = CurrentSgfDocument::open(TREE).unwrap();
    let targets = document
        .analysis_scope_snapshots(&scope(AnalysisScopeModeDto::AllBranches))
        .unwrap();
    let paths: Vec<_> = targets.iter().map(|node| node.path.indices.clone()).collect();
    assert_eq!(
        paths,
        vec![
            vec![],
            vec![0],
            vec![0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0, 0],
            vec![0, 0, 1],
            vec![0, 0, 1, 0],
            vec![0, 0, 1, 0, 0]
        ]
    );
    assert_eq!(
        targets
            .iter()
            .map(|node| node.position.move_number)
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 2, 3, 2, 2, 3]
    );
    assert_eq!(targets[2].position, targets[5].position);
    let current = document
        .analysis_scope_snapshots(&scope(AnalysisScopeModeDto::CurrentNode))
        .unwrap();
    assert_eq!(current[0].personal_comment, "note");
    assert_eq!(current[0].position.move_number, 1);
}

#[test]
fn remembered_line_and_inclusive_interval_use_actual_moves_and_side_to_play() {
    let document = CurrentSgfDocument::open(TREE).unwrap();
    let mut request = scope(AnalysisScopeModeDto::SelectedReviewLine);
    request.branch_choices.push(AnalysisBranchChoiceDto {
        parent: NodePath { indices: vec![0, 0] },
        child: 1,
    });
    request.interval = Some(AnalysisPositionIntervalDto { start: 2, end: 2 });
    let targets = document.analysis_scope_snapshots(&request).unwrap();
    assert_eq!(
        targets
            .iter()
            .map(|node| node.path.indices.clone())
            .collect::<Vec<_>>(),
        vec![vec![0, 0, 1], vec![0, 0, 1, 0]]
    );
    request.to_play = Some(PlayerColor::White);
    let targets = document.analysis_scope_snapshots(&request).unwrap();
    assert_eq!(
        targets
            .iter()
            .map(|node| node.path.indices.clone())
            .collect::<Vec<_>>(),
        vec![vec![0, 0, 1, 0]]
    );
    request.mode = AnalysisScopeModeDto::FirstChildMainline;
    let targets = document.analysis_scope_snapshots(&request).unwrap();
    assert_eq!(targets[0].path.indices, vec![0, 0, 0, 0]);
    request.interval = Some(AnalysisPositionIntervalDto { start: 0, end: 0 });
    request.to_play = None;
    let targets = document.analysis_scope_snapshots(&request).unwrap();
    assert_eq!(
        targets
            .iter()
            .map(|node| node.path.indices.clone())
            .collect::<Vec<_>>(),
        vec![Vec::<u32>::new()]
    );
}

#[test]
fn invalid_or_empty_scope_never_becomes_another_request() {
    let document = CurrentSgfDocument::open(TREE).unwrap();
    let mut request = scope(AnalysisScopeModeDto::AllBranches);
    request.interval = Some(AnalysisPositionIntervalDto { start: 3, end: 2 });
    assert!(document.analysis_scope_snapshots(&request).is_err());
    request.interval = Some(AnalysisPositionIntervalDto { start: 0, end: 4 });
    assert!(document.analysis_scope_snapshots(&request).is_err());
    request.interval = Some(AnalysisPositionIntervalDto { start: 0, end: 0 });
    request.to_play = Some(PlayerColor::White);
    assert!(document.analysis_scope_snapshots(&request).is_err());
    request.interval = None;
    request.to_play = None;
    request.mode = AnalysisScopeModeDto::SelectedReviewLine;
    request.branch_choices.push(AnalysisBranchChoiceDto {
        parent: NodePath { indices: vec![0, 0] },
        child: 2,
    });
    assert!(document.analysis_scope_snapshots(&request).is_err());
}
