//! READ-02 readboard synchronisation onto the current SGF document.
//!
//! Port of the observable results of the frozen Java `ReadBoard.syncBoardStones` chain
//! (`7b40275`: ReadBoard.java 1513–2780, SyncSnapshotClassifier, SyncSnapshotRebuildPolicy,
//! SyncConflictTracker). Java node kinds map onto SGF as: B/W nodes are MOVE/PASS history
//! actions; the root and setup-only nodes are SNAPSHOT anchors. Java per-node metadata that
//! SGF does not store (a rebuilt snapshot's inferred move number and marker) lives in this
//! session memory; the owner blocks local edits while a session exists, so paths stay valid.
//! Read-only: never PASS, never local-move, GMA or engine branches.

use super::CurrentSgfDocument;
use crate::{
    apply_setup_properties, parse_vertex, player_to_play, serialize_vertex, SgfDocument, SgfNode, SgfProperty,
};
use app_model::{CurrentGameError, CurrentGameErrorKind, MoveVertex, NodePath, PlayerColor, PointDto};
use go_core::{
    Board, Color, Point, ReadboardControl, ReadboardFrame, ReadboardLastMoveSource, ReadboardPlatform,
    ReadboardRemoteContext, ReadboardWindowKind, Vertex,
};
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadboardViewPreferences {
    pub always_sync: bool,
    pub jump_to_last: bool,
}

#[derive(Debug, Clone)]
pub enum ReadboardSyncOutcome {
    /// Frame not applied and state unchanged (Yike-platform frame, Java skips it).
    Ignored,
    /// Conflict observed for the first time; document and cursor unchanged.
    Hold,
    /// `document` is Some only when SGF content changed (moves appended or snapshot rebuild);
    /// boxed so the frequent Hold/Ignored outcomes stay small.
    Accepted {
        document: Option<Box<CurrentSgfDocument>>,
        selected: NodePath,
        source: NodePath,
        source_move_number: u32,
        rebuilt: bool,
    },
}

/// Java metadata of a snapshot root this session rebuilt (not representable in SGF).
#[derive(Debug, Clone)]
struct RootSnapshot {
    move_number: u32,
    marker: Option<Marker>,
    /// Java `BoardData.properties` copied from the anchor (explicit PL/MN, setup keys, ...).
    properties: Vec<SgfProperty>,
    has_start_stone: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Marker {
    x: u8,
    y: u8,
    color: Color,
}

#[derive(Debug, Clone)]
pub struct ReadboardSync {
    /// `SyncConflictTracker`: the key observed once and held.
    held_conflict: Option<String>,
    awaiting_first_frame: bool,
    /// `SyncResumeState`: last resolved node and its context (without forceRebuild).
    resume: Option<(NodePath, ReadboardRemoteContext)>,
    /// Node the source showed after the previous accepted frame (cursor-follow rule).
    source: Option<NodePath>,
    root: Option<RootSnapshot>,
    /// Java reopen/clearBoard: the next frame is compared against an empty board.
    fresh_board_next_frame: bool,
}

impl Default for ReadboardSync {
    fn default() -> Self {
        Self {
            held_conflict: None,
            awaiting_first_frame: true,
            resume: None,
            source: None,
            root: None,
            fresh_board_next_frame: false,
        }
    }
}

/// Java `BoardData` view of one node on a replayed path.
#[derive(Debug, Clone)]
struct NodeInfo {
    stones: Vec<Option<Color>>,
    move_number: u32,
    black_to_play: bool,
    last_move: Option<Marker>,
    /// Java SNAPSHOT anchor: the root or a setup-only node.
    snapshot: bool,
    /// Java MOVE/PASS: any B/W node.
    history_action: bool,
    explicit_pl: Option<bool>,
    /// `hasSetupOrHandicapTurnRisk` ancestry part: start stones or setup/handicap properties at or above.
    setup_risk: bool,
}

/// Java `SyncSnapshotClassifier.SnapshotDelta`.
#[derive(Debug, Clone, Copy)]
struct Delta {
    valid: bool,
    additions: usize,
    removals: usize,
    move_x: u8,
    move_y: u8,
    move_color: Option<Color>,
    marker_valid: bool,
    marker: Option<Marker>,
    /// The frame's single last-move marker, found over the whole frame. `marker` follows Java and is
    /// only recorded up to the first black/white flip; this field lets the side to play use a trusted
    /// marker that appears after that flip (Next deviation from frozen Java, see `turn_signal`).
    frame_marker: Option<Marker>,
}

impl Delta {
    fn summarize(current: &[Option<Color>], codes: &[u8], width: u8) -> Self {
        let frame_marker = unique_marker(codes, width);
        let mut delta = Self {
            valid: current.len() == codes.len(),
            additions: 0,
            removals: 0,
            move_x: 0,
            move_y: 0,
            move_color: None,
            marker_valid: true,
            marker: None,
            frame_marker,
        };
        if !delta.valid {
            return delta;
        }
        for (index, (&stone, &code)) in current.iter().zip(codes).enumerate() {
            let (x, y) = ((index % width as usize) as u8, (index / width as usize) as u8);
            if let Some(color) = marker_color(code) {
                if delta.marker.is_some() {
                    delta.marker_valid = false;
                } else {
                    delta.marker = Some(Marker { x, y, color });
                }
            }
            match (stone, code_color(code)) {
                (a, b) if a == b => {}
                (None, Some(color)) => {
                    delta.additions += 1;
                    if delta.move_color.is_none() {
                        (delta.move_x, delta.move_y, delta.move_color) = (x, y, Some(color));
                    }
                }
                (Some(_), None) => delta.removals += 1,
                _ => {
                    delta.valid = false;
                    return delta;
                }
            }
        }
        delta
    }

