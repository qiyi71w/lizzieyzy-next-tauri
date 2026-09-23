use app_model::{
    AnalysisMoveActorFilterDto, AnalysisSwingComparisonDto, CurrentGameError, CurrentGameErrorKind, GameDto,
    MoveDto, MoveVertex, NodePath, PlayerColor, PositionDto, SelectedNodeSnapshotDto, SgfPropertyDto,
    SgfTreeNodeDto, StoneDto,
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

const DOCUMENT_HISTORY_LIMIT: usize = 100;

#[derive(Debug, Default)]
pub struct DocumentHistory {
    undo: Vec<SgfDocumentEdit>,
    redo: Vec<SgfDocumentEdit>,
}

#[derive(Debug)]
pub struct SgfDocumentEdit {
    reversal: DocumentReversal,
    selected_before: NodePath,
    selected_after: NodePath,
}

#[derive(Debug)]
enum DocumentReversal {
    SetComment {
        path: NodePath,
        properties: Vec<(usize, SgfProperty)>,
    },
    SetMetadata {
        properties: Vec<(usize, SgfProperty)>,
        komi: f32,
        analysis: Option<Vec<(NodePath, Vec<(usize, SgfProperty)>)>>,
    },
    RemoveSubtree {
        parent: NodePath,
        index: usize,
    },
    InsertSubtree {
        parent: NodePath,
        index: usize,
        subtree: SgfNode,
    },
    PromoteMainline {
        moves: Vec<(NodePath, usize)>,
        to_front: bool,
    },
    RootSetup {
        properties: Vec<(usize, SgfProperty)>,
        children: Option<Vec<SgfNode>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentHistoryOutcome {
    pub selected_path: NodePath,
    pub structural: bool,
}

#[derive(Debug)]
pub struct DocumentEditOutcome {
    pub snapshot: SelectedNodeSnapshotDto,
    pub edit: Option<SgfDocumentEdit>,
}

impl DocumentHistory {
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    pub fn commit(&mut self, edit: SgfDocumentEdit) {
        self.redo.clear();
        self.undo.push(edit);
        if self.undo.len() > DOCUMENT_HISTORY_LIMIT {
            self.undo.remove(0);
        }
    }

    pub fn undo(
        &mut self,
        document: &mut CurrentSgfDocument,
    ) -> Result<Option<DocumentHistoryOutcome>, CurrentGameError> {
        let Some(mut edit) = self.undo.pop() else {
            return Ok(None);
        };
        let structural = match document.apply_reversal(&mut edit.reversal) {
            Ok(structural) => structural,
            Err(error) => {
                self.undo.push(edit);
                return Err(error);
            }
        };
        let selected_path = edit.selected_before.clone();
        self.redo.push(edit);
        Ok(Some(DocumentHistoryOutcome {
            selected_path,
            structural,
        }))
    }

    pub fn redo(
        &mut self,
        document: &mut CurrentSgfDocument,
    ) -> Result<Option<DocumentHistoryOutcome>, CurrentGameError> {
        let Some(mut edit) = self.redo.pop() else {
            return Ok(None);
        };
        let structural = match document.apply_reversal(&mut edit.reversal) {
            Ok(structural) => structural,
            Err(error) => {
                self.redo.push(edit);
                return Err(error);
            }
        };
        let selected_path = edit.selected_after.clone();
        self.undo.push(edit);
        Ok(Some(DocumentHistoryOutcome {
            selected_path,
            structural,
        }))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwingAnalysisScopeResolution {
    pub requested: Vec<SelectedNodeSnapshotDto>,
    pub supporting: Vec<SelectedNodeSnapshotDto>,
    pub comparisons: Vec<AnalysisSwingComparisonDto>,
}

fn selected_markup(node: &SgfNode, board_width: u8, board_height: u8) -> Vec<app_model::SgfMarkupDto> {
    use app_model::{PointDto, SgfMarkupDto};
    let mut markup = Vec::new();
    for property in &node.properties {
        for value in &property.values {
            if property.key == "LB" {
                if let Some((raw_point, text)) = value.split_once(':') {
                    if let Ok(point) = crate::parse_point(raw_point, board_width, board_height) {
                        markup.push(SgfMarkupDto::Label {
                            point: PointDto {
                                x: point.x,
                                y: point.y,
                            },
                            text: text.to_string(),
                        });
                    }
                }
            } else if matches!(property.key.as_str(), "CR" | "SQ" | "MA" | "TR") {
                if let Ok(points) = crate::parse_setup_points(value, board_width, board_height) {
                    for point in points {
                        let point = PointDto {
                            x: point.x,
                            y: point.y,
                        };
                        markup.push(match property.key.as_str() {
                            "CR" => SgfMarkupDto::Circle { point },
                            "SQ" => SgfMarkupDto::Square { point },
                            "MA" => SgfMarkupDto::Cross { point },
                            _ => SgfMarkupDto::Triangle { point },
                        });
                    }
                }
            }
        }
    }
    markup
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
        let (position, stone_move_numbers) = self.replay_nodes(&nodes)?;
        let projected = crate::analysis::project_node_analysis(
            selected,
            self.document.board_width,
            self.document.board_height,
            path.indices.is_empty(),
        );
        Ok(SelectedNodeSnapshotDto {
            path: path.clone(),
            position: position.clone(),
            stone_move_numbers,
            personal_comment: personal_comment(selected),
            markup: selected_markup(selected, self.document.board_width, self.document.board_height),
            generated_information: None,
            primary_analysis: projected
                .primary
                .as_ref()
                .map(|payload| payload.to_frame(position.move_number)),
            secondary_analysis: projected
                .secondary
                .as_ref()
                .map(|payload| payload.to_frame(position.move_number)),
        })
    }

    pub fn first_child_mainline_snapshots(&self) -> Result<Vec<SelectedNodeSnapshotDto>, CurrentGameError> {
        let mut snapshots = Vec::new();
        let mut indices = Vec::new();
        loop {
            let path = NodePath {
                indices: indices.clone(),
            };
            snapshots.push(self.snapshot(&path)?);
            let node = *self
                .nodes_on_path(&path)?
                .last()
                .expect("path walk includes the root");
            if node.children.is_empty() {
                break;
            }
            indices.push(0);
        }
        Ok(snapshots)
    }

    pub fn analysis_scope_snapshots(
        &self,
        scope: &app_model::AnalysisScopeDto,
    ) -> Result<Vec<SelectedNodeSnapshotDto>, CurrentGameError> {
        use app_model::AnalysisScopeModeDto;
        let invalid = |message: &str| CurrentGameError {
            kind: CurrentGameErrorKind::InvalidNodePath,
            message: message.to_string(),
        };
        if scope
            .interval
            .as_ref()
            .is_some_and(|range| range.start > range.end)
        {
            return Err(invalid("Position interval start must not exceed its end."));
        }
        let mut choices = std::collections::BTreeMap::new();
        for choice in &scope.branch_choices {
            let nodes = self.nodes_on_path(&choice.parent)?;
            let parent = nodes.last().expect("path includes root");
            if choice.child as usize >= parent.children.len() {
                return Err(invalid("Remembered branch choice no longer exists."));
            }
            if choices
                .insert(choice.parent.indices.clone(), choice.child)
                .is_some()
            {
                return Err(invalid("Duplicate remembered branch choice."));
            }
        }
        let mut pending = vec![match scope.mode {
            AnalysisScopeModeDto::CurrentNode => scope.current_node.clone(),
            _ => NodePath::default(),
        }];
        let mut snapshots = Vec::new();
        let mut maximum = 0;
        while let Some(path) = pending.pop() {
            let nodes = self.nodes_on_path(&path)?;
            let node = nodes.last().expect("path includes root");
            let explicit = scope.mode == AnalysisScopeModeDto::CurrentNode;
            let position_node = path.indices.is_empty()
                || node
                    .properties
                    .iter()
                    .any(|property| matches!(property.key.as_str(), "B" | "W" | "AB" | "AW" | "AE" | "PL"));
            if explicit || position_node {
                let snapshot = self.snapshot(&path)?;
                let position = &snapshot.position;
                maximum = maximum.max(position.move_number);
                if scope.interval.as_ref().is_none_or(|range| {
                    range.start <= position.move_number && position.move_number <= range.end
                }) && scope.to_play.is_none_or(|color| color == position.to_play)
                {
                    snapshots.push(snapshot);
                }
            }
            if explicit || node.children.is_empty() {
                continue;
            }
            if scope.mode == AnalysisScopeModeDto::AllBranches {
                for index in (0..node.children.len()).rev() {
                    let mut child = path.clone();
                    child.indices.push(index as u32);
                    pending.push(child);
                }
            } else {
                let index = if scope.mode == AnalysisScopeModeDto::SelectedReviewLine {
                    choices.get(&path.indices).copied().unwrap_or(0)
                } else {
                    0
                };
                let mut child = path;
                child.indices.push(index);
                pending.push(child);
            }
        }
        if scope.interval.as_ref().is_some_and(|range| range.end > maximum) {
            return Err(invalid("Position interval exceeds the selected scope."));
        }
        if snapshots.is_empty() {
            return Err(invalid("The selected scope and filters contain no positions."));
        }
        Ok(snapshots)
    }

    pub fn swing_analysis_scope(
        &self,
        scope: &app_model::AnalysisScopeDto,
        move_actors: AnalysisMoveActorFilterDto,
    ) -> Result<SwingAnalysisScopeResolution, CurrentGameError> {
        let requested = self.analysis_scope_snapshots(scope)?;
        let requested_paths = requested
            .iter()
            .map(|snapshot| snapshot.path.indices.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let mut supporting_paths = std::collections::BTreeSet::new();
        let mut supporting = Vec::new();
        let mut comparisons = Vec::new();

        for snapshot in &requested {
            let nodes = self.nodes_on_path(&snapshot.path)?;
            let node = nodes.last().expect("path walk includes the root");
            let move_actor = node
                .properties
                .iter()
                .find_map(|property| match property.key.as_str() {
                    "B" => Some(PlayerColor::Black),
                    "W" => Some(PlayerColor::White),
                    _ => None,
                });
            let Some(move_actor) = move_actor.filter(|color| move_actors.admits(*color)) else {
                continue;
            };
            let mut before = snapshot.path.clone();
            if before.indices.pop().is_none() {
                continue;
            }
            comparisons.push(AnalysisSwingComparisonDto {
                before: before.clone(),
                after: snapshot.path.clone(),
                move_actor,
            });
            if !requested_paths.contains(&before.indices) && supporting_paths.insert(before.indices.clone()) {
                supporting.push(self.snapshot(&before)?);
            }
        }

        Ok(SwingAnalysisScopeResolution {
            requested,
            supporting,
            comparisons,
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

    pub fn board_width(&self) -> u8 {
        self.document.board_width
    }

    pub fn board_height(&self) -> u8 {
        self.document.board_height
    }

    pub fn rules(&self) -> String {
        let raw = self
            .root()
            .ok()
            .and_then(|root| property_values(root, "RU"))
            .and_then(|values| values.first())
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        match raw {
            Some(value) => value.to_ascii_lowercase().replace(' ', "-"),
            None => "chinese".to_string(),
        }
    }

    pub fn tree(&self) -> Result<SgfTreeNodeDto, CurrentGameError> {
        Ok(tree_dto(self.root()?))
    }
    pub fn set_metadata_with_history(
        &mut self,
        selected_before: &NodePath,
        black_name: &str,
        white_name: &str,
        komi: f32,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        if !komi.is_finite() {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::MalformedSgf,
                message: "komi must be finite".into(),
            });
        }
        let before_snapshot = self.snapshot(selected_before)?;
        let root = self.root()?;
        let stored_name = |key| {
            property_values(root, key)
                .and_then(|values| values.first())
                .map(String::as_str)
                .unwrap_or("")
        };
        let black_changed = stored_name("PB") != black_name;
        let white_changed = stored_name("PW") != white_name;
        let previous_komi = self.document.komi;
        let komi_changed = previous_komi != komi;
        if !black_changed && !white_changed && !komi_changed {
            return Ok(DocumentEditOutcome {
                snapshot: before_snapshot,
                edit: None,
            });
        }
        let before = metadata_properties(root);
        let mut desired = before.clone();
        for (key, changed) in [("PB", black_changed), ("PW", white_changed), ("KM", komi_changed)] {
            if changed {
                let value = match key {
                    "PB" => black_name.to_owned(),
                    "PW" => white_name.to_owned(),
                    _ => komi.to_string(),
                };
                let index = before
                    .iter()
                    .find(|(_, property)| property.key == key)
                    .map(|(index, _)| *index)
                    .unwrap_or(root.properties.len() + desired.len());
                desired.retain(|(_, property)| property.key != key);
                desired.push((
                    index,
                    SgfProperty {
                        key: key.into(),
                        values: vec![value],
                    },
                ));
            }
        }
        let analysis = if komi_changed {
            Some(take_analysis(
                self.document.root.as_mut().expect("validated root"),
            ))
        } else {
            None
        };
        let root = self.document.root.as_mut().expect("validated root");
        restore_metadata(&mut root.properties, &desired);
        self.document.komi = komi;
        Ok(DocumentEditOutcome {
            snapshot: self.snapshot(selected_before)?,
            edit: Some(SgfDocumentEdit {
                reversal: DocumentReversal::SetMetadata {
                    properties: before,
                    komi: previous_komi,
                    analysis,
                },
                selected_before: selected_before.clone(),
                selected_after: selected_before.clone(),
            }),
        })
    }

    pub fn play(
        &mut self,
        path: &NodePath,
        vertex: MoveVertex,
    ) -> Result<SelectedNodeSnapshotDto, CurrentGameError> {
        Ok(self.play_with_history(path, path, vertex)?.snapshot)
    }

    pub fn play_with_history(
        &mut self,
        selected_before: &NodePath,
        path: &NodePath,
        vertex: MoveVertex,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        if selected_before != path {
            self.snapshot(selected_before)?;
        }
        let to_play = self.snapshot(path)?.position.to_play;
        if let Some(index) = self.existing_child_index(path, to_play, &vertex)? {
            let mut indices = path.indices.clone();
            indices.push(index);
            return Ok(DocumentEditOutcome {
                snapshot: self.snapshot(&NodePath { indices })?,
                edit: None,
            });
        }

        let mut board = self.board_after(path)?;
        board
            .play(to_core_color(to_play), to_core_vertex(&vertex))
            .map_err(rule_error)?;
        let encoded = serialize_vertex(&vertex, self.document.board_width, self.document.board_height)?;
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
            parent.children.len() - 1
        };
        let mut indices = path.indices.clone();
        indices.push(u32::try_from(index).expect("child index fits u32"));
        let selected_after = NodePath { indices };
        Ok(DocumentEditOutcome {
            snapshot: self.snapshot(&selected_after)?,
            edit: Some(SgfDocumentEdit {
                reversal: DocumentReversal::RemoveSubtree {
                    parent: path.clone(),
                    index,
                },
                selected_before: selected_before.clone(),
                selected_after,
            }),
        })
    }

    pub fn set_personal_comment(
        &mut self,
        path: &NodePath,
        comment: &str,
    ) -> Result<SelectedNodeSnapshotDto, CurrentGameError> {
        Ok(self
            .set_personal_comment_with_history(path, path, comment)?
            .snapshot)
    }

    pub fn set_personal_comment_with_history(
        &mut self,
        selected_before: &NodePath,
        path: &NodePath,
        comment: &str,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        if selected_before != path {
            self.snapshot(selected_before)?;
        }
        let before_snapshot = self.snapshot(path)?;
        let before = comment_properties(self.node_mut(path)?);
        apply_personal_comment(self.node_mut(path)?, comment);
        let after = comment_properties(self.node_mut(path)?);
        if before == after {
            return Ok(DocumentEditOutcome {
                snapshot: before_snapshot,
                edit: None,
            });
        }
        Ok(DocumentEditOutcome {
            snapshot: self.snapshot(path)?,
            edit: Some(SgfDocumentEdit {
                reversal: DocumentReversal::SetComment {
                    path: path.clone(),
                    properties: before,
                },
                selected_before: selected_before.clone(),
                selected_after: path.clone(),
            }),
        })
    }

    pub fn apply_root_setup_with_history(
        &mut self,
        selected_before: &NodePath,
        stones: &[StoneDto],
        to_play: PlayerColor,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        let root_path = NodePath::default();
        self.snapshot(selected_before)?;
        if !selected_before.indices.is_empty() || !self.root()?.children.is_empty() {
            return Err(root_setup_error(
                "Direct setup requires a childless selected root.",
            ));
        }
        validate_setup_stones(stones, self.document.board_width, self.document.board_height)?;
        let before = self.snapshot(&root_path)?;
        if same_setup(&before.position.stones, stones) && before.position.to_play == to_play {
            return Ok(DocumentEditOutcome {
                snapshot: before,
                edit: None,
            });
        }
        let properties = root_setup_properties(self.root()?);
        replace_root_setup_properties(self.node_mut(&root_path)?, stones, to_play, false);
        Ok(DocumentEditOutcome {
            snapshot: self.snapshot(&root_path)?,
            edit: Some(SgfDocumentEdit {
                reversal: DocumentReversal::RootSetup {
                    properties,
                    children: None,
                },
                selected_before: selected_before.clone(),
                selected_after: root_path,
            }),
        })
    }

    pub fn convert_to_root_setup_with_history(
        &mut self,
        selected_before: &NodePath,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        let position = self.snapshot(selected_before)?.position;
        let original_position = self.snapshot(&NodePath::default())?.position;
        let root_path = NodePath::default();
        let root = self.root()?;
        if root.children.is_empty() && !root.properties.iter().any(|p| p.key == "B" || p.key == "W") {
            return Ok(DocumentEditOutcome {
                snapshot: self.snapshot(&root_path)?,
                edit: None,
            });
        }
        let properties = root_setup_properties(root);
        let root = self.node_mut(&root_path)?;
        let children = std::mem::take(&mut root.children);
        replace_root_setup_properties(
            root,
            &position.stones,
            position.to_play,
            same_setup(&position.stones, &original_position.stones)
                && position.to_play == original_position.to_play,
        );
        Ok(DocumentEditOutcome {
            snapshot: self.snapshot(&root_path)?,
            edit: Some(SgfDocumentEdit {
                reversal: DocumentReversal::RootSetup {
                    properties,
                    children: Some(children),
                },
                selected_before: selected_before.clone(),
                selected_after: root_path,
            }),
        })
    }

    pub fn replace_primary_analysis(
        &mut self,
        path: &NodePath,
        payload: &crate::SgfAnalysisPayload,
    ) -> Result<(SelectedNodeSnapshotDto, bool), CurrentGameError> {
        let board_width = self.document.board_width;
        let board_height = self.document.board_height;
        let is_root = path.indices.is_empty();
        let changed = crate::analysis::replace_primary_analysis(
            self.node_mut(path)?,
            payload,
            board_width,
            board_height,
            is_root,
        );
        Ok((self.snapshot(path)?, changed))
    }

    pub fn remove_variation(&mut self, path: &NodePath) -> Result<NodePath, CurrentGameError> {
        Ok(self.remove_variation_with_history(path, path)?.snapshot.path)
    }

    pub fn remove_variation_with_history(
        &mut self,
        selected_before: &NodePath,
        path: &NodePath,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        if path.indices.is_empty() {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::RootRemoval,
                message: "cannot remove the root".to_string(),
            });
        }
        self.snapshot(selected_before)?;
        let _ = self.nodes_on_path(path)?;
        let index = *path.indices.last().expect("non-root path") as usize;
        let parent = NodePath {
            indices: path.indices[..path.indices.len() - 1].to_vec(),
        };
        self.snapshot(&parent)?;
        let subtree = self.node_mut(&parent)?.children.remove(index);
        Ok(DocumentEditOutcome {
            snapshot: self.snapshot(&parent)?,
            edit: Some(SgfDocumentEdit {
                reversal: DocumentReversal::InsertSubtree {
                    parent: parent.clone(),
                    index,
                    subtree,
                },
                selected_before: selected_before.clone(),
                selected_after: parent,
            }),
        })
    }

    pub fn promote_to_main_with_history(
        &mut self,
        selected_before: &NodePath,
        path: &NodePath,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        self.snapshot(selected_before)?;
        self.snapshot(path)?;
        let mut selected_after = path.clone();
        let mut moves = Vec::new();
        for depth in 0..path.indices.len() {
            let index = path.indices[depth] as usize;
            if index == 0 {
                continue;
            }
            let parent = NodePath {
                indices: selected_after.indices[..depth].to_vec(),
            };
            let children = &mut self.node_mut(&parent)?.children;
            let promoted = children.remove(index);
            children.insert(0, promoted);
            moves.push((parent, index));
            selected_after.indices[depth] = 0;
        }
        let edit = (!moves.is_empty()).then_some(SgfDocumentEdit {
            reversal: DocumentReversal::PromoteMainline {
                moves,
                to_front: false,
            },
            selected_before: selected_before.clone(),
            selected_after: selected_after.clone(),
        });
        Ok(DocumentEditOutcome {
            snapshot: self.snapshot(&selected_after)?,
            edit,
        })
    }

    fn apply_reversal(&mut self, reversal: &mut DocumentReversal) -> Result<bool, CurrentGameError> {
        match reversal {
            DocumentReversal::SetComment { path, properties } => {
                let node = self.node_mut(path)?;
                let inverse = comment_properties(node);
                replace_comment_properties(node, properties);
                *properties = inverse;
                Ok(false)
            }
            DocumentReversal::SetMetadata {
                properties,
                komi,
                analysis,
            } => {
                let root = self.document.root.as_mut().expect("validated root");
                let previous = metadata_properties(root);
                restore_metadata(&mut root.properties, properties);
                *properties = previous;
                std::mem::swap(komi, &mut self.document.komi);
                if let Some(saved) = analysis {
                    let current = take_analysis(root);
                    restore_analysis(root, saved);
                    *saved = current;
                }
                Ok(analysis.is_some())
            }
            DocumentReversal::RemoveSubtree { parent, index } => {
                let parent_path = parent.clone();
                let child_index = *index;
                let node = self.node_mut(&parent_path)?;
                if child_index >= node.children.len() {
                    return Err(invalid_history_path());
                }
                let subtree = node.children.remove(child_index);
                *reversal = DocumentReversal::InsertSubtree {
                    parent: parent_path,
                    index: child_index,
                    subtree,
                };
                Ok(true)
            }
            DocumentReversal::InsertSubtree {
                parent,
                index,
                subtree,
            } => {
                let parent_path = parent.clone();
                let child_index = *index;
                let node = self.node_mut(&parent_path)?;
                if child_index > node.children.len() {
                    return Err(invalid_history_path());
                }
                let inserted = std::mem::replace(
                    subtree,
                    SgfNode {
                        properties: Vec::new(),
                        children: Vec::new(),
                    },
                );
                node.children.insert(child_index, inserted);
                *reversal = DocumentReversal::RemoveSubtree {
                    parent: parent_path,
                    index: child_index,
                };
                Ok(true)
            }
            DocumentReversal::PromoteMainline { moves, to_front } => {
                if *to_front {
                    for (parent, index) in moves.iter() {
                        let children = &mut self.node_mut(parent)?.children;
                        let moved = children.remove(*index);
                        children.insert(0, moved);
                    }
                } else {
                    for (parent, index) in moves.iter().rev() {
                        let children = &mut self.node_mut(parent)?.children;
                        let moved = children.remove(0);
                        children.insert(*index, moved);
                    }
                }
                *to_front = !*to_front;
            }
            DocumentReversal::RootSetup { properties, children } => {
                let root = self.node_mut(&NodePath::default())?;
                let inverse = root_setup_properties(root);
                restore_root_setup_properties(root, properties);
                *properties = inverse;
                if let Some(children) = children {
                    std::mem::swap(&mut root.children, children);
                }
                Ok(true)
            }
        }
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

    fn replay_nodes(&self, nodes: &[&SgfNode]) -> Result<(PositionDto, Vec<MoveDto>), CurrentGameError> {
        let mut board = Board::new(self.document.board_width, self.document.board_height).map_err(|_| {
            CurrentGameError {
                kind: CurrentGameErrorKind::UnsupportedBoardSize,
                message: SgfError::UnsupportedBoardDimensions {
                    width: self.document.board_width,
                    height: self.document.board_height,
                }
                .to_string(),
            }
        })?;
        let mut captures_black = 0u32;
        let mut captures_white = 0u32;
        let mut to_play = PlayerColor::Black;
        let mut last_move = None;
        let mut move_number = 0u32;
        let mut errors = Vec::new();
        let mut stone_sources: Vec<Option<MoveDto>> =
            vec![None; self.document.board_width as usize * self.document.board_height as usize];

        for node in nodes {
            apply_setup_properties(
                &mut board,
                node,
                self.document.board_width,
                self.document.board_height,
            )?;
            for property in &node.properties {
                if !matches!(property.key.as_str(), "AB" | "AW" | "AE") {
                    continue;
                }
                for value in &property.values {
                    for point in crate::parse_setup_points(
                        value,
                        self.document.board_width,
                        self.document.board_height,
                    )? {
                        stone_sources
                            [point.y as usize * self.document.board_width as usize + point.x as usize] = None;
                    }
                }
            }
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
                    vertex: parse_vertex(raw, self.document.board_width, self.document.board_height)?,
                    move_number,
                };
                match board.play(to_core_color(sgf_move.color), to_core_vertex(&sgf_move.vertex)) {
                    Ok(outcome) => {
                        match sgf_move.color {
                            PlayerColor::Black => captures_black += outcome.captured.len() as u32,
                            PlayerColor::White => captures_white += outcome.captured.len() as u32,
                        }
                        for captured in outcome.captured {
                            let idx = captured.y as usize * self.document.board_width as usize
                                + captured.x as usize;
                            stone_sources[idx] = None;
                        }
                        if let MoveVertex::Point(point) = &sgf_move.vertex {
                            let idx =
                                point.y as usize * self.document.board_width as usize + point.x as usize;
                            stone_sources[idx] = Some(sgf_move.clone());
                        }
                    }
                    Err(error) => errors.push(format!("{error}")),
                }
                to_play = player_to_play(node)?.unwrap_or_else(|| sgf_move.color.opponent());
                last_move = Some(sgf_move);
            }
        }

        let mut stone_move_numbers: Vec<MoveDto> = stone_sources.into_iter().flatten().collect();
        stone_move_numbers.sort_by_key(|m| m.move_number);

        Ok((
            PositionDto {
                board_width: self.document.board_width,
                board_height: self.document.board_height,
                move_number,
                to_play,
                stones: stones_from_board(&board),
                captures_black,
                captures_white,
                last_move,
                errors,
            },
            stone_move_numbers,
        ))
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
                if &parse_vertex(raw, self.document.board_width, self.document.board_height)? == vertex {
                    return Ok(Some(u32::try_from(index).expect("child index fits u32")));
                }
            }
        }
        Ok(None)
    }

    fn board_after(&self, path: &NodePath) -> Result<Board, CurrentGameError> {
        let nodes = self.nodes_on_path(path)?;
        let mut board = Board::new(self.document.board_width, self.document.board_height).map_err(|_| {
            CurrentGameError {
                kind: CurrentGameErrorKind::UnsupportedBoardSize,
                message: SgfError::UnsupportedBoardDimensions {
                    width: self.document.board_width,
                    height: self.document.board_height,
                }
                .to_string(),
            }
        })?;
        for node in nodes {
            apply_setup_properties(
                &mut board,
                node,
                self.document.board_width,
                self.document.board_height,
            )?;
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
                let played = parse_vertex(raw, self.document.board_width, self.document.board_height)?;
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
            SgfError::UnsupportedBoardDimensions { .. } => CurrentGameError {
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

fn invalid_history_path() -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::InvalidNodePath,
        message: "document history no longer matches the current tree".to_string(),
    }
}
fn metadata_properties(node: &SgfNode) -> Vec<(usize, SgfProperty)> {
    node.properties
        .iter()
        .enumerate()
        .filter(|(_, property)| matches!(property.key.as_str(), "PB" | "PW" | "KM"))
        .map(|(index, property)| (index, property.clone()))
        .collect()
}

fn restore_metadata(properties: &mut Vec<SgfProperty>, desired: &[(usize, SgfProperty)]) {
    properties.retain(|property| !matches!(property.key.as_str(), "PB" | "PW" | "KM"));
    let mut ordered = desired.to_vec();
    ordered.sort_by_key(|(index, _)| *index);
    for (index, property) in ordered {
        properties.insert(index.min(properties.len()), property);
    }
}

fn take_analysis(root: &mut SgfNode) -> Vec<(NodePath, Vec<(usize, SgfProperty)>)> {
    fn visit(
        node: &mut SgfNode,
        path: &mut Vec<u32>,
        removed: &mut Vec<(NodePath, Vec<(usize, SgfProperty)>)>,
    ) {
        let properties: Vec<_> = node
            .properties
            .iter()
            .enumerate()
            .filter(|(_, property)| matches!(property.key.as_str(), "LZ" | "LZ2" | "LZOP" | "LZOP2"))
            .map(|(index, property)| (index, property.clone()))
            .collect();
        if !properties.is_empty() {
            node.properties
                .retain(|property| !matches!(property.key.as_str(), "LZ" | "LZ2" | "LZOP" | "LZOP2"));
            removed.push((
                NodePath {
                    indices: path.clone(),
                },
                properties,
            ));
        }
        for (index, child) in node.children.iter_mut().enumerate() {
            path.push(u32::try_from(index).expect("child index fits u32"));
            visit(child, path, removed);
            path.pop();
        }
    }
    let mut removed = Vec::new();
    visit(root, &mut Vec::new(), &mut removed);
    removed
}

fn restore_analysis(root: &mut SgfNode, saved: &[(NodePath, Vec<(usize, SgfProperty)>)]) {
    for (path, properties) in saved {
        let mut node = &mut *root;
        for &index in &path.indices {
            node = &mut node.children[index as usize];
        }
        for (index, property) in properties {
            node.properties
                .insert((*index).min(node.properties.len()), property.clone());
        }
    }
}

fn root_setup_error(message: &str) -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::InvalidNodePath,
        message: message.into(),
    }
}

