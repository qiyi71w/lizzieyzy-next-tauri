use app_model::{
    CurrentGameError, CurrentGameErrorKind, GameDto, MoveDto, MoveVertex, NodePath, PlayerColor, PositionDto,
    SelectedNodeSnapshotDto, SgfPropertyDto, SgfTreeNodeDto,
};

use crate::{
    apply_setup_properties, parse_sgf, parse_vertex, player_to_play, property_values, serialize_sgf_document,
    serialize_vertex, stones_from_board, to_core_color, to_core_vertex, to_game_dto, SgfDocument, SgfError,
    SgfNode, SgfProperty,
};
use go_core::{Board, RuleError};

#[derive(Debug, Clone)]
pub struct CurrentSgfDocument {
    document: SgfDocument,
}

impl CurrentSgfDocument {
    pub fn open(input: &str) -> Result<Self, CurrentGameError> {
        let document = parse_sgf(input)?;
        if document.root.is_none() {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::MalformedSgf,
                message: SgfError::Malformed.to_string(),
            });
        }
        Ok(Self { document })
    }

    pub fn replace(&mut self, input: &str) -> Result<(), CurrentGameError> {
        *self = Self::open(input)?;
        Ok(())
    }

    pub fn default_selected_path(&self) -> NodePath {
        let mut indices = Vec::new();
        let Ok(mut node) = self.root() else {
            return NodePath { indices };
        };
        while let Some(child) = node.children.first() {
            indices.push(0);
            node = child;
        }
        NodePath { indices }
    }

    pub fn snapshot(&self, path: &NodePath) -> Result<SelectedNodeSnapshotDto, CurrentGameError> {
        let nodes = self.nodes_on_path(path)?;
        let selected = *nodes.last().expect("path walk includes the root");
        Ok(SelectedNodeSnapshotDto {
            path: path.clone(),
            position: self.replay_nodes(&nodes)?,
            personal_comment: personal_comment(selected),
            generated_information: None,
        })
    }

    pub fn serialize(&self) -> Result<String, CurrentGameError> {
        Ok(serialize_sgf_document(&self.document)?)
    }

    pub fn mainline_projection(&self) -> GameDto {
        to_game_dto(self.document.clone())
    }

    pub fn komi(&self) -> f32 {
        self.document.komi
    }

    pub fn board_size(&self) -> u8 {
        self.document.board_size
    }

    pub fn tree(&self) -> Result<SgfTreeNodeDto, CurrentGameError> {
        Ok(tree_dto(self.root()?))
    }

    pub fn play(
        &mut self,
        path: &NodePath,
        vertex: MoveVertex,
    ) -> Result<SelectedNodeSnapshotDto, CurrentGameError> {
        let to_play = self.snapshot(path)?.position.to_play;
        if let Some(index) = self.existing_child_index(path, to_play, &vertex)? {
            let mut indices = path.indices.clone();
            indices.push(index);
            return self.snapshot(&NodePath { indices });
        }

        let mut board = self.board_after(path)?;
        board
            .play(to_core_color(to_play), to_core_vertex(&vertex))
            .map_err(rule_error)?;
        let encoded = serialize_vertex(&vertex, self.document.board_size)?;
        let index = {
            let parent = self.node_at_mut(path)?;
            parent.children.push(SgfNode {
                properties: vec![SgfProperty {
                    key: match to_play {
                        PlayerColor::Black => "B".to_string(),
                        PlayerColor::White => "W".to_string(),
                    },
                    values: vec![encoded],
                }],
                children: Vec::new(),
            });
            u32::try_from(parent.children.len() - 1).expect("child index fits u32")
        };
        let mut indices = path.indices.clone();
        indices.push(index);
        self.snapshot(&NodePath { indices })
    }

    pub fn set_personal_comment(
        &mut self,
        path: &NodePath,
        comment: &str,
    ) -> Result<SelectedNodeSnapshotDto, CurrentGameError> {
        apply_personal_comment(self.node_mut(path)?, comment);
        self.snapshot(path)
    }

    pub fn remove_variation(&mut self, path: &NodePath) -> Result<NodePath, CurrentGameError> {
        if path.indices.is_empty() {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::RootRemoval,
                message: "cannot remove the root".to_string(),
            });
        }
        let _ = self.nodes_on_path(path)?;
        let child_index = *path.indices.last().expect("non-root path") as usize;
        let parent_path = NodePath {
            indices: path.indices[..path.indices.len() - 1].to_vec(),
        };
        self.node_mut(&parent_path)?.children.remove(child_index);
        Ok(parent_path)
    }

    fn root(&self) -> Result<&SgfNode, CurrentGameError> {
        self.document.root.as_ref().ok_or_else(|| CurrentGameError {
            kind: CurrentGameErrorKind::MalformedSgf,
            message: SgfError::Malformed.to_string(),
        })
    }

    fn node_mut(&mut self, path: &NodePath) -> Result<&mut SgfNode, CurrentGameError> {
        let mut node = self.document.root.as_mut().ok_or_else(|| CurrentGameError {
            kind: CurrentGameErrorKind::MalformedSgf,
            message: SgfError::Malformed.to_string(),
        })?;
        for &index in &path.indices {
            node = node
                .children
                .get_mut(index as usize)
                .ok_or_else(|| CurrentGameError {
                    kind: CurrentGameErrorKind::InvalidNodePath,
                    message: "invalid node path".to_string(),
                })?;
        }
        Ok(node)
    }

    fn nodes_on_path(&self, path: &NodePath) -> Result<Vec<&SgfNode>, CurrentGameError> {
        let mut node = self.root()?;
        let mut nodes = vec![node];
        for &index in &path.indices {
            node = node
                .children
                .get(index as usize)
                .ok_or_else(|| CurrentGameError {
                    kind: CurrentGameErrorKind::InvalidNodePath,
                    message: "invalid node path".to_string(),
                })?;
            nodes.push(node);
        }
        Ok(nodes)
    }

    fn replay_nodes(&self, nodes: &[&SgfNode]) -> Result<PositionDto, CurrentGameError> {
        let mut board = Board::new(self.document.board_size).map_err(|_| CurrentGameError {
            kind: CurrentGameErrorKind::UnsupportedBoardSize,
            message: SgfError::UnsupportedBoardSize(self.document.board_size).to_string(),
        })?;
        let mut captures_black = 0u32;
        let mut captures_white = 0u32;
        let mut to_play = PlayerColor::Black;
        let mut last_move = None;
        let mut move_number = 0u32;
        let mut errors = Vec::new();

        for node in nodes {
            apply_setup_properties(&mut board, node, self.document.board_size)?;
            if let Some(color) = player_to_play(node)? {
                to_play = color;
            }
            for property in &node.properties {
                let color = match property.key.as_str() {
                    "B" => Some(PlayerColor::Black),
                    "W" => Some(PlayerColor::White),
                    _ => None,
                };
                let Some(color) = color else {
                    continue;
                };
                let raw = property.values.first().ok_or(SgfError::Malformed)?;
                move_number += 1;
                let sgf_move = MoveDto {
                    color,
                    vertex: parse_vertex(raw, self.document.board_size)?,
                    move_number,
                };
                match board.play(to_core_color(sgf_move.color), to_core_vertex(&sgf_move.vertex)) {
                    Ok(outcome) => match sgf_move.color {
                        PlayerColor::Black => captures_black += outcome.captured.len() as u32,
                        PlayerColor::White => captures_white += outcome.captured.len() as u32,
                    },
                    Err(error) => errors.push(format!("{error}")),
                }
                to_play = player_to_play(node)?.unwrap_or_else(|| sgf_move.color.opponent());
                last_move = Some(sgf_move);
            }
        }

        Ok(PositionDto {
            board_size: self.document.board_size,
            move_number,
            to_play,
            stones: stones_from_board(&board),
            captures_black,
            captures_white,
            last_move,
            errors,
        })
    }

    fn existing_child_index(
        &self,
        path: &NodePath,
        color: PlayerColor,
        vertex: &MoveVertex,
    ) -> Result<Option<u32>, CurrentGameError> {
        let node = *self
            .nodes_on_path(path)?
            .last()
            .expect("path walk includes the root");
        let key = match color {
            PlayerColor::Black => "B",
            PlayerColor::White => "W",
        };
        for (index, child) in node.children.iter().enumerate() {
            for property in &child.properties {
                if property.key != key {
                    continue;
                }
                let raw = property.values.first().ok_or(SgfError::Malformed)?;
                if &parse_vertex(raw, self.document.board_size)? == vertex {
                    return Ok(Some(u32::try_from(index).expect("child index fits u32")));
                }
            }
        }
        Ok(None)
    }

    fn board_after(&self, path: &NodePath) -> Result<Board, CurrentGameError> {
        let nodes = self.nodes_on_path(path)?;
        let mut board = Board::new(self.document.board_size).map_err(|_| CurrentGameError {
            kind: CurrentGameErrorKind::UnsupportedBoardSize,
            message: SgfError::UnsupportedBoardSize(self.document.board_size).to_string(),
        })?;
        for node in nodes {
            apply_setup_properties(&mut board, node, self.document.board_size)?;
            for property in &node.properties {
                let color = match property.key.as_str() {
                    "B" => Some(PlayerColor::Black),
                    "W" => Some(PlayerColor::White),
                    _ => None,
                };
                let Some(color) = color else {
                    continue;
                };
                let raw = property.values.first().ok_or(SgfError::Malformed)?;
                let played = parse_vertex(raw, self.document.board_size)?;
                let _ = board.play(to_core_color(color), to_core_vertex(&played));
            }
        }
        Ok(board)
    }

    fn node_at_mut(&mut self, path: &NodePath) -> Result<&mut SgfNode, CurrentGameError> {
        let mut node = self.document.root.as_mut().ok_or_else(|| CurrentGameError {
            kind: CurrentGameErrorKind::MalformedSgf,
            message: SgfError::Malformed.to_string(),
        })?;
        for &index in &path.indices {
            node = node
                .children
                .get_mut(index as usize)
                .ok_or_else(|| CurrentGameError {
                    kind: CurrentGameErrorKind::InvalidNodePath,
                    message: "invalid node path".to_string(),
                })?;
        }
        Ok(node)
    }
}

