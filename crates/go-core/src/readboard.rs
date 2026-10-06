//! Structured readboard frames (readboard `cdcc7b3`, wire 220430) and the frozen Java
//! `SyncRemoteContext` / `ReadBoardLastMoveSource` semantics (`7b40275`).
//!
//! The context is folded line by line in arrival order; each `with_*` mirrors the Java
//! transition, including the fields it clears. A frame owns the context that preceded its
//! `end` line; the decoder starts a fresh generic context after every `end`.

/// Snapshot code at `y * width + x`: 0 empty, 1 black, 2 white, 3 black last move, 4 white last move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadboardFrame {
    pub width: u8,
    pub height: u8,
    pub codes: Vec<u8>,
    pub context: ReadboardRemoteContext,
}

/// Host-visible control lines that change sync state without carrying a board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadboardControl {
    /// `start W H [hwnd]`; `size` is `None` when the line omits dimensions.
    Start {
        size: Option<(u8, u8)>,
    },
    Clear,
    ClearBoard,
    Sync,
    StopSync,
    EndSync,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadboardPlatform {
    Fox,
    Yike,
    #[default]
    Generic,
}

impl ReadboardPlatform {
    /// `syncPlatform <token>`: case-insensitive `fox` / `yike`, anything else generic.
    pub fn parse(token: &str) -> Self {
        let token = token.trim();
        if token.eq_ignore_ascii_case("fox") {
            Self::Fox
        } else if token.eq_ignore_ascii_case("yike") {
            Self::Yike
        } else {
            Self::Generic
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadboardWindowKind {
    LiveRoom,
    RecordView,
    #[default]
    Unknown,
}

/// `LegacyUnknown` means the frame carried no `lastMoveSource` line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReadboardLastMoveSource {
    #[default]
    LegacyUnknown,
    Unknown,
    None,
    RedBlueMarker,
    FoxCornerFlip,
    Deviation,
    StoneCount,
}

impl ReadboardLastMoveSource {
    /// Exact, case-sensitive tokens; anything else is `Unknown`.
    pub fn parse(token: &str) -> Self {
        match token {
            "none" => Self::None,
            "redBlueMarker" => Self::RedBlueMarker,
            "foxCornerFlip" => Self::FoxCornerFlip,
            "deviation" => Self::Deviation,
            "stoneCount" => Self::StoneCount,
            _ => Self::Unknown,
        }
    }

    pub fn is_trusted_visual_marker(self) -> bool {
        matches!(self, Self::RedBlueMarker | Self::FoxCornerFlip)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReadboardRemoteContext {
    pub platform: ReadboardPlatform,
    pub window_kind: ReadboardWindowKind,
    pub fox_move_number: Option<u32>,
    pub room_token: Option<String>,
    pub live_title_move: Option<u32>,
    pub record_current_move: Option<u32>,
    pub record_total_move: Option<u32>,
    pub record_at_end: bool,
    pub title_fingerprint: Option<String>,
    pub last_move_source: ReadboardLastMoveSource,
    pub force_rebuild: bool,
}

impl ReadboardRemoteContext {
    pub fn generic(force_rebuild: bool) -> Self {
        Self {
            force_rebuild,
            ..Self::default()
        }
    }

    pub fn with_platform(self, platform: ReadboardPlatform) -> Self {
        if platform == ReadboardPlatform::Generic {
            return Self::generic(self.force_rebuild);
        }
        Self {
            platform,
            window_kind: ReadboardWindowKind::Unknown,
            fox_move_number: self.fox_move_number,
            last_move_source: self.last_move_source,
            force_rebuild: self.force_rebuild,
            ..Self::default()
        }
    }

    pub fn with_fox_move_number(self, fox_move_number: Option<u32>) -> Self {
        Self {
            fox_move_number,
            ..self
        }
    }

    pub fn with_last_move_source(self, last_move_source: ReadboardLastMoveSource) -> Self {
        Self {
            last_move_source,
            ..self
        }
    }

    pub fn with_room_token(self, room_token: &str) -> Self {
        Self {
            window_kind: ReadboardWindowKind::LiveRoom,
            room_token: normalized(room_token),
            record_current_move: None,
            record_total_move: None,
            record_at_end: false,
            title_fingerprint: None,
            ..self
        }
    }

    pub fn with_live_title_move(self, live_title_move: Option<u32>) -> Self {
        Self {
            window_kind: ReadboardWindowKind::LiveRoom,
            live_title_move,
            record_current_move: None,
            record_total_move: None,
            record_at_end: false,
            title_fingerprint: None,
            ..self
        }
    }

    pub fn with_record_current_move(self, record_current_move: Option<u32>) -> Self {
        Self {
            window_kind: ReadboardWindowKind::RecordView,
            record_current_move,
            ..self.into_record_view()
        }
    }

    pub fn with_record_total_move(self, record_total_move: Option<u32>) -> Self {
        Self {
            window_kind: ReadboardWindowKind::RecordView,
            record_total_move,
            ..self.into_record_view()
        }
    }

    pub fn with_record_at_end(self, record_at_end: bool) -> Self {
        Self {
            window_kind: ReadboardWindowKind::RecordView,
            record_at_end,
            ..self.into_record_view()
        }
    }

    pub fn with_title_fingerprint(self, fingerprint: &str) -> Self {
        Self {
            window_kind: ReadboardWindowKind::RecordView,
            title_fingerprint: normalized(fingerprint),
            ..self.into_record_view()
        }
    }

    pub fn with_force_rebuild(self, force_rebuild: bool) -> Self {
        Self {
            force_rebuild,
            ..self
        }
    }

    pub fn without_force_rebuild(self) -> Self {
        self.with_force_rebuild(false)
    }

    pub fn supports_fox_recovery(&self) -> bool {
        self.platform == ReadboardPlatform::Fox && self.recovery_move_number().is_some()
    }

    /// Fox number, unless the window title reports a different move.
    pub fn recovery_move_number(&self) -> Option<u32> {
        if self.platform != ReadboardPlatform::Fox {
            return None;
        }
        let fox = self.fox_move_number?;
        match self.title_move_number() {
            Some(title) if title != fox => None,
            _ => Some(fox),
        }
    }

    /// Live rooms compare room tokens; record views compare fingerprint and total moves.
    pub fn conflicts_with(&self, other: &Self) -> bool {
        if self.platform != other.platform || self.window_kind != other.window_kind {
            return true;
        }
        match self.window_kind {
            ReadboardWindowKind::LiveRoom => self.room_token != other.room_token,
            ReadboardWindowKind::RecordView => {
                self.title_fingerprint != other.title_fingerprint
                    || self.record_total_move != other.record_total_move
            }
            ReadboardWindowKind::Unknown => false,
        }
    }

    fn title_move_number(&self) -> Option<u32> {
        match self.window_kind {
            ReadboardWindowKind::LiveRoom => self.live_title_move,
            ReadboardWindowKind::RecordView => {
                if self.record_current_move.is_none() && self.record_at_end {
                    self.record_total_move
                } else {
                    self.record_current_move
                }
            }
            ReadboardWindowKind::Unknown => None,
        }
    }

    fn into_record_view(self) -> Self {
        Self {
            room_token: None,
            live_title_move: None,
            ..self
        }
    }
}

fn normalized(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}
