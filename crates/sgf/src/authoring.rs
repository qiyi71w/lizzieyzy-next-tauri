use super::*;
use app_model::{PointDto, SgfAuthoringActionDto, SgfTransformDto};

fn refused(message: impl Into<String>) -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::InvalidNodePath,
        message: message.into(),
    }
}
fn core_point(point: &PointDto) -> go_core::Point {
    go_core::Point {
        x: point.x,
        y: point.y,
    }
}
fn raw_point(point: go_core::Point) -> String {
    String::from_utf8(vec![b'a' + point.x, b'a' + point.y]).expect("validated SGF point")
}
fn move_node(color: PlayerColor, point: PointDto) -> SgfNode {
    SgfNode {
        properties: vec![SgfProperty {
            key: if color == PlayerColor::Black { "B" } else { "W" }.into(),
            values: vec![raw_point(core_point(&point))],
        }],
        children: Vec::new(),
    }
}
fn apply_node(board: &mut Board, node: &SgfNode, width: u8, height: u8) -> Result<(), CurrentGameError> {
    apply_setup_properties(board, node, width, height)?;
    for property in &node.properties {
        let color = match property.key.as_str() {
            "B" => PlayerColor::Black,
            "W" => PlayerColor::White,
            _ => continue,
        };
        if property.values.len() != 1 {
            return Err(refused("A recorded move needs exactly one coordinate or pass."));
        }
        let vertex = parse_vertex(&property.values[0], width, height)?;
        board
            .play(to_core_color(color), to_core_vertex(&vertex))
            .map_err(rule_error)?;
    }
    Ok(())
}
fn validate_children(
    node: &SgfNode,
    board: &mut Board,
    width: u8,
    height: u8,
) -> Result<(), CurrentGameError> {
    let Some((last, siblings)) = node.children.split_last() else {
        return Ok(());
    };
    for child in siblings {
        let mut next = board.clone();
        apply_node(&mut next, child, width, height)?;
        validate_children(child, &mut next, width, height)?;
    }
    apply_node(board, last, width, height)?;
    validate_children(last, board, width, height)
}
const ANALYSIS: &[&str] = &["LZ", "LZ2", "LZOP", "LZOP2"];

pub(super) fn selected_properties(node: &SgfNode, keys: &[String]) -> Vec<(usize, SgfProperty)> {
    node.properties
        .iter()
        .enumerate()
        .filter(|(_, p)| keys.contains(&p.key))
        .map(|(i, p)| (i, p.clone()))
        .collect()
}
pub(super) fn restore_properties(node: &mut SgfNode, keys: &[String], properties: &[(usize, SgfProperty)]) {
    node.properties.retain(|p| !keys.contains(&p.key));
    for (index, property) in properties {
        node.properties
            .insert((*index).min(node.properties.len()), property.clone());
    }
}
fn clear_analysis(node: &mut SgfNode, path: &mut NodePath, steps: &mut Vec<DocumentReversal>) {
    // Empty slots are part of the inverse too: later analysis belongs to this edited position.
    let keys: Vec<String> = ANALYSIS.iter().map(|s| (*s).into()).collect();
    let properties = selected_properties(node, &keys);
    steps.push(DocumentReversal::AuthorProperties {
        path: path.clone(),
        keys,
        properties,
    });
    node.properties.retain(|p| !ANALYSIS.contains(&p.key.as_str()));
    for (index, child) in node.children.iter_mut().enumerate() {
        path.indices.push(index as u32);
        clear_analysis(child, path, steps);
        path.indices.pop();
    }
}