fn rule_error(error: RuleError) -> CurrentGameError {
    let kind = match error {
        RuleError::Occupied => CurrentGameErrorKind::OccupiedPoint,
        RuleError::Suicide => CurrentGameErrorKind::Suicide,
        RuleError::Ko => CurrentGameErrorKind::SimpleKo,
        RuleError::InvalidBoardSize => CurrentGameErrorKind::UnsupportedBoardSize,
        RuleError::OutOfBounds => CurrentGameErrorKind::OccupiedPoint,
    };
    CurrentGameError {
        kind,
        message: error.to_string(),
    }
}

impl From<SgfError> for CurrentGameError {
    fn from(error: SgfError) -> Self {
        match error {
            SgfError::UnsupportedBoardSize(_) => CurrentGameError {
                kind: CurrentGameErrorKind::UnsupportedBoardSize,
                message: error.to_string(),
            },
            SgfError::Empty | SgfError::Malformed => CurrentGameError {
                kind: CurrentGameErrorKind::MalformedSgf,
                message: error.to_string(),
            },
        }
    }
}

fn personal_comment(node: &SgfNode) -> String {
    property_values(node, "C")
        .and_then(|values| values.first())
        .cloned()
        .unwrap_or_default()
}

fn apply_personal_comment(node: &mut SgfNode, comment: &str) {
    let submitted_empty = comment.trim().is_empty();
    let current = property_values(node, "C")
        .and_then(|values| values.first())
        .cloned();
    let current_empty = current
        .as_deref()
        .map(str::trim)
        .map(str::is_empty)
        .unwrap_or(true);

    if submitted_empty {
        if current_empty {
            return;
        }
        node.properties.retain(|property| property.key != "C");
        return;
    }

    if current.as_deref() == Some(comment) {
        return;
    }

    if let Some(property) = node.properties.iter_mut().find(|property| property.key == "C") {
        property.values = vec![comment.to_string()];
        return;
    }

    node.properties.push(SgfProperty {
        key: "C".to_string(),
        values: vec![comment.to_string()],
    });
}

