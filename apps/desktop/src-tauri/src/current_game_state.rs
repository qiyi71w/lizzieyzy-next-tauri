use app_model::{
    CurrentGameError, CurrentGameErrorKind, CurrentGameResultDto, GameDto, MoveVertex, NodePath,
    SelectedNodeSnapshotDto,
};
use sgf::CurrentSgfDocument;
use std::sync::Mutex;

#[cfg(test)]
mod current_game_save_write;

#[derive(Debug, Clone)]
pub struct WholeGameAdmission {
    pub generation: u64,
    pub board_size: u8,
    pub komi: f32,
    pub rules: String,
    pub nodes: Vec<SelectedNodeSnapshotDto>,
}

#[derive(Default)]
pub struct CurrentGameState {
    holder: Mutex<CurrentGameHolder>,
}

#[derive(Default)]
struct CurrentGameHolder {
    document: Option<CurrentSgfDocument>,
    generation: u64,
    dirty: bool,
    native_path: Option<String>,
}

impl CurrentGameState {
    pub fn replace(
        &self,
        sgf_text: &str,
        native_path: Option<String>,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.replace_unless_discarded(sgf_text, native_path, true)?
            .ok_or_else(no_current_game)
    }

    pub fn replace_unless_discarded(
        &self,
        sgf_text: &str,
        native_path: Option<String>,
        discard_confirmed: bool,
    ) -> Result<Option<CurrentGameResultDto>, CurrentGameError> {
        let mut holder = self.holder.lock().expect("current game state");
        if holder.dirty && !discard_confirmed {
            return Ok(None);
        }
        Ok(Some(holder.replace(sgf_text, native_path)?))
    }

    pub fn serialize(&self) -> Result<String, CurrentGameError> {
        self.with_document(|document| document.serialize())
    }

    pub fn save_to_path(
        &self,
        path: String,
        selected_path: NodePath,
    ) -> Result<CurrentGameResultDto, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("path must not be empty".to_string());
        }
        let target = std::path::PathBuf::from(trimmed);
        let mut holder = self.holder.lock().expect("current game state");
        let document = holder
            .document
            .as_ref()
            .ok_or_else(|| no_current_game().to_string())?;
        let serialized = document.serialize().map_err(|error| error.to_string())?;
        let snapshot = document
            .snapshot(&selected_path)
            .map_err(|error| error.to_string())?;
        let tree = document.tree().map_err(|error| error.to_string())?;
        std::fs::write(&target, &serialized)
            .map_err(|err| format!("failed to write SGF file {}: {err}", target.display()))?;
        holder.dirty = false;
        holder.native_path = Some(trimmed.to_string());
        Ok(CurrentGameResultDto {
            tree,
            selected_path,
            snapshot,
            generation: holder.generation,
            dirty: holder.dirty,
            native_path: holder.native_path.clone(),
        })
    }

    pub fn mainline_projection(&self) -> Result<GameDto, CurrentGameError> {
        self.with_document(|document| Ok(document.mainline_projection()))
    }

    pub fn admit_whole_game(&self, generation: u64) -> Result<WholeGameAdmission, CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        let document = holder.document.as_ref().ok_or_else(no_current_game)?;
        if holder.generation != generation {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::NoCurrentGame,
                message: "current game generation does not match".to_string(),
            });
        }
        Ok(WholeGameAdmission {
            generation: holder.generation,
            board_size: document.board_size(),
            komi: document.komi(),
            rules: document.rules(),
            nodes: document.first_child_mainline_snapshots()?,
        })
    }

    pub fn admit_selected_node(
        &self,
        generation: u64,
        path: &NodePath,
    ) -> Result<(SelectedNodeSnapshotDto, u8, f32, String), CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        let document = holder.document.as_ref().ok_or_else(no_current_game)?;
        if holder.generation != generation {
            return Err(CurrentGameError {
                kind: CurrentGameErrorKind::NoCurrentGame,
                message: "current game generation does not match".to_string(),
            });
        }
        let snapshot = document.snapshot(path)?;
        Ok((snapshot, document.board_size(), document.komi(), document.rules()))
    }

    #[allow(dead_code)]
    pub fn discard_confirmation_required(&self) -> bool {
        self.holder.lock().expect("current game state").dirty
    }

    pub fn select_path(&self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.holder.lock().expect("current game state").select_path(path)
    }

    pub fn play(&self, path: NodePath, vertex: MoveVertex) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.holder.lock().expect("current game state").play(path, vertex)
    }

    pub fn set_personal_comment(
        &self,
        path: NodePath,
        comment: String,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.holder
            .lock()
            .expect("current game state")
            .set_personal_comment(path, &comment)
    }

    pub fn remove_variation(&self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.holder
            .lock()
            .expect("current game state")
            .remove_variation(path)
    }

    fn with_document<T>(
        &self,
        f: impl FnOnce(&CurrentSgfDocument) -> Result<T, CurrentGameError>,
    ) -> Result<T, CurrentGameError> {
        let holder = self.holder.lock().expect("current game state");
        f(holder.document.as_ref().ok_or_else(no_current_game)?)
    }
}