    fn has_marker(&self) -> bool {
        self.marker_valid && self.marker.is_some()
    }

    fn marker_matches_single_addition(&self) -> bool {
        self.marker.is_some_and(|marker| {
            (marker.x, marker.y, Some(marker.color)) == (self.move_x, self.move_y, self.move_color)
        })
    }

    fn allows_incremental_sync(&self) -> bool {
        if !self.valid || !self.marker_valid || self.removals > 0 {
            return false;
        }
        if self.additions == 0 {
            return self.marker.is_none();
        }
        self.additions == 1 && self.marker.is_some() && self.marker_matches_single_addition()
    }

    fn has_only_additions(&self) -> bool {
        self.valid && self.removals == 0
    }

    fn changed_stones(&self) -> usize {
        self.additions + self.removals
    }

    fn single_addition_color(&self) -> Option<Color> {
        (self.additions == 1).then_some(self.move_color).flatten()
    }

    /// Removals of the mover's own colour are rejected separately by `own_stone_removed`.
    fn is_single_move_capture(&self) -> bool {
        self.valid
            && self.marker_valid
            && self.additions == 1
            && self.move_color.is_some()
            && (self.marker.is_none() || self.marker_matches_single_addition())
    }
}

enum Recovery {
    Hold,
    Rebuild,
    NoChange(NodePath),
    SingleMove(CurrentSgfDocument, NodePath),
}

impl ReadboardSync {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start (any size), Clear, StopSync and EndSync are Java's control-line reset; a changed board
    /// size is detected against the document when the next frame arrives (Java reopen).
    pub fn control(&mut self, control: ReadboardControl) {
        match control {
            ReadboardControl::Sync => {}
            ReadboardControl::Start { .. }
            | ReadboardControl::Clear
            | ReadboardControl::StopSync
            | ReadboardControl::EndSync => self.reset_active(),
            ReadboardControl::ClearBoard => {
                self.reset_active();
                self.fresh_board_next_frame = true;
            }
        }
    }

    fn reset_active(&mut self) {
        self.held_conflict = None;
        self.awaiting_first_frame = true;
    }

    pub fn apply(
        &mut self,
        document: Option<&CurrentSgfDocument>,
        selected: &NodePath,
        frame: &ReadboardFrame,
        view: ReadboardViewPreferences,
    ) -> Result<ReadboardSyncOutcome, CurrentGameError> {
        if frame.context.platform == ReadboardPlatform::Yike {
            return Ok(ReadboardSyncOutcome::Ignored);
        }
        if frame.codes.len() != frame.width as usize * frame.height as usize
            || frame.codes.iter().any(|&code| code > 4)
        {
            return Err(invalid("readboard frame does not match its board size"));
        }
        let mut next = self.clone();
        let outcome = next.sync(document, selected, frame, view)?;
        *self = next;
        Ok(outcome)
    }