fn tree_dto(node: &SgfNode) -> SgfTreeNodeDto {
    SgfTreeNodeDto {
        properties: node
            .properties
            .iter()
            .map(|property| SgfPropertyDto {
                key: property.key.clone(),
                values: property.values.clone(),
            })
            .collect(),
        children: node.children.iter().map(tree_dto).collect(),
    }
}

#[cfg(test)]
mod editable_workspace_open {
    use super::*;
    use app_model::{CurrentGameErrorKind, MoveVertex, PlayerColor, PointDto, PositionDto};
    const BRANCHING: &str = include_str!("../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn editable_workspace_open_loads_complete_tree_and_default_mainline_snapshot() {
        let document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let path = document.default_selected_path();
        let snapshot = document.snapshot(&path).unwrap();
        let tree = document.tree().unwrap();

        assert_eq!(path.indices, vec![0, 0, 0]);
        assert_eq!(snapshot.path.indices, path.indices);
        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].children.len(), 2);
        assert!(tree
            .properties
            .iter()
            .any(|property| property.key == "XY" && property.values == ["keep-me"]));
        assert_eq!(
            tree.children[0].children[1]
                .properties
                .iter()
                .find(|property| property.key == "C")
                .map(|property| property.values.clone()),
            Some(vec!["second continuation".to_string()])
        );

        assert_eq!(snapshot.position.board_size, 5);
        assert_eq!(snapshot.position.move_number, 3);
        assert_eq!(snapshot.position.to_play, PlayerColor::Black);
        assert_eq!(snapshot.position.captures_black, 0);
        assert_eq!(snapshot.position.captures_white, 0);
        assert_eq!(snapshot.personal_comment, "mainline pass");
        assert!(snapshot.generated_information.is_none());
        let last_move = snapshot.position.last_move.as_ref().unwrap();
        assert_eq!(last_move.color, PlayerColor::White);
        assert_eq!(last_move.move_number, 3);
        assert!(matches!(last_move.vertex, MoveVertex::Pass));
        assert!(has_stone(&snapshot.position, 0, 0, PlayerColor::Black));
        assert!(has_stone(&snapshot.position, 2, 2, PlayerColor::White));
        assert!(has_stone(&snapshot.position, 3, 3, PlayerColor::White));
        assert!(has_stone(&snapshot.position, 4, 4, PlayerColor::Black));
        assert!(!snapshot
            .position
            .stones
            .iter()
            .any(|stone| stone.x == 1 && stone.y == 1));
        assert!(snapshot.position.errors.is_empty());
    }

    #[test]
    fn editable_workspace_open_preserves_document_on_malformed_and_unsupported() {
        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let before = document.serialize().unwrap();
        let path = document.default_selected_path();
        let snapshot = document.snapshot(&path).unwrap();

        let malformed = document.replace("not an sgf").unwrap_err();
        assert_eq!(malformed.kind, CurrentGameErrorKind::MalformedSgf);
        assert_eq!(document.serialize().unwrap(), before);
        assert_eq!(document.default_selected_path(), path);
        assert_eq!(document.snapshot(&path).unwrap(), snapshot);

        let unsupported = document.replace("(;SZ[99])").unwrap_err();
        assert_eq!(unsupported.kind, CurrentGameErrorKind::UnsupportedBoardSize);
        assert_eq!(document.serialize().unwrap(), before);
        assert_eq!(
            document.snapshot(&path).unwrap().personal_comment,
            "mainline pass"
        );
    }

    #[test]
    fn editable_workspace_open_serializes_and_projects_mainline() {
        let document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let serialized = document.serialize().unwrap();
        let reopened = CurrentSgfDocument::open(&serialized).unwrap();
        let tree = reopened.tree().unwrap();
        let projection = document.mainline_projection();

        assert!(tree
            .properties
            .iter()
            .any(|property| property.key == "XY" && property.values == ["keep-me"]));
        assert_eq!(tree.children[0].children.len(), 2);
        assert_eq!(projection.moves.len(), 3);
        assert_eq!(projection.moves[0].color, PlayerColor::White);
        assert_eq!(
            projection.moves[0].vertex,
            MoveVertex::Point(PointDto { x: 3, y: 3 })
        );
        assert_eq!(
            projection.moves[1].vertex,
            MoveVertex::Point(PointDto { x: 4, y: 4 })
        );
        assert!(matches!(projection.moves[2].vertex, MoveVertex::Pass));
        assert_eq!(reopened.default_selected_path().indices, vec![0, 0, 0]);
        assert_eq!(
            reopened
                .snapshot(&reopened.default_selected_path())
                .unwrap()
                .personal_comment,
            "mainline pass"
        );
    }

    fn has_stone(position: &PositionDto, x: u8, y: u8, color: PlayerColor) -> bool {
        position
            .stones
            .iter()
            .any(|stone| stone.x == x && stone.y == y && stone.color == color)
    }
}