fn no_current_game() -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::NoCurrentGame,
        message: "no current game".to_string(),
    }
}

impl CurrentGameHolder {
    fn replace(
        &mut self,
        sgf_text: &str,
        native_path: Option<String>,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let document = CurrentSgfDocument::open(sgf_text)?;
        let selected_path = document.default_selected_path();
        let snapshot = document.snapshot(&selected_path)?;
        let tree = document.tree()?;
        self.document = Some(document);
        self.generation += 1;
        self.dirty = false;
        self.native_path = native_path;
        Ok(CurrentGameResultDto {
            tree,
            selected_path,
            snapshot,
            generation: self.generation,
            dirty: self.dirty,
            native_path: self.native_path.clone(),
        })
    }

    #[cfg(test)]
    fn snapshot_state(&self) -> (u64, bool, Option<String>, Option<String>) {
        (
            self.generation,
            self.dirty,
            self.native_path.clone(),
            self.document
                .as_ref()
                .and_then(|document| document.serialize().ok()),
        )
    }

    fn play(&mut self, path: NodePath, vertex: MoveVertex) -> Result<CurrentGameResultDto, CurrentGameError> {
        let document = self.document.as_mut().ok_or_else(no_current_game)?;
        let before = document.serialize()?;
        let snapshot = document.play(&path, vertex)?;
        let after = document.serialize()?;
        if before != after {
            self.generation += 1;
            self.dirty = true;
        }
        Ok(CurrentGameResultDto {
            tree: document.tree()?,
            selected_path: snapshot.path.clone(),
            snapshot,
            generation: self.generation,
            dirty: self.dirty,
            native_path: self.native_path.clone(),
        })
    }

    fn select_path(&self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        let document = self.document.as_ref().ok_or_else(|| CurrentGameError {
            kind: CurrentGameErrorKind::NoCurrentGame,
            message: "no current game".to_string(),
        })?;
        let snapshot = document.snapshot(&path)?;
        Ok(CurrentGameResultDto {
            tree: document.tree()?,
            selected_path: path,
            snapshot,
            generation: self.generation,
            dirty: self.dirty,
            native_path: self.native_path.clone(),
        })
    }

    fn set_personal_comment(
        &mut self,
        path: NodePath,
        comment: &str,
    ) -> Result<CurrentGameResultDto, CurrentGameError> {
        let (snapshot, tree, changed) = {
            let document = self.document.as_mut().ok_or_else(no_current_game)?;
            let before = document.serialize()?;
            let snapshot = document.set_personal_comment(&path, comment)?;
            let changed = document.serialize()? != before;
            (snapshot, document.tree()?, changed)
        };
        if changed {
            self.generation += 1;
            self.dirty = true;
        }
        Ok(CurrentGameResultDto {
            tree,
            selected_path: path,
            snapshot,
            generation: self.generation,
            dirty: self.dirty,
            native_path: self.native_path.clone(),
        })
    }