    fn sync(
        &mut self,
        document: Option<&CurrentSgfDocument>,
        selected: &NodePath,
        frame: &ReadboardFrame,
        view: ReadboardViewPreferences,
    ) -> Result<ReadboardSyncOutcome, CurrentGameError> {
        let reopen = std::mem::take(&mut self.fresh_board_next_frame)
            || document
                .is_none_or(|doc| (doc.board_width(), doc.board_height()) != (frame.width, frame.height));
        let fresh;
        let (base, mut changed) = if reopen {
            // Java `start` with another size / clearBoard: an empty board, no resume state.
            self.resume = None;
            self.root = None;
            fresh = empty_document(frame.width, frame.height)?;
            (&fresh, true)
        } else {
            (document.expect("document present"), false)
        };
        let selected = if reopen {
            NodePath::default()
        } else {
            selected.clone()
        };
        let ctx = &frame.context;
        let main_end = base.default_selected_path();
        let main_line = self.line(base, &main_end)?;
        let start = main_line.last().expect("path includes root");
        let delta = Delta::summarize(&start.stones, &frame.codes, frame.width);

        // Copy the document only once a frame actually changes it; unchanged and held frames borrow.
        let mut work = Cow::Borrowed(base);
        let mut played = false;
        let mut need_resync = true;
        let fox_step = ctx.supports_fox_recovery()
            && delta.has_only_additions()
            && delta.additions == 1
            && ctx.recovery_move_number() == Some(start.move_number + 1);
        if delta.allows_incremental_sync() || fox_step {
            need_resync = false;
            if let Some(color) = delta.single_addition_color() {
                let point = Point {
                    x: delta.move_x,
                    y: delta.move_y,
                };
                match play_on(&start.stones, frame.width, frame.height, color, point) {
                    Some(after)
                        if after == frame_stones(&frame.codes) && !repeats_previous(&main_line, &after) =>
                    {
                        append_move(work.to_mut(), &main_end, color, point)?;
                        played = true;
                    }
                    _ => need_resync = true,
                }
            }
        }

        let mut single_move = false;
        let mut source = None;
        if need_resync {
            match self.recover(&work, &selected, &main_end, &main_line, frame, &delta)? {
                Recovery::Hold => return Ok(ReadboardSyncOutcome::Hold),
                Recovery::Rebuild => return self.rebuild(base, &main_line, frame, &delta, &selected, view),
                Recovery::NoChange(node) => {
                    if node != selected || self.awaiting_first_frame {
                        self.resume = Some((node.clone(), ctx.clone().without_force_rebuild()));
                        self.awaiting_first_frame = false;
                    }
                    source = Some(node);
                }
                Recovery::SingleMove(document, end) => {
                    work = Cow::Owned(document);
                    single_move = true;
                    played = true;
                    self.resume = Some((end.clone(), ctx.clone().without_force_rebuild()));
                    self.awaiting_first_frame = false;
                    source = Some(end);
                }
            }
        }
        let sync_end = work.default_selected_path();
        if played && !single_move {
            self.resume = Some((sync_end.clone(), ctx.clone().without_force_rebuild()));
            self.awaiting_first_frame = false;
        }
        changed |= played;
        let end_line = if played {
            Some(self.line(&work, &sync_end)?)
        } else {
            None
        };
        let end_line = end_line.as_deref().unwrap_or(&main_line);
        if self.should_rebuild_for_fox_metadata(end_line.last().expect("root"), ctx, &frame.codes, &delta) {
            return self.rebuild(&work, end_line, frame, &delta, &selected, view);
        }
        self.held_conflict = None;
        self.awaiting_first_frame = false;
        let source = source.unwrap_or_else(|| sync_end.clone());
        let source_move_number = if source == sync_end {
            end_line.last().expect("root").move_number
        } else {
            self.line(&work, &source)?.last().expect("root").move_number
        };
        let selected = self.cursor(&work, &selected, &source, changed, view);
        self.source = Some(source.clone());
        Ok(ReadboardSyncOutcome::Accepted {
            document: changed.then(|| Box::new(work.into_owned())),
            selected,
            source,
            source_move_number,
            rebuilt: false,
        })
    }

    /// Java `resolveCompleteSnapshotRecovery` (pending-local-move hold excluded).
    fn recover(
        &mut self,
        work: &CurrentSgfDocument,
        current: &NodePath,
        start_path: &NodePath,
        main_line: &[NodeInfo],
        frame: &ReadboardFrame,
        delta: &Delta,
    ) -> Result<Recovery, CurrentGameError> {
        let ctx = &frame.context;
        if ctx.force_rebuild {
            return Ok(Recovery::Rebuild);
        }
        if ctx.supports_fox_recovery()
            && self
                .resume
                .as_ref()
                .is_some_and(|(_, resumed)| resumed.conflicts_with(ctx))
        {
            return Ok(Recovery::Rebuild);
        }
        let width = frame.width;
        let matched = if ctx.supports_fox_recovery() {
            self.match_in_mainline_window(work, current, start_path, main_line, &frame.codes, width, ctx)?
        } else {
            matches_identity(main_line.last().expect("root"), &frame.codes, width, ctx)
                .then(|| start_path.clone())
        };
        if let Some(node) = matched {
            return Ok(Recovery::NoChange(node));
        }
        if let Some(node) = self.adjacent_match(work, &frame.codes, width, ctx)? {
            return Ok(Recovery::NoChange(node));
        }
        if delta.has_marker() {
            if let Some((document, end)) = single_move_recovery(work, start_path, main_line, frame)? {
                return Ok(Recovery::SingleMove(document, end));
            }
        }
        let rebuild_now = if self.awaiting_first_frame {
            ctx.supports_fox_recovery()
        } else {
            ctx.supports_fox_recovery()
                && ctx.recovery_move_number() != Some(main_line.last().expect("root").move_number)
        };
        if rebuild_now || main_line.len() == 1 {
            // `shouldRebuildImmediatelyWithoutHistory`: a sync start without a previous node.
            return Ok(Recovery::Rebuild);
        }
        let key = conflict_key(&frame.codes, ctx);
        if self.held_conflict.as_deref() == Some(key.as_str()) {
            self.held_conflict = None;
            return Ok(Recovery::Rebuild);
        }
        self.held_conflict = Some(key);
        Ok(Recovery::Hold)
    }