#[cfg(test)]
mod editable_workspace_navigation {
    use super::*;
    use app_model::{CurrentGameErrorKind, MoveVertex, PlayerColor, PointDto};

    const BRANCHING: &str = include_str!("../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn editable_workspace_navigation_snapshots_root_parent_siblings_and_rejects_invalid_paths() {
        let document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let before = document.serialize().unwrap();
        let tree = document.tree().unwrap();
        let root = NodePath { indices: Vec::new() };
        let parent = NodePath { indices: vec![0] };
        let first_sibling = NodePath { indices: vec![0, 0] };
        let second_sibling = NodePath { indices: vec![0, 1] };
        let mainline_leaf = NodePath {
            indices: vec![0, 0, 0],
        };

        assert_eq!(tree.children.len(), 1);
        assert_eq!(tree.children[0].children.len(), 2);
        assert_eq!(comment(&tree.children[0].children[0]), Some("first continuation"));
        assert_eq!(
            comment(&tree.children[0].children[1]),
            Some("second continuation")
        );

        let root_snapshot = document.snapshot(&root).unwrap();
        assert_eq!(root_snapshot.path.indices, root.indices);
        assert_eq!(root_snapshot.personal_comment, "root personal");
        assert_eq!(root_snapshot.position.board_size, 5);
        assert_eq!(root_snapshot.position.move_number, 0);
        assert_eq!(root_snapshot.position.to_play, PlayerColor::White);
        assert_eq!(root_snapshot.position.captures_black, 0);
        assert_eq!(root_snapshot.position.captures_white, 0);
        assert!(root_snapshot.position.last_move.is_none());
        assert!(has_stone(&root_snapshot.position, 0, 0, PlayerColor::Black));
        assert!(has_stone(&root_snapshot.position, 2, 2, PlayerColor::White));
        assert!(!root_snapshot
            .position
            .stones
            .iter()
            .any(|stone| stone.x == 1 && stone.y == 1));

        let parent_snapshot = document.snapshot(&parent).unwrap();
        assert_eq!(parent_snapshot.personal_comment, "main move");
        assert_eq!(parent_snapshot.position.move_number, 1);
        assert_eq!(parent_snapshot.position.to_play, PlayerColor::Black);
        let parent_last = parent_snapshot.position.last_move.as_ref().unwrap();
        assert_eq!(parent_last.color, PlayerColor::White);
        assert_eq!(parent_last.move_number, 1);
        assert_eq!(parent_last.vertex, MoveVertex::Point(PointDto { x: 3, y: 3 }));
        assert!(has_stone(&parent_snapshot.position, 3, 3, PlayerColor::White));

        let first_snapshot = document.snapshot(&first_sibling).unwrap();
        assert_eq!(first_snapshot.personal_comment, "first continuation");
        assert_eq!(first_snapshot.position.move_number, 2);
        assert_eq!(first_snapshot.position.to_play, PlayerColor::White);
        assert_eq!(first_snapshot.position.captures_black, 0);
        assert_eq!(first_snapshot.position.captures_white, 0);
        let first_last = first_snapshot.position.last_move.as_ref().unwrap();
        assert_eq!(first_last.color, PlayerColor::Black);
        assert_eq!(first_last.vertex, MoveVertex::Point(PointDto { x: 4, y: 4 }));
        assert!(has_stone(&first_snapshot.position, 4, 4, PlayerColor::Black));
        assert!(!has_stone(&first_snapshot.position, 0, 3, PlayerColor::Black));

        let second_snapshot = document.snapshot(&second_sibling).unwrap();
        assert_eq!(second_snapshot.personal_comment, "second continuation");
        assert_eq!(second_snapshot.position.move_number, 2);
        assert_eq!(second_snapshot.position.to_play, PlayerColor::White);
        assert_eq!(second_snapshot.position.captures_black, 0);
        assert_eq!(second_snapshot.position.captures_white, 0);
        let second_last = second_snapshot.position.last_move.as_ref().unwrap();
        assert_eq!(second_last.color, PlayerColor::Black);
        assert_eq!(second_last.vertex, MoveVertex::Point(PointDto { x: 0, y: 3 }));
        assert!(has_stone(&second_snapshot.position, 0, 3, PlayerColor::Black));
        assert!(!has_stone(&second_snapshot.position, 4, 4, PlayerColor::Black));

        let leaf_snapshot = document.snapshot(&mainline_leaf).unwrap();
        assert_eq!(leaf_snapshot.personal_comment, "mainline pass");
        assert_eq!(leaf_snapshot.position.move_number, 3);
        assert_eq!(leaf_snapshot.position.to_play, PlayerColor::Black);
        assert!(matches!(
            leaf_snapshot.position.last_move.as_ref().unwrap().vertex,
            MoveVertex::Pass
        ));

        for path in [
            NodePath { indices: vec![9] },
            NodePath { indices: vec![0, 2] },
            NodePath {
                indices: vec![0, 0, 0, 0],
            },
        ] {
            let error = document.snapshot(&path).unwrap_err();
            assert_eq!(error.kind, CurrentGameErrorKind::InvalidNodePath);
            assert_eq!(document.serialize().unwrap(), before);
            assert_eq!(
                document.snapshot(&mainline_leaf).unwrap().personal_comment,
                "mainline pass"
            );
        }

        assert_eq!(document.serialize().unwrap(), before);
        assert_eq!(document.default_selected_path(), mainline_leaf);
    }