    fn remove_variation(&mut self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        let document = self.document.as_mut().ok_or_else(no_current_game)?;
        let selected_path = document.remove_variation(&path)?;
        let snapshot = document.snapshot(&selected_path)?;
        let tree = document.tree()?;
        self.generation += 1;
        self.dirty = true;
        Ok(CurrentGameResultDto {
            tree,
            selected_path,
            snapshot,
            generation: self.generation,
            dirty: self.dirty,
            native_path: self.native_path.clone(),
        })
    }
}

#[cfg(test)]
mod current_game_replacement {
    use super::*;
    use app_model::{CurrentGameErrorKind, MoveVertex, NodePath};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");
    const EMPTY: &str = "(;GM[1]FF[4]SZ[19]KM[7.5]PB[黑]PW[白])";

    #[test]
    fn current_game_replacement_installs_shared_result_and_preserves_state_on_cancel_or_failure() {
        let state = CurrentGameState::default();

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        assert_eq!(opened.generation, 1);
        assert!(!opened.dirty);
        assert_eq!(opened.native_path.as_deref(), Some("/tmp/branching.sgf"));
        assert_eq!(opened.selected_path.indices, vec![0, 0, 0]);
        assert_eq!(opened.snapshot.personal_comment, "mainline pass");
        assert_eq!(opened.snapshot.position.move_number, 3);

        let serialized = state.serialize().unwrap();
        let projection = state.mainline_projection().unwrap();
        assert_eq!(projection.moves.len(), 3);
        assert!(matches!(projection.moves[2].vertex, MoveVertex::Pass));
        assert_eq!(
            CurrentSgfDocument::open(&serialized)
                .unwrap()
                .default_selected_path()
                .indices,
            vec![0, 0, 0]
        );

        let imported = state.replace(EMPTY, None).unwrap();
        assert_eq!(imported.generation, 2);
        assert!(!imported.dirty);
        assert!(imported.native_path.is_none());
        assert!(imported.selected_path.indices.is_empty());
        assert_eq!(imported.snapshot.position.move_number, 0);
        assert!(state.mainline_projection().unwrap().moves.is_empty());

        state.force_dirty();
        let before_cancel = state.inspect();
        assert!(state.discard_confirmation_required());
        let cancelled = state.replace_unless_discarded(EMPTY, None, false).unwrap();
        assert!(cancelled.is_none());
        assert_eq!(state.inspect(), before_cancel);

        let before_failure = state.inspect();
        let error = state
            .replace("not an sgf", Some("/tmp/bad.sgf".to_string()))
            .unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::MalformedSgf);
        assert_eq!(state.inspect(), before_failure);