fn is_root_setup_property(key: &str) -> bool {
    matches!(
        key,
        "AB" | "AW" | "AE" | "PL" | "B" | "W" | "LZ" | "LZ2" | "LZOP" | "LZOP2"
    )
}

fn root_setup_properties(root: &SgfNode) -> Vec<(usize, SgfProperty)> {
    root.properties
        .iter()
        .enumerate()
        .filter(|(_, property)| is_root_setup_property(&property.key))
        .map(|(index, property)| (index, property.clone()))
        .collect()
}

fn restore_root_setup_properties(root: &mut SgfNode, properties: &[(usize, SgfProperty)]) {
    root.properties
        .retain(|property| !is_root_setup_property(&property.key));
    for (index, property) in properties {
        root.properties
            .insert((*index).min(root.properties.len()), property.clone());
    }
}

fn replace_root_setup_properties(
    root: &mut SgfNode,
    stones: &[StoneDto],
    to_play: PlayerColor,
    retain_analysis: bool,
) {
    root.properties.retain(|property| {
        !is_root_setup_property(&property.key)
            || retain_analysis && matches!(property.key.as_str(), "LZ" | "LZ2" | "LZOP" | "LZOP2")
    });
    for (color, key) in [(PlayerColor::Black, "AB"), (PlayerColor::White, "AW")] {
        let values: Vec<String> = stones
            .iter()
            .filter(|stone| stone.color == color)
            .map(|stone| format!("{}{}", char::from(b'a' + stone.x), char::from(b'a' + stone.y)))
            .collect();
        if !values.is_empty() {
            root.properties.push(SgfProperty {
                key: key.into(),
                values,
            });
        }
    }
    root.properties.push(SgfProperty {
        key: "PL".into(),
        values: vec![match to_play {
            PlayerColor::Black => "B",
            PlayerColor::White => "W",
        }
        .into()],
    });
}