    fn comment(node: &SgfTreeNodeDto) -> Option<&str> {
        node.properties
            .iter()
            .find(|property| property.key == "C")
            .and_then(|property| property.values.first())
            .map(String::as_str)
    }

    fn has_stone(position: &PositionDto, x: u8, y: u8, color: PlayerColor) -> bool {
        position
            .stones
            .iter()
            .any(|stone| stone.x == x && stone.y == y && stone.color == color)
    }
}

#[cfg(test)]
mod editable_workspace_mutation_edit {
    use super::*;
    use app_model::{CurrentGameErrorKind, MoveVertex, PlayerColor, PointDto, PositionDto};

    const BRANCHING: &str = include_str!("../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn editable_workspace_move_edit_adds_legal_black_point_from_parent() {
        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let parent = NodePath { indices: vec![0] };

        let snapshot = document
            .play(&parent, MoveVertex::Point(PointDto { x: 1, y: 1 }))
            .unwrap();

        assert_eq!(snapshot.path.indices, vec![0, 2]);
        assert_eq!(snapshot.personal_comment, "");
        assert_eq!(snapshot.position.move_number, 2);
        assert_eq!(snapshot.position.to_play, PlayerColor::White);
        assert_eq!(snapshot.position.captures_black, 0);
        assert_eq!(snapshot.position.captures_white, 0);
        let last = snapshot.position.last_move.as_ref().unwrap();
        assert_eq!(last.color, PlayerColor::Black);
        assert_eq!(last.move_number, 2);
        assert_eq!(last.vertex, MoveVertex::Point(PointDto { x: 1, y: 1 }));
        assert!(has_stone(&snapshot.position, 1, 1, PlayerColor::Black));
        assert!(has_stone(&snapshot.position, 0, 0, PlayerColor::Black));
        assert!(has_stone(&snapshot.position, 2, 2, PlayerColor::White));
        assert!(has_stone(&snapshot.position, 3, 3, PlayerColor::White));
        assert!(!has_stone(&snapshot.position, 4, 4, PlayerColor::Black));

        let tree = document.tree().unwrap();
        assert_eq!(tree.children[0].children.len(), 3);
        assert_eq!(comment(&tree.children[0].children[0]), Some("first continuation"));
        assert_eq!(
            comment(&tree.children[0].children[1]),
            Some("second continuation")
        );
        assert_eq!(
            tree.children[0].children[2]
                .properties
                .iter()
                .find(|property| property.key == "B")
                .map(|property| property.values.clone()),
            Some(vec!["bb".to_string()])
        );

        let reopened = CurrentSgfDocument::open(&document.serialize().unwrap()).unwrap();
        assert_eq!(reopened.tree().unwrap().children[0].children.len(), 3);
        assert_eq!(
            reopened
                .snapshot(&NodePath { indices: vec![0, 2] })
                .unwrap()
                .position
                .last_move
                .unwrap()
                .vertex,
            MoveVertex::Point(PointDto { x: 1, y: 1 })
        );
    }

    #[test]
    fn editable_workspace_move_edit_pass_creates_child_and_switches_to_play() {
        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let second = NodePath { indices: vec![0, 1] };

        let snapshot = document.play(&second, MoveVertex::Pass).unwrap();

        assert_eq!(snapshot.path.indices, vec![0, 1, 0]);
        assert_eq!(snapshot.position.move_number, 3);
        assert_eq!(snapshot.position.to_play, PlayerColor::Black);
        assert_eq!(
            snapshot.position.last_move.as_ref().unwrap().color,
            PlayerColor::White
        );
        assert!(matches!(
            snapshot.position.last_move.as_ref().unwrap().vertex,
            MoveVertex::Pass
        ));
        assert_eq!(document.tree().unwrap().children[0].children[1].children.len(), 1);
        assert_eq!(document.tree().unwrap().children[0].children[0].children.len(), 1);
    }