    /// `findMatchingNodeInMainlineWindow`: current node and its ancestors, then the main trunk
    /// forward from the current node to the main end.
    #[allow(clippy::too_many_arguments)]
    fn match_in_mainline_window(
        &self,
        work: &CurrentSgfDocument,
        current: &NodePath,
        main_end: &NodePath,
        main_line: &[NodeInfo],
        codes: &[u8],
        width: u8,
        ctx: &ReadboardRemoteContext,
    ) -> Result<Option<NodePath>, CurrentGameError> {
        let current_line = self.line(work, current)?;
        if let Some(depth) = current_line
            .iter()
            .rposition(|info| matches_identity(info, codes, width, ctx))
        {
            return Ok(Some(prefix(current, depth)));
        }
        if !on_main_trunk(current) {
            return Ok(None);
        }
        let start = current.indices.len() + 1;
        Ok((start..main_line.len())
            .find(|&depth| matches_identity(&main_line[depth], codes, width, ctx))
            .map(|depth| prefix(main_end, depth)))
    }

    /// `findAdjacentMatchFromLastResolvedNode`.
    fn adjacent_match(
        &self,
        work: &CurrentSgfDocument,
        codes: &[u8],
        width: u8,
        ctx: &ReadboardRemoteContext,
    ) -> Result<Option<NodePath>, CurrentGameError> {
        let Some((node, resumed)) = self.resume.as_ref() else {
            return Ok(None);
        };
        if !ctx.supports_fox_recovery() || resumed.conflicts_with(ctx) || !on_main_trunk(node) {
            return Ok(None);
        }
        let mut next = node.clone();
        next.indices.push(0);
        if work.nodes_on_path(&next).is_err() {
            return Ok(None);
        }
        let line = self.line(work, &next)?;
        Ok(matches_identity(line.last().expect("root"), codes, width, ctx).then_some(next))
    }

    /// `shouldRebuildForFoxMetadataChange` on the sync end after a non-rebuilding frame.
    fn should_rebuild_for_fox_metadata(
        &self,
        end: &NodeInfo,
        ctx: &ReadboardRemoteContext,
        codes: &[u8],
        delta: &Delta,
    ) -> bool {
        let Some(fox) = ctx.recovery_move_number().filter(|_| ctx.supports_fox_recovery()) else {
            return false;
        };
        if end.stones != frame_stones(codes) || end.history_action {
            return false;
        }
        let expected = end
            .explicit_pl
            .or_else(|| self.turn_signal(&frame_stones(codes), delta, ctx, Some(end)));
        end.move_number != fox || expected.is_some_and(|black| black != end.black_to_play)
    }

    /// Turn signals shared by `expectedBlackToPlayForFoxMetadataChange` and
    /// `inferSnapshotBlackToPlay` after their own first checks. A trusted marker is read from the
    /// whole frame: after a target change the Java delta scan stops at the first black/white flip
    /// and would otherwise drop it, leaving the previous target's side to play.
    fn turn_signal(
        &self,
        stones: &[Option<Color>],
        delta: &Delta,
        ctx: &ReadboardRemoteContext,
        start: Option<&NodeInfo>,
    ) -> Option<bool> {
        let fox = ctx.recovery_move_number();
        if fox_zero_handicap(stones, delta, ctx, fox) {
            return Some(false);
        }
        if let Some(black) = trusted_marker_turn(delta, ctx) {
            return Some(black);
        }
        if self.ordinary_fox_fallback(start, delta, ctx.last_move_source, fox) {
            return Some(fox.expect("fox number") % 2 == 0);
        }
        None
    }

    /// `isMarkerlessOrdinaryFoxTurnFallback`.
    fn ordinary_fox_fallback(
        &self,
        start: Option<&NodeInfo>,
        delta: &Delta,
        source: ReadboardLastMoveSource,
        fox: Option<u32>,
    ) -> bool {
        let Some(fox) = fox else { return false };
        let expected = if fox % 2 == 1 { Color::Black } else { Color::White };
        !delta.has_marker()
            && matches!(
                source,
                ReadboardLastMoveSource::None | ReadboardLastMoveSource::LegacyUnknown
            )
            && delta.removals == 0
            && delta.additions == 1
            && delta.single_addition_color() == Some(expected)
            && !self.setup_risk(start, delta)
    }

    /// `hasSetupOrHandicapTurnRisk` (walks the sync-start ancestry recorded in `line`).
    fn setup_risk(&self, start: Option<&NodeInfo>, delta: &Delta) -> bool {
        delta.removals > 0 || delta.additions > 1 || start.is_some_and(|start| start.setup_risk)
    }