fn validate_setup_stones(stones: &[StoneDto], width: u8, height: u8) -> Result<(), CurrentGameError> {
    let mut used = vec![false; width as usize * height as usize];
    for stone in stones {
        if stone.x >= width || stone.y >= height {
            return Err(root_setup_error("Setup point is outside the board."));
        }
        let index = stone.y as usize * width as usize + stone.x as usize;
        if std::mem::replace(&mut used[index], true) {
            return Err(root_setup_error("Setup point occurs more than once."));
        }
    }
    Ok(())
}

fn same_setup(left: &[StoneDto], right: &[StoneDto]) -> bool {
    left.len() == right.len() && left.iter().all(|stone| right.contains(stone))
}

fn comment_properties(node: &SgfNode) -> Vec<(usize, SgfProperty)> {
    node.properties
        .iter()
        .enumerate()
        .filter(|(_, property)| property.key == "C")
        .map(|(index, property)| (index, property.clone()))
        .collect()
}

fn replace_comment_properties(node: &mut SgfNode, properties: &[(usize, SgfProperty)]) {
    node.properties.retain(|property| property.key != "C");
    for (index, property) in properties {
        node.properties
            .insert((*index).min(node.properties.len()), property.clone());
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

        assert_eq!(
            (snapshot.position.board_width, snapshot.position.board_height),
            (5, 5)
        );
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
        assert_eq!(
            (
                root_snapshot.position.board_width,
                root_snapshot.position.board_height
            ),
            (5, 5)
        );
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

#[cfg(test)]
mod selected_node_analysis_capture {
    use super::*;
    use app_model::NodePath;

    #[test]
    fn selected_node_rules_come_from_ru_or_default_chinese() {
        let japanese = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[5]KM[6.5]RU[Japanese];B[cc])").unwrap();
        assert_eq!(japanese.rules(), "japanese");
        assert_eq!(japanese.komi(), 6.5);
        assert_eq!((japanese.board_width(), japanese.board_height()), (5, 5));

        let defaulted = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[9]KM[7.5];B[cc])").unwrap();
        assert_eq!(defaulted.rules(), "chinese");

        let spaced = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[9]KM[7.5]RU[New Zealand])").unwrap();
        assert_eq!(spaced.rules(), "new-zealand");
    }

    #[test]
    fn selected_node_snapshot_captures_exact_path_board_and_turn() {
        let document = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[5]KM[6.5]RU[Japanese];B[cc];W[ee])").unwrap();
        let path = NodePath { indices: vec![0, 0] };
        let snapshot = document.snapshot(&path).unwrap();
        assert_eq!(snapshot.path.indices, vec![0, 0]);
        assert_eq!(snapshot.position.move_number, 2);
        assert_eq!(snapshot.position.to_play, PlayerColor::Black);
        assert_eq!(snapshot.position.stones.len(), 2);
        assert!(snapshot
            .position
            .stones
            .iter()
            .any(|stone| { stone.x == 2 && stone.y == 2 && stone.color == PlayerColor::Black }));
        assert!(snapshot
            .position
            .stones
            .iter()
            .any(|stone| { stone.x == 4 && stone.y == 4 && stone.color == PlayerColor::White }));
    }
}