        let before_unsupported = state.inspect();
        let unsupported = state.replace("(;GM[1]FF[4]SZ[99])", None).unwrap_err();
        assert_eq!(unsupported.kind, CurrentGameErrorKind::UnsupportedBoardSize);
        assert_eq!(state.inspect(), before_unsupported);
    }

    #[test]
    fn current_game_replacement_reports_no_current_game_before_install() {
        let state = CurrentGameState::default();
        assert_eq!(
            state.serialize().unwrap_err().kind,
            CurrentGameErrorKind::NoCurrentGame
        );
        assert_eq!(
            state.mainline_projection().unwrap_err().kind,
            CurrentGameErrorKind::NoCurrentGame
        );
        assert_eq!(
            state.admit_whole_game(1).unwrap_err().kind,
            CurrentGameErrorKind::NoCurrentGame
        );
    }

    #[test]
    fn select_path_returns_snapshot_without_mutating_document_identity() {
        let state = CurrentGameState::default();
        let missing = state.select_path(NodePath { indices: Vec::new() }).unwrap_err();
        assert_eq!(missing.kind, CurrentGameErrorKind::NoCurrentGame);

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        let before = state.holder.lock().expect("current game state").snapshot_state();

        let second = state.select_path(NodePath { indices: vec![0, 1] }).unwrap();
        assert_eq!(
            state.holder.lock().expect("current game state").snapshot_state(),
            before
        );
        assert_eq!(second.generation, opened.generation);
        assert_eq!(second.dirty, opened.dirty);
        assert_eq!(second.native_path, opened.native_path);
        assert_eq!(second.tree, opened.tree);
        assert_eq!(second.selected_path.indices, vec![0, 1]);
        assert_eq!(second.snapshot.path.indices, vec![0, 1]);
        assert_eq!(second.snapshot.personal_comment, "second continuation");
        assert_eq!(second.snapshot.position.move_number, 2);
        assert_eq!(second.snapshot.position.to_play, app_model::PlayerColor::White);

        let invalid = state.select_path(NodePath { indices: vec![0, 2] }).unwrap_err();
        assert_eq!(invalid.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(
            state.holder.lock().expect("current game state").snapshot_state(),
            before
        );
        assert_eq!(opened.selected_path.indices, vec![0, 0, 0]);
    }

    #[test]
    fn admit_selected_node_requires_matching_generation_and_existing_path() {
        let state = CurrentGameState::default();
        let opened = state.replace(EMPTY, None).unwrap();
        let (snapshot, board_size, komi, rules) = state
            .admit_selected_node(opened.generation, &opened.selected_path)
            .unwrap();
        assert_eq!(snapshot.path, opened.selected_path);
        assert_eq!(board_size, 19);
        assert_eq!(komi, 7.5);
        assert_eq!(rules, "chinese");

        let stale = state
            .admit_selected_node(opened.generation + 1, &opened.selected_path)
            .unwrap_err();
        assert_eq!(stale.message, "current game generation does not match");

        let missing = state
            .admit_selected_node(opened.generation, &NodePath { indices: vec![9] })
            .unwrap_err();
        assert_eq!(missing.kind, CurrentGameErrorKind::InvalidNodePath);
    }

    #[test]
    fn admit_whole_game_captures_first_child_mainline_and_rejects_stale_generation() {
        let state = CurrentGameState::default();
        assert_eq!(
            state.admit_whole_game(1).unwrap_err().kind,
            CurrentGameErrorKind::NoCurrentGame
        );

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        let sibling = state.select_path(NodePath { indices: vec![0, 1] }).unwrap();
        assert_eq!(sibling.generation, opened.generation);
        assert_eq!(sibling.selected_path.indices, vec![0, 1]);

        let admitted = state.admit_whole_game(opened.generation).unwrap();
        assert_eq!(admitted.generation, opened.generation);
        assert_eq!(admitted.board_size, 5);
        assert_eq!(admitted.komi, 0.5);
        assert_eq!(admitted.rules, "chinese");
        let paths: Vec<Vec<u32>> = admitted
            .nodes
            .iter()
            .map(|node| node.path.indices.clone())
            .collect();
        assert_eq!(paths, vec![Vec::new(), vec![0], vec![0, 0], vec![0, 0, 0]]);
        assert_eq!(
            admitted.nodes[3].position.last_move.as_ref().unwrap().vertex,
            MoveVertex::Pass
        );

        let stale = state.admit_whole_game(opened.generation + 1).unwrap_err();
        assert_eq!(stale.message, "current game generation does not match");
    }
}

