use super::*;
use app_model::{AnalysisBranchChoiceDto, PointDto, PointSearchScopeDto};

impl CurrentSgfDocument {
    /// Resolves a recorded point without editing the document or moving its cursor.
    pub fn find_recorded_point(
        &self,
        selected: &NodePath,
        point: PointDto,
        choices: &[AnalysisBranchChoiceDto],
        scope: PointSearchScopeDto,
    ) -> Result<Option<NodePath>, CurrentGameError> {
        let ancestors = self.nodes_on_path(selected)?;
        let width = self.board_width();
        let height = self.board_height();
        if point.x >= width || point.y >= height {
            return Ok(None);
        }
        let matches = |node: &SgfNode| {
            ["B", "W"].iter().any(|key| {
                property_values(node, key)
                    .and_then(|values| values.first())
                    .and_then(|raw| parse_vertex(raw, width, height).ok())
                    == Some(MoveVertex::Point(point))
            })
        };
        // Current node, then nearest ancestor. Repeating a query is deliberately a no-op.
        for (depth, node) in ancestors.iter().enumerate().rev() {
            if matches(node) {
                return Ok(Some(NodePath {
                    indices: selected.indices[..depth].to_vec(),
                }));
            }
        }
        let mut path = selected.clone();
        let mut node = *ancestors.last().expect("root exists");
        while !node.children.is_empty() {
            let child = choices
                .iter()
                .find(|choice| choice.parent == path)
                .map_or(0, |choice| choice.child as usize);
            let Some(next) = node.children.get(child) else {
                return Err(CurrentGameError {
                    kind: CurrentGameErrorKind::InvalidNodePath,
                    message: "Selected continuation no longer exists.".into(),
                });
            };
            path.indices.push(child as u32);
            node = next;
            if matches(node) {
                return Ok(Some(path));
            }
        }
        if scope == PointSearchScopeDto::CurrentLine {
            return Ok(None);
        }
        // Stable pre-order DFS uses one path and an iterator stack, not a copied path per node.
        let root = self.root()?;
        let mut path = NodePath::default();
        if matches(root) {
            return Ok(Some(path));
        }
        let mut stack = vec![root.children.iter().enumerate()];
        while let Some(children) = stack.last_mut() {
            if let Some((index, child)) = children.next() {
                path.indices.push(index as u32);
                if matches(child) {
                    return Ok(Some(path));
                }
                stack.push(child.children.iter().enumerate());
            } else {
                stack.pop();
                path.indices.pop();
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path(indices: &[u32]) -> NodePath {
        NodePath {
            indices: indices.to_vec(),
        }
    }
    #[test]
    fn point_search_preserves_source_order_scope_and_repeated_exact_paths() {
        let game = CurrentSgfDocument::open("(;SZ[19](;B[aa];W[bb])(;B[aa];W[cc])(;B[dd];W[ee]))").unwrap();
        let before = game.serialize().unwrap();
        let aa = PointDto { x: 0, y: 0 };
        let full = PointSearchScopeDto::AllBranches;
        let line = PointSearchScopeDto::CurrentLine;
        assert_eq!(
            game.find_recorded_point(&path(&[1, 0]), aa, &[], full).unwrap(),
            Some(path(&[1]))
        );
        assert_eq!(
            game.find_recorded_point(&path(&[1]), aa, &[], full).unwrap(),
            Some(path(&[1]))
        );
        assert_eq!(
            game.find_recorded_point(&path(&[2]), aa, &[], full).unwrap(),
            Some(path(&[0]))
        );
        assert_eq!(
            game.find_recorded_point(&path(&[2]), aa, &[], line).unwrap(),
            None
        );
        assert_eq!(
            game.find_recorded_point(&path(&[2]), PointDto { x: 4, y: 4 }, &[], full)
                .unwrap(),
            Some(path(&[2, 0]))
        );
        assert_eq!(
            game.find_recorded_point(&path(&[]), aa, &[], full).unwrap(),
            Some(path(&[0]))
        );
        let choices = [AnalysisBranchChoiceDto {
            parent: path(&[]),
            child: 1,
        }];
        assert_eq!(
            game.find_recorded_point(&path(&[]), aa, &choices, full).unwrap(),
            Some(path(&[1]))
        );
        assert_eq!(
            game.find_recorded_point(&path(&[2]), PointDto { x: 5, y: 5 }, &[], full)
                .unwrap(),
            None
        );
        assert_eq!(game.serialize().unwrap(), before);
    }
    #[test]
    fn point_search_ignores_setup_pass_and_offboard_and_prefers_nearest_ancestor() {
        let game = CurrentSgfDocument::open("(;SZ[5]AB[ee];B[ab];W[aa];B[ba];W[];B[aa];W[])").unwrap();
        let full = PointSearchScopeDto::AllBranches;
        assert_eq!(
            game.find_recorded_point(&path(&[0, 0, 0, 0, 0, 0]), PointDto { x: 0, y: 0 }, &[], full)
                .unwrap(),
            Some(path(&[0, 0, 0, 0, 0]))
        );
        assert_eq!(
            game.find_recorded_point(&path(&[]), PointDto { x: 4, y: 4 }, &[], full)
                .unwrap(),
            None
        );
        assert_eq!(
            game.find_recorded_point(&path(&[]), PointDto { x: 5, y: 0 }, &[], full)
                .unwrap(),
            None
        );
        assert!(game
            .find_recorded_point(&path(&[7]), PointDto { x: 0, y: 0 }, &[], full)
            .is_err());
    }
}
