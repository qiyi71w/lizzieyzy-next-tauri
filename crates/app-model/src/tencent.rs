use crate::{NetworkRouteDto, ProviderImportResult, ProviderRequestIdentityDto};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TencentQueryKindDto {
    Username,
    ChessId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TencentQueryDto {
    pub kind: TencentQueryKindDto,
    pub value: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TencentHistoryDto {
    pub recent: Vec<TencentQueryDto>,
    pub last_query: Option<TencentQueryDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TencentListEntryDto {
    pub chess_id: String,
    pub black_name: String,
    pub white_name: String,
    pub black_rank: String,
    pub white_rank: String,
    pub played_at: String,
    pub result: String,
    pub move_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TencentListPageDto {
    pub username: String,
    pub last_code: String,
    pub games: Vec<TencentListEntryDto>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TencentListResultDto {
    pub identity: ProviderRequestIdentityDto,
    pub result: TencentListPageDto,
    pub routes: Vec<NetworkRouteDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TencentPreviewResultDto {
    pub identity: ProviderRequestIdentityDto,
    pub result: ProviderImportResult,
    pub routes: Vec<NetworkRouteDto>,
}