impl CurrentSgfDocument {
    fn strict_board(&self, path: &NodePath) -> Result<Board, CurrentGameError> {
        let mut board =
            Board::new(self.document.board_width, self.document.board_height).map_err(rule_error)?;
        for node in self.nodes_on_path(path)? {
            apply_node(
                &mut board,
                node,
                self.document.board_width,
                self.document.board_height,
            )?;
        }
        Ok(board)
    }
    fn authoring_outcome(
        &self,
        selected_before: &NodePath,
        selected_after: NodePath,
        reversal: DocumentReversal,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        Ok(DocumentEditOutcome {
            snapshot: self.snapshot(&selected_after)?,
            edit: Some(SgfDocumentEdit {
                reversal,
                selected_before: selected_before.clone(),
                selected_after,
            }),
        })
    }
    pub(super) fn authoring_refresh_metadata(&mut self) {
        if let Some(root) = &self.document.root {
            self.document.black_name = property_values(root, "PB").and_then(|v| v.first()).cloned();
            self.document.white_name = property_values(root, "PW").and_then(|v| v.first()).cloned();
            self.document.result = property_values(root, "RE").and_then(|v| v.first()).cloned();
        }
    }
    /// All validation precedes the first live mutation. The inverse covers only affected content.
    pub fn author_with_history(
        &mut self,
        selected: &NodePath,
        action: SgfAuthoringActionDto,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        self.nodes_on_path(selected)?;
        match action {
            SgfAuthoringActionDto::Add { point, color, insert } => {
                self.author_add(selected, point, color, insert)
            }
            SgfAuthoringActionDto::Drag { from, to } => self.author_drag(selected, from, to),
            SgfAuthoringActionDto::Transform { transform } => self.author_transform(selected, transform),
            SgfAuthoringActionDto::ContinueLadder => self.author_ladder(selected),
        }
    }
    fn author_add(
        &mut self,
        selected: &NodePath,
        point: PointDto,
        color: Option<PlayerColor>,
        insert: bool,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        if point.x >= self.document.board_width || point.y >= self.document.board_height {
            return Err(refused("Move point is outside the board."));
        }
        let mut anchor = selected.clone();
        if insert && anchor.indices.is_empty() {
            let mut node = self.root()?;
            // Frozen ordinary-root rule: skip to the first recorded mainline descendant.
            while !node
                .properties
                .iter()
                .any(|p| matches!(p.key.as_str(), "B" | "W"))
            {
                let Some(child) = node.children.first() else { break };
                anchor.indices.push(0);
                node = child;
            }
            if !node
                .properties
                .iter()
                .any(|p| matches!(p.key.as_str(), "B" | "W"))
            {
                anchor = selected.clone();
            }
        }
        let color = color.unwrap_or(self.snapshot(&anchor)?.position.to_play);
        let mut board = self.strict_board(&anchor)?;
        let mut node = move_node(color, point);
        apply_node(
            &mut board,
            &node,
            self.document.board_width,
            self.document.board_height,
        )?;
        let parent = *self.nodes_on_path(&anchor)?.last().expect("validated anchor");
        if insert {
            validate_children(
                parent,
                &mut board,
                self.document.board_width,
                self.document.board_height,
            )?;
        }
        let index = if insert { 0 } else { parent.children.len() };
        let mut after = anchor.clone();
        after.indices.push(index as u32);
        let mut steps = Vec::new();
        if insert {
            let parent = self.node_mut(&anchor)?;
            node.children = std::mem::take(&mut parent.children);
            parent.children.push(node);
            steps.push(DocumentReversal::SpliceMove {
                parent: anchor,
                properties: Vec::new(),
                inserted: true,
            });
            // Old descendant positions changed; preserve their analysis only in the inverse.
            clear_analysis(self.node_mut(&after)?, &mut after.clone(), &mut steps);
        } else {
            self.node_mut(&anchor)?.children.push(node);
            steps.push(DocumentReversal::RemoveSubtree {
                parent: anchor,
                index,
            });
        }
        self.authoring_outcome(selected, after, DocumentReversal::Sequence(steps))
    }
    fn author_drag(
        &mut self,
        selected: &NodePath,
        from: PointDto,
        to: PointDto,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        if from == to {
            return Ok(DocumentEditOutcome {
                snapshot: self.snapshot(selected)?,
                edit: None,
            });
        }
        let width = self.document.board_width;
        let height = self.document.board_height;
        if to.x >= width || to.y >= height || from.x >= width || from.y >= height {
            return Err(refused("Drag point is outside the board."));
        }
        let snapshot = self.snapshot(selected)?;
        if !snapshot
            .position
            .stones
            .iter()
            .any(|s| s.x == from.x && s.y == from.y)
        {
            return Err(refused(
                "No surviving recorded or starting stone at the drag origin.",
            ));
        }
        if snapshot
            .position
            .stones
            .iter()
            .any(|s| s.x == to.x && s.y == to.y)
        {
            return Err(refused("Drag destination is occupied."));
        }
        let nodes = self.nodes_on_path(selected)?;
        let mut origin = None;
        for (depth, node) in nodes.iter().enumerate().rev() {
            for (index, p) in node.properties.iter().enumerate() {
                let found = match p.key.as_str() {
                    "B" | "W" => p.values.first().is_some_and(|v| {
                        parse_vertex(v, width, height).ok() == Some(MoveVertex::Point(from))
                    }),
                    "AB" | "AW" => p.values.iter().any(|v| {
                        crate::parse_setup_points(v, width, height)
                            .is_ok_and(|points| points.contains(&core_point(&from)))
                    }),
                    _ => false,
                };
                if found {
                    origin = Some((depth, index));
                    break;
                }
            }
            if origin.is_some() {
                break;
            }
        }
        let (depth, index) = origin.ok_or_else(|| refused("The stone has no editable SGF origin."))?;
        let path = NodePath {
            indices: selected.indices[..depth].to_vec(),
        };
        let original = nodes[depth];
        let mut candidate = SgfNode {
            properties: original.properties.clone(),
            children: Vec::new(),
        };
        let p = &mut candidate.properties[index];
        if matches!(p.key.as_str(), "B" | "W") {
            p.values = vec![raw_point(core_point(&to))];
        } else {
            let mut values = Vec::new();
            for v in &p.values {
                let points = crate::parse_setup_points(v, width, height)?;
                if points.contains(&core_point(&from)) {
                    values.extend(points.into_iter().map(|q| {
                        raw_point(if q == core_point(&from) {
                            core_point(&to)
                        } else {
                            q
                        })
                    }));
                } else {
                    values.push(v.clone());
                }
            }
            p.values = values;
        }
        let mut board = Board::new(width, height).map_err(rule_error)?;
        for ancestor in &nodes[..depth] {
            apply_node(&mut board, ancestor, width, height)?;
        }
        let mut origin_position = board.clone();
        apply_node(&mut origin_position, original, width, height)?;
        if origin_position
            .get(core_point(&to))
            .map_err(rule_error)?
            .is_some()
        {
            return Err(refused(
                "Drag destination is occupied at the stone's original position.",
            ));
        }
        apply_node(&mut board, &candidate, width, height)?;
        validate_children(original, &mut board, width, height)?;
        let key = original.properties[index].key.clone();
        let keys = vec![key];
        let properties = selected_properties(original, &keys);
        let mut steps = vec![DocumentReversal::AuthorProperties {
            path: path.clone(),
            keys,
            properties,
        }];
        self.node_mut(&path)?.properties = candidate.properties;
        clear_analysis(self.node_mut(&path)?, &mut path.clone(), &mut steps);
        self.authoring_outcome(selected, selected.clone(), DocumentReversal::Sequence(steps))
    }
    fn author_transform(
        &mut self,
        selected: &NodePath,
        transform: SgfTransformDto,
    ) -> Result<DocumentEditOutcome, CurrentGameError> {
        let width = self.document.board_width;
        let height = self.document.board_height;
        if matches!(
            transform,
            SgfTransformDto::RotateClockwise | SgfTransformDto::RotateCounterclockwise
        ) && width != height
        {
            return Err(refused(
                "Rotation requires a square board; rectangular mirrors are supported.",
            ));
        }
        let mut changes = Vec::new();
        prepare_transform(
            self.root()?,
            &mut NodePath::default(),
            transform,
            width,
            height,
            &mut changes,
        )?;
        if changes.is_empty() {
            return Ok(DocumentEditOutcome {
                snapshot: self.snapshot(selected)?,
                edit: None,
            });
        }
        // Geometry and color exchange preserve rules; validate the original complete tree once.
        let mut board = Board::new(width, height).map_err(rule_error)?;
        apply_node(&mut board, self.root()?, width, height)?;
        validate_children(self.root()?, &mut board, width, height)?;
        let mut steps = Vec::new();
        for (path, properties) in changes {
            let node = self.node_mut(&path)?;
            let mut keys: Vec<String> = node
                .properties
                .iter()
                .filter(|p| transform_key(&p.key, transform))
                .map(|p| p.key.clone())
                .collect();
            for (_, property) in &properties {
                if !keys.contains(&property.key) {
                    keys.push(property.key.clone());
                }
            }
            let old = selected_properties(node, &keys);
            restore_properties(node, &keys, &properties);
            steps.push(DocumentReversal::AuthorProperties {
                path,
                keys,
                properties: old,
            });
        }
        clear_analysis(
            self.node_mut(&NodePath::default())?,
            &mut NodePath::default(),
            &mut steps,
        );
        self.authoring_refresh_metadata();
        self.authoring_outcome(selected, selected.clone(), DocumentReversal::Sequence(steps))
    }
    fn author_ladder(&mut self, selected: &NodePath) -> Result<DocumentEditOutcome, CurrentGameError> {
        let mut recent = Vec::new();
        for node in self.nodes_on_path(selected)?.into_iter().rev() {
            if recent.len() == 5 {
                break;
            }
            let p = node
                .properties
                .iter()
                .find(|p| matches!(p.key.as_str(), "B" | "W"))
                .ok_or_else(|| refused("Ladder history contains a node without a coordinate move."))?;
            let vertex = parse_vertex(
                p.values.first().ok_or(SgfError::Malformed)?,
                self.document.board_width,
                self.document.board_height,
            )?;
            let MoveVertex::Point(point) = vertex else {
                return Err(refused("Ladder history contains a pass."));
            };
            recent.push(point);
        }
        if recent.len() < 5 {
            return Err(refused("Ladder continuation requires five coordinate moves."));
        }
        let mut board = self.strict_board(selected)?;
        let mut color = self.snapshot(selected)?.position.to_play;
        let mut moves = Vec::new();
        loop {
            let next = ladder_next(&recent, &board)?;
            let Some(point) = next else { break };
            board
                .play(to_core_color(color), go_core::Vertex::Point(core_point(&point)))
                .map_err(rule_error)?;
            moves.push(move_node(color, point));
            recent.rotate_right(1);
            recent[0] = point;
            color = color.opponent();
        }
        if moves.is_empty() {
            return Err(refused(
                "Ladder pattern is non-diagonal, blocked, or outside the board.",
            ));
        }
        let count = moves.len();
        let mut line = moves.pop().expect("nonempty continuation");
        while let Some(mut parent) = moves.pop() {
            parent.children.push(line);
            line = parent;
        }
        let index = self.node_mut(selected)?.children.len();
        self.node_mut(selected)?.children.push(line);
        let mut after = selected.clone();
        after.indices.push(index as u32);
        after.indices.extend(std::iter::repeat_n(0, count - 1));
        self.authoring_outcome(
            selected,
            after,
            DocumentReversal::RemoveSubtree {
                parent: selected.clone(),
                index,
            },
        )
    }
}

