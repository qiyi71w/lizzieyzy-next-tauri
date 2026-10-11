use super::CurrentSgfDocument;
use app_model::{CurrentGameError, CurrentGameErrorKind, NodePath};

impl CurrentSgfDocument {
    /// Capture descendants of the actual selected branch, including structural nodes.
    pub fn continuation_line(
        &self,
        selected: &NodePath,
        choices: &[app_model::AnalysisBranchChoiceDto],
    ) -> Result<Vec<NodePath>, CurrentGameError> {
        let mut path = selected.clone();
        let mut line = Vec::new();
        loop {
            let nodes = self.nodes_on_path(&path)?;
            let node = nodes.last().expect("root included");
            if node.children.is_empty() {
                return Ok(line);
            }
            let child = choices
                .iter()
                .find(|choice| choice.parent == path)
                .map_or(0, |choice| choice.child);
            if child as usize >= node.children.len() {
                return Err(CurrentGameError {
                    kind: CurrentGameErrorKind::InvalidNodePath,
                    message: "Selected review branch no longer exists.".into(),
                });
            }
            path.indices.push(child);
            line.push(path.clone());
        }
    }

    pub fn continuation_terminal(&self, selected: &NodePath) -> Result<bool, CurrentGameError> {
        if self
            .result()
            .is_some_and(|result| !result.trim().is_empty() && result != "?")
        {
            return Ok(true);
        }
        let nodes = self.nodes_on_path(selected)?;
        let mut passes = 0;
        for node in nodes.into_iter().rev() {
            // Setup/PL changes end a played sequence; structural comments do not.
            if node
                .properties
                .iter()
                .any(|p| matches!(p.key.as_str(), "AB" | "AW" | "AE" | "PL"))
            {
                break;
            }
            if let Some(property) = node
                .properties
                .iter()
                .find(|p| matches!(p.key.as_str(), "B" | "W"))
            {
                if property.values.first().is_some_and(|value| {
                    matches!(
                        crate::parse_vertex(value, self.board_width(), self.board_height()),
                        Ok(app_model::MoveVertex::Pass)
                    )
                }) {
                    passes += 1;
                } else {
                    break;
                }
                if passes == 2 {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
