use app_model::{CurrentGameError, CurrentGameResultDto};
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::CurrentGameErrorKind;

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
}
