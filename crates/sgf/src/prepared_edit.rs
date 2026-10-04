use super::*;

/// A private edit prepared against the owner's locked document generation.
/// Apply before releasing that lock or changing the document.
#[derive(Debug)]
pub struct PreparedSgfEdit {
    mutation: PreparedMutation,
    outcome: DocumentEditOutcome,
}

#[derive(Debug)]
enum PreparedMutation {
    Append { parent: NodePath, node: SgfNode },
    Result { properties: Vec<SgfProperty>, result: String },
    Unchanged,
}

impl CurrentSgfDocument {
    /// Prepares a fresh match move, including exact legality and its full snapshot.
    pub fn prepare_play(
        &self,
        path: &NodePath,
        vertex: MoveVertex,
    ) -> Result<PreparedSgfEdit, CurrentGameError> {
        let exact_error = |message| CurrentGameError {
            kind: CurrentGameErrorKind::MalformedSgf,
            message,
        };
        let exact = self.exact_position(path).map_err(exact_error)?;
        exact.validate_move(&vertex).map_err(exact_error)?;
        let color = exact.dto().to_play;
        if self.existing_child_index(path, color, &vertex)?.is_some() {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::InvalidNodePath,
                message: "A prepared match move cannot reuse an existing continuation.".into(),
            });
        }
        let node = SgfNode {
            properties: vec![SgfProperty {
                key: match color {
                    PlayerColor::Black => "B",
                    PlayerColor::White => "W",
                }.into(),
                values: vec![serialize_vertex(&vertex, self.document.board_width, self.document.board_height)?],
            }],
            children: Vec::new(),
        };
        let mut nodes = self.nodes_on_path(path)?;
        let index = nodes.last().expect("validated path").children.len();
        let mut selected_after = path.clone();
        selected_after.indices.push(u32::try_from(index).map_err(|_| invalid_history_path())?);
        nodes.push(&node);
        let snapshot = self.snapshot_from_nodes(&selected_after, &nodes)?;
        Ok(PreparedSgfEdit {
            mutation: PreparedMutation::Append { parent: path.clone(), node },
            outcome: DocumentEditOutcome {
                snapshot,
                edit: Some(SgfDocumentEdit {
                    reversal: DocumentReversal::RemoveSubtree { parent: path.clone(), index },
                    selected_before: path.clone(),
                    selected_after,
                }),
            },
        })
    }

    /// Prepares a standard result without changing root properties or cached metadata.
    pub fn prepare_result(
        &self,
        path: &NodePath,
        result: &str,
    ) -> Result<PreparedSgfEdit, CurrentGameError> {
        validate_result(result)?;
        let snapshot = self.snapshot(path)?;
        let root = self.root()?;
        let before = result_properties(root);
        if before.len() == 1 && before[0].1.values.len() == 1 && before[0].1.values[0] == result {
            return Ok(PreparedSgfEdit {
                mutation: PreparedMutation::Unchanged,
                outcome: DocumentEditOutcome { snapshot, edit: None },
            });
        }
        let index = before.first().map(|(index, _)| *index).unwrap_or(root.properties.len());
        let mut staged_root = SgfNode { properties: root.properties.clone(), children: Vec::new() };
        replace_result_properties(&mut staged_root, &[(index, SgfProperty {
            key: "RE".into(),
            values: vec![result.into()],
        })]);
        let properties = staged_root.properties;
        Ok(PreparedSgfEdit {
            mutation: PreparedMutation::Result { properties, result: result.into() },
            outcome: DocumentEditOutcome {
                snapshot,
                edit: Some(SgfDocumentEdit {
                    reversal: DocumentReversal::SetResult {
                        properties: before,
                        result: self.document.result.clone(),
                    },
                    selected_before: path.clone(),
                    selected_after: path.clone(),
                }),
            },
        })
    }

    /// Commits an edit prepared under the same uninterrupted owner lock/generation.
    pub fn apply_prepared(&mut self, prepared: PreparedSgfEdit) -> DocumentEditOutcome {
        match prepared.mutation {
            PreparedMutation::Append { parent, node } => {
                self.node_mut(&parent)
                    .expect("prepared path remains valid under owner lock")
                    .children.push(node);
            }
            PreparedMutation::Result { properties, result } => {
                self.document.root.as_mut()
                    .expect("prepared root remains valid under owner lock")
                    .properties = properties;
                self.document.result = Some(result);
            }
            PreparedMutation::Unchanged => {}
        }
        prepared.outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::{ExactRulesDto, PointDto};

    fn game() -> CurrentSgfDocument {
        CurrentSgfDocument::stage_new_game(9, 7.5, 0, ExactRulesDto::ChineseKgs, "Human", "Engine").unwrap()
    }

    #[test]
    fn prepared_moves_are_private_and_reversible_with_exact_snapshots() {
        let mut document = game();
        let root = NodePath::default();
        let before = document.serialize().unwrap();
        let prepared = document.prepare_play(&root, MoveVertex::Point(PointDto { x: 2, y: 2 })).unwrap();
        assert_eq!(document.serialize().unwrap(), before);
        let outcome = document.apply_prepared(prepared);
        assert_eq!(outcome.snapshot, document.snapshot(&outcome.snapshot.path).unwrap());
        assert_eq!(outcome.snapshot.position.to_play, PlayerColor::White);
        let path = outcome.snapshot.path.clone();
        let after = document.serialize().unwrap();
        let mut history = DocumentHistory::default();
        history.commit(outcome.edit.unwrap());
        assert!(document.prepare_play(&path, MoveVertex::Point(PointDto { x: 2, y: 2 })).is_err());
        assert!(document.prepare_play(&root, MoveVertex::Point(PointDto { x: 2, y: 2 })).is_err());
        assert!(document.prepare_play(&NodePath { indices: vec![99] }, MoveVertex::Pass).is_err());
        assert_eq!(document.serialize().unwrap(), after);
        let pass = document.prepare_play(&path, MoveVertex::Pass).unwrap();
        assert_eq!(document.serialize().unwrap(), after);
        let pass = document.apply_prepared(pass);
        assert_eq!(pass.snapshot, document.snapshot(&pass.snapshot.path).unwrap());
        history.commit(pass.edit.unwrap());
        assert_eq!(history.undo(&mut document).unwrap().unwrap().selected_path, path);
        assert_eq!(document.serialize().unwrap(), after);
        assert_eq!(history.undo(&mut document).unwrap().unwrap().selected_path, root);
        assert_eq!(document.serialize().unwrap(), before);
        history.redo(&mut document).unwrap().unwrap();
        assert_eq!(document.serialize().unwrap(), after);
    }

    #[test]
    fn prepared_result_preserves_metadata_and_is_private_until_applied() {
        let mut document = CurrentSgfDocument::open(
            "(;SZ[9]RU[Chinese]KM[7.5]RE[W+1.5]C[personal]XX[unknown]LZ[opaque](;B[aa])(;B[bb]))",
        ).unwrap();
        let path = NodePath { indices: vec![1] };
        let before = document.serialize().unwrap();
        assert!(document.prepare_result(&path, "unfinished").is_err());
        assert!(document.prepare_result(&NodePath { indices: vec![99] }, "B+R").is_err());
        let prepared = document.prepare_result(&path, "B+R").unwrap();
        assert_eq!(document.serialize().unwrap(), before);
        assert_eq!(document.result(), Some("W+1.5"));
        let outcome = document.apply_prepared(prepared);
        assert_eq!(outcome.snapshot, document.snapshot(&path).unwrap());
        assert_eq!(document.result(), Some("B+R"));
        let after = document.serialize().unwrap();
        let mut history = DocumentHistory::default();
        history.commit(outcome.edit.unwrap());
        let same = document.prepare_result(&path, "B+R").unwrap();
        assert!(document.apply_prepared(same).edit.is_none());
        assert_eq!(history.undo(&mut document).unwrap().unwrap().selected_path, path);
        assert_eq!(document.serialize().unwrap(), before);
        history.redo(&mut document).unwrap().unwrap();
        assert_eq!(document.serialize().unwrap(), after);
    }
}