#[cfg(test)]
mod current_game_remove_variation {
    use super::*;
    use app_model::{CurrentGameErrorKind, PlayerColor};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn current_game_remove_variation_returns_parent_and_preserves_state_on_rejection() {
        let state = CurrentGameState::default();
        assert_eq!(
            state
                .remove_variation(NodePath { indices: vec![0] })
                .unwrap_err()
                .kind,
            CurrentGameErrorKind::NoCurrentGame
        );

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        let before = state.inspect();
        let before_projection = state.mainline_projection().unwrap();
        let before_serialize = state.serialize().unwrap();

        let root = state
            .remove_variation(NodePath { indices: Vec::new() })
            .unwrap_err();
        assert_eq!(root.kind, CurrentGameErrorKind::RootRemoval);
        assert_eq!(state.inspect(), before);
        assert_eq!(state.serialize().unwrap(), before_serialize);
        assert_eq!(
            state.mainline_projection().unwrap().moves.len(),
            before_projection.moves.len()
        );

        let invalid = state
            .remove_variation(NodePath { indices: vec![0, 2] })
            .unwrap_err();
        assert_eq!(invalid.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(state.inspect(), before);
        assert_eq!(state.serialize().unwrap(), before_serialize);

        let removed = state.remove_variation(NodePath { indices: vec![0, 1] }).unwrap();
        assert_eq!(removed.selected_path.indices, vec![0]);
        assert_eq!(removed.snapshot.path.indices, vec![0]);
        assert_eq!(removed.snapshot.personal_comment, "main move");
        assert_eq!(removed.snapshot.position.to_play, PlayerColor::Black);
        assert_eq!(removed.generation, opened.generation + 1);
        assert!(removed.dirty);
        assert_eq!(removed.native_path, opened.native_path);
        assert_eq!(removed.tree.children[0].children.len(), 1);
        assert!(state.serialize().unwrap().contains("first continuation"));
        assert!(!state.serialize().unwrap().contains("second continuation"));
        assert_eq!(
            state.inspect(),
            (
                removed.generation,
                true,
                opened.native_path.clone(),
                Some(state.serialize().unwrap())
            )
        );
        assert_eq!(state.mainline_projection().unwrap().moves.len(), 3);
    }
}

#[cfg(test)]
impl CurrentGameState {
    fn force_dirty(&self) {
        self.holder.lock().expect("current game state").dirty = true;
    }