#[cfg(test)]
mod first_child_mainline_worklist {
    use super::*;
    use app_model::{MoveVertex, PlayerColor, PointDto};

    const BRANCHING: &str = include_str!("../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn first_child_mainline_includes_root_and_ignores_sibling_variations() {
        let document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let snapshots = document.first_child_mainline_snapshots().unwrap();
        let paths: Vec<Vec<u32>> = snapshots
            .iter()
            .map(|snapshot| snapshot.path.indices.clone())
            .collect();
        assert_eq!(
            paths,
            vec![Vec::new(), vec![0], vec![0, 0], vec![0, 0, 0]],
            "whole-game mainline is root plus recursive Primary Child, not the selected continuation or sibling variations"
        );
        assert_eq!(document.default_selected_path().indices, vec![0, 0, 0]);
    }

    #[test]
    fn first_child_mainline_captures_setup_pass_komi_and_rules() {
        let document = CurrentSgfDocument::open(BRANCHING).unwrap();
        let snapshots = document.first_child_mainline_snapshots().unwrap();
        assert_eq!(snapshots.len(), 4);
        assert_eq!(document.komi(), 0.5);
        assert_eq!((document.board_width(), document.board_height()), (5, 5));
        assert_eq!(document.rules(), "chinese");

        let root = &snapshots[0];
        assert!(root.path.indices.is_empty());
        assert_eq!(root.position.move_number, 0);
        assert_eq!(root.position.to_play, PlayerColor::White);
        assert!(has_stone(&root.position, 0, 0, PlayerColor::Black));
        assert!(has_stone(&root.position, 2, 2, PlayerColor::White));
        assert!(!has_stone(&root.position, 1, 1, PlayerColor::Black));

        let first = &snapshots[1];
        assert_eq!(first.path.indices, vec![0]);
        assert_eq!(first.position.move_number, 1);
        assert_eq!(first.position.to_play, PlayerColor::Black);
        assert_eq!(
            first.position.last_move.as_ref().unwrap().vertex,
            MoveVertex::Point(PointDto { x: 3, y: 3 })
        );

        let pass = &snapshots[3];
        assert_eq!(pass.path.indices, vec![0, 0, 0]);
        assert_eq!(pass.position.move_number, 3);
        assert_eq!(pass.position.to_play, PlayerColor::Black);
        assert_eq!(pass.position.last_move.as_ref().unwrap().vertex, MoveVertex::Pass);
        assert_eq!(
            pass.position.last_move.as_ref().unwrap().color,
            PlayerColor::White
        );
    }