fn ladder_next(recent: &[PointDto], board: &Board) -> Result<Option<PointDto>, CurrentGameError> {
    // Board.java@7b40275:6793-6849: pastMove[0]-pastMove[4], next=pastMove[3]+delta.
    let dx = i16::from(recent[0].x) - i16::from(recent[4].x);
    let dy = i16::from(recent[0].y) - i16::from(recent[4].y);
    if dx.abs() != 1 || dy.abs() != 1 {
        return Ok(None);
    }
    let x = i16::from(recent[3].x) + dx;
    let y = i16::from(recent[3].y) + dy;
    for (x, y) in [(x, y), (x + dx, y), (x, y + dy)] {
        if x < 0 || y < 0 || x >= i16::from(board.width()) || y >= i16::from(board.height()) {
            return Ok(None);
        }
        if board
            .get(go_core::Point {
                x: x as u8,
                y: y as u8,
            })
            .map_err(rule_error)?
            .is_some()
        {
            return Ok(None);
        }
    }
    Ok(Some(PointDto {
        x: x as u8,
        y: y as u8,
    }))
}
fn transform_key(key: &str, transform: SgfTransformDto) -> bool {
    if transform == SgfTransformDto::SwapColors {
        matches!(
            key,
            "B" | "W"
                | "AB"
                | "AW"
                | "PL"
                | "PB"
                | "PW"
                | "BR"
                | "WR"
                | "BT"
                | "WT"
                | "BL"
                | "WL"
                | "OB"
                | "OW"
                | "TB"
                | "TW"
                | "RE"
        )
    } else {
        matches!(
            key,
            "B" | "W"
                | "AB"
                | "AW"
                | "AE"
                | "LB"
                | "CR"
                | "SQ"
                | "MA"
                | "TR"
                | "SL"
                | "DD"
                | "VW"
                | "TB"
                | "TW"
                | "AR"
                | "LN"
        )
    }
}
fn transformed_point(p: go_core::Point, transform: SgfTransformDto, width: u8, height: u8) -> go_core::Point {
    match transform {
        SgfTransformDto::RotateClockwise => go_core::Point {
            x: width - 1 - p.y,
            y: p.x,
        },
        SgfTransformDto::RotateCounterclockwise => go_core::Point {
            x: p.y,
            y: height - 1 - p.x,
        },
        SgfTransformDto::MirrorHorizontal => go_core::Point {
            x: width - 1 - p.x,
            y: p.y,
        },
        SgfTransformDto::MirrorVertical => go_core::Point {
            x: p.x,
            y: height - 1 - p.y,
        },
        SgfTransformDto::SwapColors => p,
    }
}
fn prepare_transform(
    node: &SgfNode,
    path: &mut NodePath,
    transform: SgfTransformDto,
    width: u8,
    height: u8,
    changes: &mut Vec<(NodePath, Vec<(usize, SgfProperty)>)>,
) -> Result<(), CurrentGameError> {
    let mut properties = node
        .properties
        .iter()
        .enumerate()
        .filter(|(_, p)| transform_key(&p.key, transform))
        .map(|(index, p)| (index, p.clone()))
        .collect::<Vec<_>>();
    for (_, p) in &mut properties {
        if transform == SgfTransformDto::SwapColors {
            p.key = match p.key.as_str() {
                "B" => "W",
                "W" => "B",
                "AB" => "AW",
                "AW" => "AB",
                "PB" => "PW",
                "PW" => "PB",
                "BR" => "WR",
                "WR" => "BR",
                "BT" => "WT",
                "WT" => "BT",
                "BL" => "WL",
                "WL" => "BL",
                "OB" => "OW",
                "OW" => "OB",
                "TB" => "TW",
                "TW" => "TB",
                k => k,
            }
            .into();
            if p.key == "PL" {
                for v in &mut p.values {
                    *v = match v.as_str() {
                        "B" => "W",
                        "W" => "B",
                        _ => return Err(refused("Cannot safely exchange malformed PL.")),
                    }
                    .into();
                }
            } else if p.key == "RE" {
                for v in &mut p.values {
                    if v.starts_with("B+") {
                        v.replace_range(..1, "W");
                    } else if v.starts_with("W+") {
                        v.replace_range(..1, "B");
                    } else if !matches!(v.as_str(), "0" | "Draw" | "Void" | "?") {
                        return Err(refused("Cannot safely exchange this result."));
                    }
                }
            }
        } else {
            let mut values = Vec::new();
            for v in &p.values {
                let mapped = |raw: &str| -> Result<String, CurrentGameError> {
                    Ok(raw_point(transformed_point(
                        crate::parse_point(raw, width, height)?,
                        transform,
                        width,
                        height,
                    )))
                };
                match p.key.as_str() {
                    "B" | "W" => match parse_vertex(v, width, height)? {
                        MoveVertex::Pass => values.push(v.clone()),
                        MoveVertex::Point(q) => values.push(raw_point(transformed_point(
                            core_point(&q),
                            transform,
                            width,
                            height,
                        ))),
                    },
                    "LB" => {
                        let (point, text) = v
                            .split_once(':')
                            .ok_or_else(|| refused("Cannot safely transform malformed label."))?;
                        values.push(format!("{}:{text}", mapped(point)?));
                    }
                    "AR" | "LN" => {
                        let (a, b) = v
                            .split_once(':')
                            .ok_or_else(|| refused("Cannot safely transform malformed line or arrow."))?;
                        values.push(format!("{}:{}", mapped(a)?, mapped(b)?));
                    }
                    _ => {
                        if v.is_empty() && matches!(p.key.as_str(), "VW" | "DD") {
                            values.push(v.clone());
                            continue;
                        }
                        let (a, b) = crate::parse_setup_point_bounds(v, width, height)?;
                        let a = transformed_point(a, transform, width, height);
                        if let Some(b) = b {
                            let b = transformed_point(b, transform, width, height);
                            values.push(format!(
                                "{}:{}",
                                raw_point(go_core::Point {
                                    x: a.x.min(b.x),
                                    y: a.y.min(b.y)
                                }),
                                raw_point(go_core::Point {
                                    x: a.x.max(b.x),
                                    y: a.y.max(b.y)
                                })
                            ));
                        } else {
                            values.push(raw_point(a));
                        }
                    }
                }
            }
            p.values = values;
        }
    }
    if properties
        .iter()
        .any(|(index, property)| property != &node.properties[*index])
    {
        changes.push((path.clone(), properties));
    }
    for (index, child) in node.children.iter().enumerate() {
        path.indices.push(index as u32);
        prepare_transform(child, path, transform, width, height, changes)?;
        path.indices.pop();
    }
    Ok(())
}