    #[test]
    fn editable_workspace_move_edit_selects_existing_child_without_duplication() {
        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let parent = NodePath { indices: vec![0] };
        let first = NodePath { indices: vec![0, 0] };
        let before = document.serialize().unwrap();

        let existing_point = document
            .play(&parent, MoveVertex::Point(PointDto { x: 4, y: 4 }))
            .unwrap();
        assert_eq!(existing_point.path.indices, vec![0, 0]);
        assert_eq!(existing_point.personal_comment, "first continuation");
        assert_eq!(document.serialize().unwrap(), before);
        assert_eq!(document.tree().unwrap().children[0].children.len(), 2);

        let existing_pass = document.play(&first, MoveVertex::Pass).unwrap();
        assert_eq!(existing_pass.path.indices, vec![0, 0, 0]);
        assert_eq!(existing_pass.personal_comment, "mainline pass");
        assert_eq!(document.serialize().unwrap(), before);
    }

    #[test]
    fn editable_workspace_move_edit_captures_and_rejects_occupied_suicide_ko_and_invalid_path() {
        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let root = NodePath { indices: Vec::new() };
        let parent = NodePath { indices: vec![0] };

        let captured = play_sequence(
            &mut document,
            &root,
            &[
                MoveVertex::Point(PointDto { x: 1, y: 0 }),
                MoveVertex::Pass,
                MoveVertex::Point(PointDto { x: 0, y: 1 }),
            ],
        );
        assert_eq!(captured.position.captures_white, 1);
        assert_eq!(captured.position.captures_black, 0);
        assert!(!has_stone(&captured.position, 0, 0, PlayerColor::Black));
        assert!(has_stone(&captured.position, 1, 0, PlayerColor::White));
        assert!(has_stone(&captured.position, 0, 1, PlayerColor::White));

        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let before = document.serialize().unwrap();
        let occupied = document
            .play(&parent, MoveVertex::Point(PointDto { x: 3, y: 3 }))
            .unwrap_err();
        assert_eq!(occupied.kind, CurrentGameErrorKind::OccupiedPoint);
        assert_eq!(document.serialize().unwrap(), before);
        assert_eq!(document.default_selected_path().indices, vec![0, 0, 0]);

        let invalid = document
            .play(&NodePath { indices: vec![9] }, MoveVertex::Pass)
            .unwrap_err();
        assert_eq!(invalid.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(document.serialize().unwrap(), before);

        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let suicide_setup = play_sequence(
            &mut document,
            &root,
            &[
                MoveVertex::Point(PointDto { x: 3, y: 0 }),
                MoveVertex::Pass,
                MoveVertex::Point(PointDto { x: 2, y: 1 }),
                MoveVertex::Pass,
                MoveVertex::Point(PointDto { x: 4, y: 1 }),
                MoveVertex::Pass,
                MoveVertex::Point(PointDto { x: 3, y: 2 }),
            ],
        );
        let before_suicide = document.serialize().unwrap();
        let suicide = document
            .play(&suicide_setup.path, MoveVertex::Point(PointDto { x: 3, y: 1 }))
            .unwrap_err();
        assert_eq!(suicide.kind, CurrentGameErrorKind::Suicide);
        assert_eq!(document.serialize().unwrap(), before_suicide);

        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let after_capture = play_sequence(
            &mut document,
            &root,
            &[
                MoveVertex::Point(PointDto { x: 2, y: 0 }),
                MoveVertex::Point(PointDto { x: 1, y: 0 }),
                MoveVertex::Point(PointDto { x: 3, y: 1 }),
                MoveVertex::Point(PointDto { x: 0, y: 1 }),
                MoveVertex::Pass,
                MoveVertex::Point(PointDto { x: 2, y: 1 }),
                MoveVertex::Pass,
                MoveVertex::Point(PointDto { x: 1, y: 2 }),
                MoveVertex::Point(PointDto { x: 1, y: 1 }),
            ],
        );
        assert_eq!(after_capture.position.captures_white, 1);
        assert!(!has_stone(&after_capture.position, 2, 1, PlayerColor::Black));
        let before_ko = document.serialize().unwrap();
        let ko = document
            .play(&after_capture.path, MoveVertex::Point(PointDto { x: 2, y: 1 }))
            .unwrap_err();
        assert_eq!(ko.kind, CurrentGameErrorKind::SimpleKo);
        assert_eq!(document.serialize().unwrap(), before_ko);
    }

    fn play_sequence(
        document: &mut CurrentSgfDocument,
        start: &NodePath,
        moves: &[MoveVertex],
    ) -> SelectedNodeSnapshotDto {
        let mut path = start.clone();
        let mut snapshot = document.snapshot(&path).unwrap();
        for vertex in moves {
            snapshot = document.play(&path, vertex.clone()).unwrap();
            path = snapshot.path.clone();
        }
        snapshot
    }

    #[test]
    fn editable_workspace_comment_edit_adds_replaces_clears_and_preserves_empty() {
        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let root = NodePath { indices: Vec::new() };
        let move_node = NodePath { indices: vec![0] };
        let first_sibling = NodePath { indices: vec![0, 0] };
        let second_sibling = NodePath { indices: vec![0, 1] };

        assert_eq!(
            document.snapshot(&root).unwrap().personal_comment,
            "root personal"
        );
        assert_eq!(
            document.snapshot(&move_node).unwrap().personal_comment,
            "main move"
        );
        assert_eq!(
            document.snapshot(&first_sibling).unwrap().personal_comment,
            "first continuation"
        );
        assert_eq!(
            document.snapshot(&second_sibling).unwrap().personal_comment,
            "second continuation"
        );

        let replaced_root = document.set_personal_comment(&root, "root edited").unwrap();
        assert_eq!(replaced_root.path.indices, root.indices);
        assert_eq!(replaced_root.personal_comment, "root edited");
        assert!(replaced_root.generated_information.is_none());
        assert_eq!(
            document.snapshot(&move_node).unwrap().personal_comment,
            "main move"
        );

        let replaced_move = document.set_personal_comment(&move_node, "move edited").unwrap();
        assert_eq!(replaced_move.path.indices, move_node.indices);
        assert_eq!(replaced_move.personal_comment, "move edited");

        let replaced_sibling = document
            .set_personal_comment(&second_sibling, "sibling edited")
            .unwrap();
        assert_eq!(replaced_sibling.path.indices, second_sibling.indices);
        assert_eq!(replaced_sibling.personal_comment, "sibling edited");
        assert_eq!(
            document.snapshot(&first_sibling).unwrap().personal_comment,
            "first continuation"
        );

        let after_edits = document.serialize().unwrap();
        let unchanged = document
            .set_personal_comment(&second_sibling, "sibling edited")
            .unwrap();
        assert_eq!(unchanged.personal_comment, "sibling edited");
        assert_eq!(document.serialize().unwrap(), after_edits);

        let cleared = document.set_personal_comment(&first_sibling, "").unwrap();
        assert_eq!(cleared.path.indices, first_sibling.indices);
        assert_eq!(cleared.personal_comment, "");
        assert!(comment(&document.tree().unwrap().children[0].children[0]).is_none());

        let after_clear = document.serialize().unwrap();
        let empty_noop = document.set_personal_comment(&first_sibling, "   ").unwrap();
        assert_eq!(empty_noop.personal_comment, "");
        assert_eq!(document.serialize().unwrap(), after_clear);

        let added = document
            .set_personal_comment(&first_sibling, "first added")
            .unwrap();
        assert_eq!(added.personal_comment, "first added");
        assert_eq!(
            comment(&document.tree().unwrap().children[0].children[0]),
            Some("first added")
        );

        let serialized = document.serialize().unwrap();
        let reopened = CurrentSgfDocument::open(&serialized).unwrap();
        let reopened_root = reopened.snapshot(&root).unwrap();
        assert_eq!(reopened_root.personal_comment, "root edited");
        assert!(reopened_root.generated_information.is_none());
        assert_eq!(
            reopened.snapshot(&move_node).unwrap().personal_comment,
            "move edited"
        );
        assert_eq!(
            reopened.snapshot(&first_sibling).unwrap().personal_comment,
            "first added"
        );
        assert_eq!(
            reopened.snapshot(&second_sibling).unwrap().personal_comment,
            "sibling edited"
        );
        assert!(reopened
            .tree()
            .unwrap()
            .properties
            .iter()
            .any(|property| property.key == "XY" && property.values == ["keep-me"]));

        let before_invalid = document.serialize().unwrap();
        let error = document
            .set_personal_comment(&NodePath { indices: vec![9] }, "nope")
            .unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(document.serialize().unwrap(), before_invalid);

        let mut empty_comment_doc = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[5]C[])").unwrap();
        let empty_root = NodePath { indices: Vec::new() };
        assert_eq!(
            empty_comment_doc.snapshot(&empty_root).unwrap().personal_comment,
            ""
        );
        let preserved = empty_comment_doc.serialize().unwrap();
        assert!(comment(&empty_comment_doc.tree().unwrap()).is_some());
        let empty_submit = empty_comment_doc.set_personal_comment(&empty_root, "").unwrap();
        assert_eq!(empty_submit.personal_comment, "");
        assert_eq!(empty_comment_doc.serialize().unwrap(), preserved);
        assert_eq!(comment(&empty_comment_doc.tree().unwrap()), Some(""));

        let filled = empty_comment_doc
            .set_personal_comment(&empty_root, "now filled")
            .unwrap();
        assert_eq!(filled.personal_comment, "now filled");
        let filled_reopened = CurrentSgfDocument::open(&empty_comment_doc.serialize().unwrap()).unwrap();
        assert_eq!(
            filled_reopened.snapshot(&empty_root).unwrap().personal_comment,
            "now filled"
        );
        assert!(filled_reopened
            .snapshot(&empty_root)
            .unwrap()
            .generated_information
            .is_none());
    }