    #[test]
    fn first_child_mainline_uses_current_game_owned_rules() {
        let document = CurrentSgfDocument::open("(;GM[1]FF[4]SZ[9]KM[6.5]RU[Japanese];B[dd])").unwrap();
        assert_eq!(document.rules(), "japanese");
        assert_eq!(document.komi(), 6.5);
        let snapshots = document.first_child_mainline_snapshots().unwrap();
        assert_eq!(
            snapshots
                .iter()
                .map(|snapshot| snapshot.path.indices.clone())
                .collect::<Vec<_>>(),
            vec![Vec::new(), vec![0]]
        );
    }

    fn has_stone(position: &app_model::PositionDto, x: u8, y: u8, color: PlayerColor) -> bool {
        position
            .stones
            .iter()
            .any(|stone| stone.x == x && stone.y == y && stone.color == color)
    }
}

#[cfg(test)]
mod snapshot_stone_move_numbers {
    use super::*;
    use app_model::{MoveDto, MoveVertex, NodePath, PlayerColor, PointDto};

    #[test]
    fn snapshot_stone_move_numbers_multilevel_alternate_branch_vs_mainline() {
        let sgf = "(;SZ[5]\
            ;B[aa]\
            (;W[bb];B[cc])\
            (;W[dd];B[ee]\
                (;W[ab])\
                (;W[ba])\
            )\
        )";
        let document = CurrentSgfDocument::open(sgf).unwrap();
        let initial_serialized = document.serialize().unwrap();

        let mainline_path = NodePath {
            indices: vec![0, 0, 0],
        };
        let mainline_snap = document.snapshot(&mainline_path).unwrap();
        assert_eq!(mainline_snap.position.move_number, 3);
        assert_eq!(
            mainline_snap.stone_move_numbers,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 0 }),
                    move_number: 1,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 1, y: 1 }),
                    move_number: 2,
                },
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 2, y: 2 }),
                    move_number: 3,
                },
            ]
        );

        let branch_a_path = NodePath {
            indices: vec![0, 1, 0, 0],
        };
        let branch_a_snap = document.snapshot(&branch_a_path).unwrap();
        assert_eq!(branch_a_snap.position.move_number, 4);
        assert_eq!(
            branch_a_snap.stone_move_numbers,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 0 }),
                    move_number: 1,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 3, y: 3 }),
                    move_number: 2,
                },
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 4, y: 4 }),
                    move_number: 3,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 1 }),
                    move_number: 4,
                },
            ]
        );

        let branch_b_path = NodePath {
            indices: vec![0, 1, 0, 1],
        };
        let branch_b_snap = document.snapshot(&branch_b_path).unwrap();
        assert_eq!(branch_b_snap.position.move_number, 4);
        assert_eq!(
            branch_b_snap.stone_move_numbers,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 0 }),
                    move_number: 1,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 3, y: 3 }),
                    move_number: 2,
                },
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 4, y: 4 }),
                    move_number: 3,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 1, y: 0 }),
                    move_number: 4,
                },
            ]
        );

        assert_eq!(document.serialize().unwrap(), initial_serialized);
    }

    #[test]
    fn snapshot_stone_move_numbers_capture_and_replay_same_point() {
        let sgf = "(;SZ[5];B[ab];W[aa];B[ba];W[];B[aa])";
        let document = CurrentSgfDocument::open(sgf).unwrap();
        let initial_serialized = document.serialize().unwrap();

        let path_move2 = NodePath { indices: vec![0, 0] };
        let snap2 = document.snapshot(&path_move2).unwrap();
        assert_eq!(snap2.position.move_number, 2);
        assert_eq!(
            snap2.stone_move_numbers,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 1 }),
                    move_number: 1,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 0 }),
                    move_number: 2,
                },
            ]
        );

        let path_move3 = NodePath {
            indices: vec![0, 0, 0],
        };
        let snap3 = document.snapshot(&path_move3).unwrap();
        assert_eq!(snap3.position.move_number, 3);
        assert_eq!(snap3.position.captures_black, 1);
        assert_eq!(
            snap3.stone_move_numbers,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 1 }),
                    move_number: 1,
                },
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 1, y: 0 }),
                    move_number: 3,
                },
            ]
        );

        let path_move4 = NodePath {
            indices: vec![0, 0, 0, 0],
        };
        let snap4 = document.snapshot(&path_move4).unwrap();
        assert_eq!(snap4.position.move_number, 4);
        assert_eq!(snap4.stone_move_numbers, snap3.stone_move_numbers);

        let path_move5 = NodePath {
            indices: vec![0, 0, 0, 0, 0],
        };
        let snap5 = document.snapshot(&path_move5).unwrap();
        assert_eq!(snap5.position.move_number, 5);
        assert_eq!(
            snap5.stone_move_numbers,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 1 }),
                    move_number: 1,
                },
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 1, y: 0 }),
                    move_number: 3,
                },
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 0 }),
                    move_number: 5,
                },
            ]
        );

        assert_eq!(document.serialize().unwrap(), initial_serialized);
    }

    #[test]
    fn snapshot_stone_move_numbers_setup_same_color_reassertion_pass_comment() {
        let sgf = "(;SZ[5]AB[ba]AW[bb]C[root setup]PL[B]\
            ;B[ab]C[first move]\
            ;W[]\
            ;C[tactical note]\
            ;AB[ab]AE[ba]AW[cc]\
            ;W[dd]\
        )";
        let document = CurrentSgfDocument::open(sgf).unwrap();
        let initial_serialized = document.serialize().unwrap();

        let root_path = NodePath { indices: Vec::new() };
        let root_snap = document.snapshot(&root_path).unwrap();
        assert_eq!(root_snap.position.move_number, 0);
        assert!(root_snap.stone_move_numbers.is_empty());
        assert_eq!(root_snap.position.to_play, PlayerColor::Black);

        let path_1 = NodePath { indices: vec![0] };
        let snap_1 = document.snapshot(&path_1).unwrap();
        assert_eq!(snap_1.position.move_number, 1);
        assert_eq!(
            snap_1.stone_move_numbers,
            vec![MoveDto {
                color: PlayerColor::Black,
                vertex: MoveVertex::Point(PointDto { x: 0, y: 1 }),
                move_number: 1,
            }]
        );

        let path_2 = NodePath { indices: vec![0, 0] };
        let snap_2 = document.snapshot(&path_2).unwrap();
        assert_eq!(snap_2.position.move_number, 2);
        assert_eq!(snap_2.stone_move_numbers, snap_1.stone_move_numbers);

        let path_3 = NodePath {
            indices: vec![0, 0, 0],
        };
        let snap_3 = document.snapshot(&path_3).unwrap();
        assert_eq!(snap_3.position.move_number, 2);
        assert_eq!(snap_3.stone_move_numbers, snap_1.stone_move_numbers);

        let path_4 = NodePath {
            indices: vec![0, 0, 0, 0],
        };
        let snap_4 = document.snapshot(&path_4).unwrap();
        assert_eq!(snap_4.position.move_number, 2);
        assert!(
            snap_4.stone_move_numbers.is_empty(),
            "same-color reassertion AB[ab] clears move source for (0, 1)"
        );
        assert!(snap_4
            .position
            .stones
            .iter()
            .any(|s| s.x == 0 && s.y == 1 && s.color == PlayerColor::Black));

        let path_5 = NodePath {
            indices: vec![0, 0, 0, 0, 0],
        };
        let snap_5 = document.snapshot(&path_5).unwrap();
        assert_eq!(snap_5.position.move_number, 3);
        assert_eq!(
            snap_5.stone_move_numbers,
            vec![MoveDto {
                color: PlayerColor::White,
                vertex: MoveVertex::Point(PointDto { x: 3, y: 3 }),
                move_number: 3,
            }]
        );

        assert_eq!(document.serialize().unwrap(), initial_serialized);
    }

    #[test]
    fn snapshot_stone_move_numbers_rectangular_edge_and_illegal_replay_move() {
        let sgf = "(;SZ[4:6];B[ad];W[df];B[da];W[af];B[ad])";
        let document = CurrentSgfDocument::open(sgf).unwrap();
        let initial_serialized = document.serialize().unwrap();

        let leaf_path = NodePath {
            indices: vec![0, 0, 0, 0, 0],
        };
        let snap = document.snapshot(&leaf_path).unwrap();

        assert_eq!((snap.position.board_width, snap.position.board_height), (4, 6));
        assert_eq!(snap.position.move_number, 5);
        assert_eq!(snap.position.errors.len(), 1);
        assert!(snap.position.errors[0].contains("already occupied"));

        assert_eq!(
            snap.stone_move_numbers,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 3 }),
                    move_number: 1,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 3, y: 5 }),
                    move_number: 2,
                },
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 3, y: 0 }),
                    move_number: 3,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 0, y: 5 }),
                    move_number: 4,
                },
            ]
        );

        assert_eq!(document.serialize().unwrap(), initial_serialized);
    }

    #[test]
    fn snapshot_stone_move_numbers_root_move_counts() {
        let sgf = "(;SZ[5]B[cc];W[dd])";
        let document = CurrentSgfDocument::open(sgf).unwrap();
        let root_path = NodePath { indices: Vec::new() };
        let root_snap = document.snapshot(&root_path).unwrap();
        assert_eq!(root_snap.position.move_number, 1);
        assert_eq!(
            root_snap.stone_move_numbers,
            vec![MoveDto {
                color: PlayerColor::Black,
                vertex: MoveVertex::Point(PointDto { x: 2, y: 2 }),
                move_number: 1,
            }]
        );

        let child_path = NodePath { indices: vec![0] };
        let child_snap = document.snapshot(&child_path).unwrap();
        assert_eq!(child_snap.position.move_number, 2);
        assert_eq!(
            child_snap.stone_move_numbers,
            vec![
                MoveDto {
                    color: PlayerColor::Black,
                    vertex: MoveVertex::Point(PointDto { x: 2, y: 2 }),
                    move_number: 1,
                },
                MoveDto {
                    color: PlayerColor::White,
                    vertex: MoveVertex::Point(PointDto { x: 3, y: 3 }),
                    move_number: 2,
                },
            ]
        );
    }
}
