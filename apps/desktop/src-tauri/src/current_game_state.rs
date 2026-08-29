use app_model::{CurrentGameError, CurrentGameErrorKind, CurrentGameResultDto, GameDto, NodePath};
use sgf::CurrentSgfDocument;
use std::sync::Mutex;

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

    pub fn mainline_projection(&self) -> Result<GameDto, CurrentGameError> {
        self.with_document(|document| Ok(document.mainline_projection()))
    }

    #[allow(dead_code)]
    pub fn discard_confirmation_required(&self) -> bool {
        self.holder.lock().expect("current game state").dirty
    }

    pub fn select_path(&self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.holder.lock().expect("current game state").select_path(path)
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
