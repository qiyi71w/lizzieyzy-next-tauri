use serde::{Deserialize, Serialize};

/// The three user lookups. Nickname and UID return an account game list; only chessid names a game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FoxLookupKindDto {
    Nickname,
    Uid,
    Chessid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoxLookupDto {
    pub kind: FoxLookupKindDto,
    pub value: String,
}

/// A resolved, non-secret Fox account. `nickname` may be empty when only the UID is known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoxAccountDto {
    pub uid: String,
    pub nickname: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoxGameEntryDto {
    pub chessid: String,
    pub start_time: String,
    pub title: String,
    pub black_name: String,
    pub black_uid: String,
    pub black_rank: String,
    pub white_name: String,
    pub white_uid: String,
    pub white_rank: String,
    pub result: String,
    pub move_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board_size: Option<u8>,
    /// Whether the queried account won; `None` when the winner is neither side of that account.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_won: Option<bool>,
}

/// One upstream batch. `next_cursor` is the chessid of the last kept game; continuation is UID-only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoxListPageDto {
    pub account: FoxAccountDto,
    pub games: Vec<FoxGameEntryDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoxListResultDto {
    pub identity: crate::ProviderRequestIdentityDto,
    pub result: FoxListPageDto,
    pub routes: Vec<crate::NetworkRouteDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FoxPreviewResultDto {
    pub identity: crate::ProviderRequestIdentityDto,
    pub result: crate::ProviderImportResult,
    pub routes: Vec<crate::NetworkRouteDto>,
}

/// Fox-owned PREF-01 state: at most eight recent accounts and the last lookup. No responses or secrets.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoxKifuStateDto {
    #[serde(default)]
    pub recents: Vec<FoxAccountDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_query: Option<FoxLookupDto>,
}