    fn inspect(&self) -> (u64, bool, Option<String>, Option<String>) {
        self.holder.lock().expect("current game state").snapshot_state()
    }
}

#[cfg(test)]
mod current_game_mutation_edit {
    use super::*;
    use app_model::{CurrentGameErrorKind, MoveVertex, NodePath, PointDto};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn current_game_move_edit_updates_generation_dirty_and_stays_atomic() {
        let state = CurrentGameState::default();
        assert_eq!(
            state
                .play(NodePath { indices: Vec::new() }, MoveVertex::Pass)
                .unwrap_err()
                .kind,
            CurrentGameErrorKind::NoCurrentGame
        );

        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        assert_eq!(opened.generation, 1);
        assert!(!opened.dirty);

        let parent = NodePath { indices: vec![0] };
        let played = state
            .play(parent.clone(), MoveVertex::Point(PointDto { x: 1, y: 1 }))
            .unwrap();
        assert_eq!(played.generation, 2);
        assert!(played.dirty);
        assert_eq!(played.selected_path.indices, vec![0, 2]);
        assert_eq!(
            played.snapshot.position.last_move.as_ref().unwrap().color,
            app_model::PlayerColor::Black
        );
        assert_eq!(state.mainline_projection().unwrap().moves.len(), 3);
        assert_eq!(
            CurrentSgfDocument::open(&state.serialize().unwrap())
                .unwrap()
                .tree()
                .unwrap()
                .children[0]
                .children
                .len(),
            3
        );

        let existing = state
            .play(parent.clone(), MoveVertex::Point(PointDto { x: 4, y: 4 }))
            .unwrap();
        assert_eq!(existing.generation, 2);
        assert!(existing.dirty);
        assert_eq!(existing.selected_path.indices, vec![0, 0]);
        assert_eq!(existing.snapshot.personal_comment, "first continuation");

        let before = state.inspect();
        let occupied = state
            .play(parent, MoveVertex::Point(PointDto { x: 3, y: 3 }))
            .unwrap_err();
        assert_eq!(occupied.kind, CurrentGameErrorKind::OccupiedPoint);
        assert_eq!(state.inspect(), before);

        let invalid = state
            .play(NodePath { indices: vec![9] }, MoveVertex::Pass)
            .unwrap_err();
        assert_eq!(invalid.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(state.inspect(), before);
    }
}

#[cfg(test)]
mod current_game_comment_edit {
    use super::*;
    use app_model::{CurrentGameErrorKind, NodePath};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn current_game_comment_edit_marks_dirty_and_leaves_noop_unchanged() {
        let state = CurrentGameState::default();
        let opened = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();
        let root = NodePath { indices: Vec::new() };
        let move_node = NodePath { indices: vec![0] };
        let second_sibling = NodePath { indices: vec![0, 1] };

        let edited = state
            .set_personal_comment(second_sibling.clone(), "sibling edited".to_string())
            .unwrap();
        assert_eq!(edited.selected_path.indices, second_sibling.indices);
        assert_eq!(edited.snapshot.path.indices, second_sibling.indices);
        assert_eq!(edited.snapshot.personal_comment, "sibling edited");
        assert!(edited.snapshot.generated_information.is_none());
        assert_eq!(edited.generation, opened.generation + 1);
        assert!(edited.dirty);
        assert_eq!(edited.native_path, opened.native_path);
        assert!(state.serialize().unwrap().contains("C[sibling edited]"));
        assert_eq!(
            state.mainline_projection().unwrap().moves.len(),
            opened.snapshot.position.move_number as usize
        );

        let before_noop = state.inspect();
        let serialized = state.serialize().unwrap();
        let projection = state.mainline_projection().unwrap();
        let noop = state
            .set_personal_comment(second_sibling.clone(), "sibling edited".to_string())
            .unwrap();
        assert_eq!(noop.generation, edited.generation);
        assert_eq!(noop.dirty, edited.dirty);
        assert_eq!(noop.snapshot.personal_comment, "sibling edited");
        assert_eq!(state.inspect(), before_noop);
        assert_eq!(state.serialize().unwrap(), serialized);
        assert_eq!(state.mainline_projection().unwrap().moves, projection.moves);

        let root_edit = state
            .set_personal_comment(root.clone(), "root edited".to_string())
            .unwrap();
        assert_eq!(root_edit.selected_path.indices, root.indices);
        assert_eq!(root_edit.snapshot.personal_comment, "root edited");
        assert_eq!(root_edit.generation, edited.generation + 1);
        assert!(root_edit.dirty);

        let move_edit = state
            .set_personal_comment(move_node.clone(), "".to_string())
            .unwrap();
        assert_eq!(move_edit.selected_path.indices, move_node.indices);
        assert_eq!(move_edit.snapshot.personal_comment, "");
        assert_eq!(move_edit.generation, root_edit.generation + 1);

        let before_invalid = state.inspect();
        let error = state
            .set_personal_comment(NodePath { indices: vec![9] }, "nope".to_string())
            .unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::InvalidNodePath);
        assert_eq!(state.inspect(), before_invalid);
    }

    #[test]
    fn current_game_comment_edit_preserves_loaded_empty_comment_on_empty_submit() {
        let state = CurrentGameState::default();
        let opened = state.replace("(;GM[1]FF[4]SZ[5]C[])", None).unwrap();
        let root = NodePath { indices: Vec::new() };
        assert_eq!(opened.snapshot.personal_comment, "");
        let before = state.inspect();
        let serialized = state.serialize().unwrap();
        let noop = state.set_personal_comment(root, "".to_string()).unwrap();
        assert_eq!(noop.generation, opened.generation);
        assert!(!noop.dirty);
        assert_eq!(noop.snapshot.personal_comment, "");
        assert_eq!(state.inspect(), before);
        assert_eq!(state.serialize().unwrap(), serialized);
    }

    #[test]
    fn current_game_comment_edit_reports_no_current_game() {
        let state = CurrentGameState::default();
        let error = state
            .set_personal_comment(NodePath { indices: Vec::new() }, "note".to_string())
            .unwrap_err();
        assert_eq!(error.kind, CurrentGameErrorKind::NoCurrentGame);
    }
}