    #[test]
    fn editable_workspace_remove_variation_deletes_named_second_sibling_and_selects_parent() {
        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let parent = NodePath { indices: vec![0] };
        let first_sibling = NodePath { indices: vec![0, 0] };
        let second_sibling = NodePath { indices: vec![0, 1] };
        let parent_before = document.snapshot(&parent).unwrap();

        let selected = document.remove_variation(&second_sibling).unwrap();
        let snapshot = document.snapshot(&selected).unwrap();
        let tree = document.tree().unwrap();

        assert_eq!(selected.indices, parent.indices);
        assert_eq!(snapshot.path.indices, parent.indices);
        assert_eq!(snapshot.personal_comment, "main move");
        assert_eq!(snapshot.position.to_play, PlayerColor::Black);
        assert_eq!(snapshot.position.move_number, 1);
        let last_move = snapshot.position.last_move.as_ref().unwrap();
        assert_eq!(last_move.color, PlayerColor::White);
        assert_eq!(last_move.vertex, MoveVertex::Point(PointDto { x: 3, y: 3 }));
        assert_eq!(snapshot.position.stones, parent_before.position.stones);
        assert_eq!(tree.children[0].children.len(), 1);
        assert_eq!(comment(&tree.children[0].children[0]), Some("first continuation"));
        assert!(has_unknown_property(&tree));
        assert_eq!(
            document.snapshot(&first_sibling).unwrap().personal_comment,
            "first continuation"
        );
        assert_eq!(
            document.snapshot(&second_sibling).unwrap_err().kind,
            CurrentGameErrorKind::InvalidNodePath
        );
    }

