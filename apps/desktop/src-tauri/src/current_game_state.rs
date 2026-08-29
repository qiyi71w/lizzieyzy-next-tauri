use app_model::{CurrentGameError, CurrentGameErrorKind, CurrentGameResultDto, NodePath};
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
        self.holder
            .lock()
            .expect("current game state")
            .replace(sgf_text, native_path)
    }

    pub fn select_path(&self, path: NodePath) -> Result<CurrentGameResultDto, CurrentGameError> {
        self.holder.lock().expect("current game state").select_path(path)
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
mod tests {
    use super::*;
    use app_model::{CurrentGameErrorKind, NodePath};

    const BRANCHING: &str = include_str!("../../../../tests/golden/editable-workspace-branching.sgf");

    #[test]
    fn replace_installs_current_game_and_failed_open_preserves_it() {
        let state = CurrentGameState::default();
        let first = state
            .replace(BRANCHING, Some("/tmp/branching.sgf".to_string()))
            .unwrap();

        assert_eq!(first.generation, 1);
        assert!(!first.dirty);
        assert_eq!(first.native_path.as_deref(), Some("/tmp/branching.sgf"));
        assert_eq!(first.selected_path.indices, vec![0, 0, 0]);
        assert_eq!(first.snapshot.personal_comment, "mainline pass");
        assert_eq!(first.snapshot.position.move_number, 3);

        let before = state.holder.lock().expect("current game state").snapshot_state();
        let error = state
            .replace("not an sgf", Some("/tmp/bad.sgf".to_string()))
            .unwrap_err();
        let after = state.holder.lock().expect("current game state").snapshot_state();

        assert_eq!(error.kind, CurrentGameErrorKind::MalformedSgf);
        assert_eq!(after, before);

        let unsupported = state
            .replace("(;SZ[99])", Some("/tmp/big.sgf".to_string()))
            .unwrap_err();
        assert_eq!(unsupported.kind, CurrentGameErrorKind::UnsupportedBoardSize);
        assert_eq!(
            state.holder.lock().expect("current game state").snapshot_state(),
            before
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