    /// Java `rebuildFromSnapshot`: one static SNAPSHOT root replaces the history; GameInfo and
    /// the nearest snapshot anchor's comment/properties survive.
    fn rebuild(
        &mut self,
        previous: &CurrentSgfDocument,
        start_line: &[NodeInfo],
        frame: &ReadboardFrame,
        delta: &Delta,
        selected: &NodePath,
        view: ReadboardViewPreferences,
    ) -> Result<ReadboardSyncOutcome, CurrentGameError> {
        let ctx = frame.context.clone().without_force_rebuild();
        let start = start_line.last().expect("root");
        let stones = frame_stones(&frame.codes);
        let fox = ctx.recovery_move_number();
        let move_number = infer_move_number(start, &stones, delta, fox);
        let mut black_to_play = if stones.iter().all(Option::is_none) {
            true
        } else {
            self.turn_signal(&stones, delta, &ctx, Some(start))
                .unwrap_or(start.black_to_play)
        };
        let start_path = previous.default_selected_path();
        let anchor_depth = start_line.iter().rposition(|info| info.snapshot);
        let nodes = previous.nodes_on_path(&start_path)?;
        let (mut anchor_properties, anchor_comment) = match anchor_depth {
            Some(0) if self.root.is_some() => (
                self.root.as_ref().expect("root").properties.clone(),
                comment(nodes[0]),
            ),
            Some(depth) => (java_properties(nodes[depth]), comment(nodes[depth])),
            None => (Vec::new(), None),
        };
        // Next deviation: the frame's trusted marker describes the new board directly, so it wins over
        // a PL inherited from the old anchor (which may be an earlier session's materialised turn).
        // The overridden PL is dropped so later frames do not read it back as explicit.
        if stones.iter().any(Option::is_some) && trusted_marker_turn(delta, &ctx).is_some() {
            anchor_properties.retain(|property| property.key != "PL");
        } else if let Some(explicit) = pl_value(&anchor_properties) {
            black_to_play = explicit;
        }
        let had_start_stone = self.has_start_stone(nodes[0]);
        let mut root = SgfNode {
            properties: Vec::new(),
            children: Vec::new(),
        };
        for property in nodes[0]
            .properties
            .iter()
            .filter(|property| GAME_INFO.contains(&property.key.as_str()))
        {
            root.properties.push(property.clone());
        }
        for property in &anchor_properties {
            if !matches!(property.key.as_str(), "AB" | "AW" | "AE" | "PL" | "C" | "SZ")
                && !root
                    .properties
                    .iter()
                    .any(|existing| existing.key == property.key)
            {
                root.properties.push(property.clone());
            }
        }
        for (color, key) in [(Color::Black, "AB"), (Color::White, "AW")] {
            let values: Vec<String> = stones
                .iter()
                .enumerate()
                .filter(|(_, stone)| **stone == Some(color))
                .map(|(index, _)| coordinate(index, frame.width))
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
            values: vec![if black_to_play { "B" } else { "W" }.into()],
        });
        if let Some(text) = anchor_comment {
            root.properties.push(SgfProperty {
                key: "C".into(),
                values: vec![text],
            });
        }
        let rebuilt = document_from_root(root, frame.width, frame.height, previous.komi())?;
        self.root = Some(RootSnapshot {
            move_number,
            marker: delta.has_marker().then_some(delta.marker).flatten(),
            properties: anchor_properties,
            has_start_stone: had_start_stone && anchor_depth.is_none_or(|depth| depth == 0),
        });
        self.resume = Some((NodePath::default(), ctx));
        self.held_conflict = None;
        self.awaiting_first_frame = false;
        let source = NodePath::default();
        let selected = self.cursor(&rebuilt, selected, &source, true, view);
        self.source = Some(source.clone());
        Ok(ReadboardSyncOutcome::Accepted {
            document: Some(Box::new(rebuilt)),
            selected,
            source,
            source_move_number: move_number,
            rebuilt: true,
        })
    }

    /// Next mapping of Java alwaysSyncBoardStat / alwaysGotoLastOnLive: the view cursor follows the
    /// source only when asked to; otherwise it stays on a node that still exists.
    fn cursor(
        &self,
        document: &CurrentSgfDocument,
        selected: &NodePath,
        source: &NodePath,
        changed: bool,
        view: ReadboardViewPreferences,
    ) -> NodePath {
        let moved = changed || self.source.as_ref() != Some(source);
        let follows = self.source.is_none()
            || view.jump_to_last
            || (view.always_sync && self.source.as_ref() == Some(selected));
        if (moved && follows) || document.nodes_on_path(selected).is_err() {
            source.clone()
        } else {
            selected.clone()
        }
    }

    fn has_start_stone(&self, root: &SgfNode) -> bool {
        match &self.root {
            Some(snapshot) => snapshot.has_start_stone,
            None => root
                .properties
                .iter()
                .any(|property| matches!(property.key.as_str(), "AB" | "AW") && !property.values.is_empty()),
        }
    }

