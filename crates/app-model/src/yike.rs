use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YikeCategoryDto {
    #[default]
    Recommend,
    Local,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YikeListEntryDto {
    pub locator: String,
    pub title: String,
    pub black_name: String,
    pub white_name: String,
    pub status: String,
    pub move_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YikeListPageDto {
    pub category: YikeCategoryDto,
    pub page: u32,
    pub since: u64,
    pub games: Vec<YikeListEntryDto>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YikeListResultDto {
    pub identity: crate::ProviderRequestIdentityDto,
    pub result: YikeListPageDto,
    pub routes: Vec<crate::NetworkRouteDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YikePreviewResultDto {
    pub identity: crate::ProviderRequestIdentityDto,
    pub result: crate::ProviderImportResult,
    pub routes: Vec<crate::NetworkRouteDto>,
}
