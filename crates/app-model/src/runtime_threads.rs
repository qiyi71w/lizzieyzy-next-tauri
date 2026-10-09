use serde::{Deserialize, Serialize};

/// Ordered, immutable launch evidence. Paths are capture-local diagnostic aliases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadSourceEntryDto {
    pub key: String,
    pub value: Option<u32>,
    pub source: String,
    pub layer: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThreadSourcesDto {
    pub entries: Vec<ThreadSourceEntryDto>,
    pub saved: Option<u32>,
    pub launch_override: Option<u32>,
    pub effective: Option<u32>,
    pub analysis_threads: Option<u32>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeControlIdentityDto {
    pub run_id: String,
    pub profile_revision: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeThreadsActionDto { Read, Apply, Reset }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeThreadsRequestDto {
    pub identity: RuntimeControlIdentityDto,
    pub action: RuntimeThreadsActionDto,
    pub value: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeThreadsStatusDto { Unknown, Pending, Confirmed, Failed }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeThreadsSnapshotDto {
    pub run_id: Option<String>,
    pub profile_revision: Option<String>,
    pub supported: bool,
    pub reason: Option<String>,
    pub minimum: u32,
    pub maximum: u32,
    pub sources: Option<ThreadSourcesDto>,
    pub request_id: Option<String>,
    pub requested: Option<u32>,
    pub actual: Option<u32>,
    pub temporary: bool,
    pub status: RuntimeThreadsStatusDto,
    pub failure: Option<String>,
}