    /// Replays `path` once, returning the Java node view of every node on it.
    fn line(
        &self,
        document: &CurrentSgfDocument,
        path: &NodePath,
    ) -> Result<Vec<NodeInfo>, CurrentGameError> {
        let (width, height) = (document.board_width(), document.board_height());
        let nodes = document.nodes_on_path(path)?;
        let mut board = Board::new(width, height).map_err(|_| invalid("unsupported board size"))?;
        let handicap_root = nodes[0]
            .properties
            .iter()
            .any(|p| p.key == "AB" && !p.values.is_empty())
            && !nodes[0]
                .properties
                .iter()
                .any(|p| matches!(p.key.as_str(), "AW" | "AE"));
        let mut black_to_play = !handicap_root;
        let mut moves = 0u32;
        let mut risk = self.has_start_stone(nodes[0]);
        let mut infos = Vec::with_capacity(nodes.len());
        for (depth, node) in nodes.iter().enumerate() {
            apply_setup_properties(&mut board, node, width, height)?;
            if let Some(color) = player_to_play(node)? {
                black_to_play = color == PlayerColor::Black;
            }
            let mut action = false;
            let mut last = None;
            for property in &node.properties {
                let color = match property.key.as_str() {
                    "B" => Color::Black,
                    "W" => Color::White,
                    _ => continue,
                };
                action = true;
                moves += 1;
                let raw = property.values.first().map(String::as_str).unwrap_or("");
                if let MoveVertex::Point(point) = parse_vertex(raw, width, height)? {
                    let point = Point {
                        x: point.x,
                        y: point.y,
                    };
                    // Replay tolerates illegal stored moves like the document projection does.
                    let _ = board.play(color, Vertex::Point(point));
                    last = Some(Marker {
                        x: point.x,
                        y: point.y,
                        color,
                    });
                }
                black_to_play = match player_to_play(node)? {
                    Some(explicit) => explicit == PlayerColor::Black,
                    None => color == Color::White,
                };
            }
            let engine_root = depth == 0 && self.root.is_some();
            let properties = if engine_root {
                &self.root.as_ref().expect("root").properties
            } else {
                &node.properties
            };
            risk |= properties
                .iter()
                .any(|p| matches!(p.key.as_str(), "AB" | "AW" | "AE" | "PL" | "HA"));
            infos.push(NodeInfo {
                stones: board.stones_snapshot(),
                move_number: moves + self.root.as_ref().map_or(0, |root| root.move_number),
                black_to_play,
                last_move: if action {
                    last
                } else if engine_root {
                    self.root.as_ref().expect("root").marker
                } else {
                    None
                },
                // Java SGFParser: a move-less node with AB/AW/AE or PL becomes a setup SNAPSHOT.
                snapshot: !action
                    && (depth == 0
                        || node
                            .properties
                            .iter()
                            .any(|p| matches!(p.key.as_str(), "AB" | "AW" | "AE" | "PL"))),
                history_action: action,
                explicit_pl: pl_value(properties),
                setup_risk: risk,
            });
        }
        Ok(infos)
    }
}

/// Java GameInfo fields carried by `rebuiltHistory.setGameInfo(previousHistory.getGameInfo())`.
const GAME_INFO: &[&str] = &[
    "GM", "FF", "CA", "AP", "KM", "HA", "PB", "PW", "BR", "WR", "BT", "WT", "RE", "DT", "EV", "RO", "PC",
    "GN", "RU", "TM", "OT", "SO", "US", "AN", "CP", "GC", "ON",
];

fn infer_move_number(start: &NodeInfo, stones: &[Option<Color>], delta: &Delta, fox: Option<u32>) -> u32 {
    if let Some(fox) = fox {
        return fox;
    }
    if stones.iter().all(Option::is_none) {
        return 0;
    }
    let Some(marker) = delta.marker.filter(|_| delta.has_marker()) else {
        return start.move_number;
    };
    if delta.changed_stones() == 0 {
        return start.move_number;
    }
    if delta.has_only_additions() {
        return start.move_number + delta.additions as u32;
    }
    let occupied = stones.iter().filter(|stone| stone.is_some()).count() as u32;
    let black_to_play = marker.color == Color::White;
    if (occupied % 2 == 0) == black_to_play {
        occupied
    } else {
        occupied + 1
    }
}

/// `isTrustedFoxZeroMoveHandicapSetupTurn`.
fn fox_zero_handicap(
    stones: &[Option<Color>],
    delta: &Delta,
    ctx: &ReadboardRemoteContext,
    fox: Option<u32>,
) -> bool {
    ctx.platform == ReadboardPlatform::Fox
        && fox == Some(0)
        && !delta.has_marker()
        && !ctx.last_move_source.is_trusted_visual_marker()
        && !stones.contains(&Some(Color::White))
        && stones
            .iter()
            .filter(|stone| **stone == Some(Color::Black))
            .count()
            >= 2
}