    #[test]
    fn editable_workspace_remove_variation_serializes_leaf_and_subtree_without_removed_nodes() {
        let mut leaf_document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let pass = NodePath {
            indices: vec![0, 0, 0],
        };
        leaf_document.remove_variation(&pass).unwrap();
        let leaf_reopened = CurrentSgfDocument::open(&leaf_document.serialize().unwrap()).unwrap();
        let leaf_tree = leaf_reopened.tree().unwrap();
        assert!(has_unknown_property(&leaf_tree));
        assert_eq!(leaf_tree.children[0].children.len(), 2);
        assert_eq!(
            comment(&leaf_tree.children[0].children[0]),
            Some("first continuation")
        );
        assert_eq!(
            comment(&leaf_tree.children[0].children[1]),
            Some("second continuation")
        );
        assert!(leaf_tree.children[0].children[0].children.is_empty());
        assert!(serialized_omits(&leaf_document, "mainline pass"));
        assert!(serialized_contains(&leaf_document, "first continuation"));
        assert!(serialized_contains(&leaf_document, "second continuation"));

        let mut subtree_document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let first_sibling = NodePath { indices: vec![0, 0] };
        subtree_document.remove_variation(&first_sibling).unwrap();
        let subtree_reopened = CurrentSgfDocument::open(&subtree_document.serialize().unwrap()).unwrap();
        let subtree_tree = subtree_reopened.tree().unwrap();
        assert!(has_unknown_property(&subtree_tree));
        assert_eq!(subtree_tree.children[0].children.len(), 1);
        assert_eq!(
            comment(&subtree_tree.children[0].children[0]),
            Some("second continuation")
        );
        assert!(serialized_omits(&subtree_document, "first continuation"));
        assert!(serialized_omits(&subtree_document, "mainline pass"));
        assert!(serialized_contains(&subtree_document, "second continuation"));
    }

    #[test]
    fn editable_workspace_remove_variation_rejects_root_and_invalid_paths_atomically() {
        let mut document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let before = document.serialize().unwrap();
        let mainline = document.default_selected_path();
        let snapshot = document.snapshot(&mainline).unwrap();
        let tree = document.tree().unwrap();
        let projection = document.mainline_projection();

        let root = document
            .remove_variation(&NodePath { indices: Vec::new() })
            .unwrap_err();
        assert_eq!(root.kind, CurrentGameErrorKind::RootRemoval);
        assert_eq!(document.serialize().unwrap(), before);
        assert_eq!(document.default_selected_path(), mainline);
        assert_eq!(document.snapshot(&mainline).unwrap(), snapshot);
        assert_eq!(document.tree().unwrap(), tree);
        assert_eq!(document.mainline_projection().moves.len(), projection.moves.len());

        for path in [
            NodePath { indices: vec![9] },
            NodePath { indices: vec![0, 2] },
            NodePath {
                indices: vec![0, 0, 0, 0],
            },
        ] {
            let error = document.remove_variation(&path).unwrap_err();
            assert_eq!(error.kind, CurrentGameErrorKind::InvalidNodePath);
            assert_eq!(document.serialize().unwrap(), before);
            assert_eq!(document.snapshot(&mainline).unwrap(), snapshot);
            assert_eq!(document.tree().unwrap(), tree);
        }
    }

    fn comment(node: &SgfTreeNodeDto) -> Option<&str> {
        node.properties
            .iter()
            .find(|property| property.key == "C")
            .and_then(|property| property.values.first())
            .map(String::as_str)
    }

    fn has_stone(position: &PositionDto, x: u8, y: u8, color: PlayerColor) -> bool {
        position
            .stones
            .iter()
            .any(|stone| stone.x == x && stone.y == y && stone.color == color)
    }

    fn has_unknown_property(node: &SgfTreeNodeDto) -> bool {
        node.properties
            .iter()
            .any(|property| property.key == "XY" && property.values == ["keep-me"])
    }

    fn serialized_contains(document: &CurrentSgfDocument, needle: &str) -> bool {
        document.serialize().unwrap().contains(needle)
    }

    fn serialized_omits(document: &CurrentSgfDocument, needle: &str) -> bool {
        !serialized_contains(document, needle)
    }
}