/// `SyncSnapshotRebuildPolicy.matchesRemoteIdentity`.
fn matches_identity(info: &NodeInfo, codes: &[u8], width: u8, ctx: &ReadboardRemoteContext) -> bool {
    let fox = ctx.supports_fox_recovery();
    let stones_match = info.stones.len() == codes.len()
        && codes
            .iter()
            .zip(&info.stones)
            .all(|(&code, &stone)| (fox && matches!(code, 3 | 4)) || code_color(code) == stone);
    if !stones_match {
        return false;
    }
    if fox {
        return ctx.recovery_move_number() == Some(info.move_number);
    }
    let mut marker = None;
    for (index, &code) in codes.iter().enumerate() {
        if let Some(color) = marker_color(code) {
            if marker.is_some() {
                return false;
            }
            marker = Some(Marker {
                x: (index % width as usize) as u8,
                y: (index / width as usize) as u8,
                color,
            });
        }
    }
    marker.is_none_or(|marker| info.last_move == Some(marker))
}

/// `SyncSnapshotRebuildPolicy.buildConflictKey`.
fn conflict_key(codes: &[u8], ctx: &ReadboardRemoteContext) -> String {
    let fox = ctx.recovery_move_number().filter(|_| ctx.supports_fox_recovery());
    let mut key: String = codes
        .iter()
        .map(|&code| match (code, fox) {
            (3 | 4, Some(number)) => {
                if number % 2 == 0 {
                    '2'
                } else {
                    '1'
                }
            }
            _ => match code_color(code) {
                Some(Color::Black) => '1',
                Some(Color::White) => '2',
                None => '0',
            },
        })
        .collect();
    key.push_str(&format!("|{:?}|{:?}", ctx.platform, ctx.window_kind));
    if let Some(number) = fox {
        key.push_str(&format!("|{number}"));
    }
    match ctx.window_kind {
        ReadboardWindowKind::LiveRoom => {
            key.push_str(&format!("|{}", ctx.room_token.as_deref().unwrap_or("")))
        }
        ReadboardWindowKind::RecordView => key.push_str(&format!(
            "|{}|{}",
            ctx.record_total_move.map_or(-1, i64::from),
            ctx.title_fingerprint.as_deref().unwrap_or("")
        )),
        ReadboardWindowKind::Unknown => {}
    }
    key
}

/// `tryApplySingleMoveRecovery`: one legal move (with captures) from the sync start reaches the
/// frame; a recapture repeating the previous position (ko) or a suicide is refused.
fn single_move_recovery(
    work: &CurrentSgfDocument,
    start_path: &NodePath,
    main_line: &[NodeInfo],
    frame: &ReadboardFrame,
) -> Result<Option<(CurrentSgfDocument, NodePath)>, CurrentGameError> {
    let start = main_line.last().expect("root");
    let delta = Delta::summarize(&start.stones, &frame.codes, frame.width);
    if !delta.is_single_move_capture()
        || own_stone_removed(
            &start.stones,
            &frame.codes,
            delta.move_color.expect("move colour"),
        )
    {
        return Ok(None);
    }
    let color = delta.move_color.expect("move colour");
    let point = Point {
        x: delta.move_x,
        y: delta.move_y,
    };
    let Some(after) = play_on(&start.stones, frame.width, frame.height, color, point) else {
        return Ok(None);
    };
    if after != frame_stones(&frame.codes) || repeats_previous(main_line, &after) {
        return Ok(None);
    }
    let mut document = work.clone();
    let end = append_move(&mut document, start_path, color, point)?;
    Ok(Some((document, end)))
}

/// Java `isSingleMoveCapture` rejects frames where the mover lost stones.
fn own_stone_removed(start: &[Option<Color>], codes: &[u8], color: Color) -> bool {
    start
        .iter()
        .zip(codes)
        .any(|(&stone, &code)| stone == Some(color) && code_color(code).is_none())
}

/// Places `color` at `point` with captures; None when occupied or suicidal.
fn play_on(
    stones: &[Option<Color>],
    width: u8,
    height: u8,
    color: Color,
    point: Point,
) -> Option<Vec<Option<Color>>> {
    let mut board = Board::new(width, height).ok()?;
    for (index, stone) in stones.iter().enumerate() {
        if stone.is_some() {
            let at = Point {
                x: (index % width as usize) as u8,
                y: (index / width as usize) as u8,
            };
            board.set_stone(at, *stone).ok()?;
        }
    }
    board.play(color, Vertex::Point(point)).ok()?;
    Some(board.stones_snapshot())
}

/// Java recovery ko: the result may not equal the position before the sync start.
fn repeats_previous(line: &[NodeInfo], after: &[Option<Color>]) -> bool {
    line.len() >= 2 && line[line.len() - 2].stones == after
}

fn append_move(
    document: &mut CurrentSgfDocument,
    at: &NodePath,
    color: Color,
    point: Point,
) -> Result<NodePath, CurrentGameError> {
    let (width, height) = (document.board_width(), document.board_height());
    let key = if color == Color::Black { "B" } else { "W" };
    let value = serialize_vertex(
        &MoveVertex::Point(PointDto {
            x: point.x,
            y: point.y,
        }),
        width,
        height,
    )?;
    let node = document.node_mut(at)?;
    let index = match node.children.iter().position(|child| {
        child
            .properties
            .iter()
            .any(|property| property.key == key && property.values.first() == Some(&value))
    }) {
        Some(index) => index,
        None => {
            node.children.push(SgfNode {
                properties: vec![SgfProperty {
                    key: key.into(),
                    values: vec![value],
                }],
                children: Vec::new(),
            });
            node.children.len() - 1
        }
    };
    let mut path = at.clone();
    path.indices
        .push(u32::try_from(index).expect("child index fits u32"));
    Ok(path)
}

fn empty_document(width: u8, height: u8) -> Result<CurrentSgfDocument, CurrentGameError> {
    let root = SgfNode {
        properties: [("GM", "1"), ("FF", "4")]
            .into_iter()
            .map(|(key, value)| SgfProperty {
                key: key.into(),
                values: vec![value.into()],
            })
            .collect(),
        children: Vec::new(),
    };
    document_from_root(root, width, height, 0.0)
}

fn document_from_root(
    root: SgfNode,
    width: u8,
    height: u8,
    komi: f32,
) -> Result<CurrentSgfDocument, CurrentGameError> {
    let staged = CurrentSgfDocument {
        document: SgfDocument {
            board_width: width,
            board_height: height,
            komi,
            handicap: None,
            black_name: None,
            white_name: None,
            result: None,
            moves: Vec::new(),
            root: Some(root),
        },
    };
    // Normalise derived document fields (komi, names, handicap) through the regular parser.
    CurrentSgfDocument::open(&staged.serialize()?)
}

/// Java `BoardData` properties: analysis tags live in Java's analysis state, so a snapshot copy
/// never carries LZ/LZOP results of another position.
fn java_properties(node: &SgfNode) -> Vec<SgfProperty> {
    node.properties
        .iter()
        .filter(|property| {
            !matches!(
                property.key.as_str(),
                "B" | "W" | "C" | "LZ" | "LZ2" | "LZOP" | "LZOP2"
            )
        })
        .cloned()
        .collect()
}

fn comment(node: &SgfNode) -> Option<String> {
    node.properties
        .iter()
        .find(|property| property.key == "C")
        .and_then(|property| property.values.first().cloned())
}

fn pl_value(properties: &[SgfProperty]) -> Option<bool> {
    properties
        .iter()
        .find(|property| property.key == "PL")
        .and_then(|property| property.values.first())
        .filter(|value| !value.is_empty())
        .map(|value| !value.eq_ignore_ascii_case("W"))
}

fn coordinate(index: usize, width: u8) -> String {
    let (x, y) = ((index % width as usize) as u8, (index / width as usize) as u8);
    format!("{}{}", char::from(b'a' + x), char::from(b'a' + y))
}

fn on_main_trunk(path: &NodePath) -> bool {
    path.indices.iter().all(|&index| index == 0)
}

fn prefix(path: &NodePath, depth: usize) -> NodePath {
    NodePath {
        indices: path.indices[..depth].to_vec(),
    }
}

fn frame_stones(codes: &[u8]) -> Vec<Option<Color>> {
    codes.iter().map(|&code| code_color(code)).collect()
}

/// Side to play from the frame's single marker when its source is a trusted visual marker.
fn trusted_marker_turn(delta: &Delta, ctx: &ReadboardRemoteContext) -> Option<bool> {
    delta
        .frame_marker
        .filter(|_| ctx.last_move_source.is_trusted_visual_marker())
        .map(|marker| marker.color == Color::White)
}

/// The frame's last-move marker when exactly one is present.
fn unique_marker(codes: &[u8], width: u8) -> Option<Marker> {
    let mut markers = codes.iter().enumerate().filter_map(|(index, &code)| {
        marker_color(code).map(|color| Marker {
            x: (index % width as usize) as u8,
            y: (index / width as usize) as u8,
            color,
        })
    });
    let first = markers.next();
    if markers.next().is_some() {
        None
    } else {
        first
    }
}

fn code_color(code: u8) -> Option<Color> {
    match code {
        1 | 3 => Some(Color::Black),
        2 | 4 => Some(Color::White),
        _ => None,
    }
}

fn marker_color(code: u8) -> Option<Color> {
    match code {
        3 => Some(Color::Black),
        4 => Some(Color::White),
        _ => None,
    }
}

fn invalid(message: &str) -> CurrentGameError {
    CurrentGameError {
        kind: CurrentGameErrorKind::MalformedSgf,
        message: message.into(),
    }
}

#[cfg(test)]
#[path = "readboard_sync_tests.rs"]
mod readboard_sync_tests;
