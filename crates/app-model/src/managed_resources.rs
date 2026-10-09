use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedTargetDto {
    pub id: String,
    pub platform: String,
    pub backend: String,
    pub archive: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub executable_sha256: String,
    pub source_availability: String,
    pub artifact_availability: String,
    pub hardware_qualification: String,
    pub runtime_acceptance: String,
    pub acquisition_allowed: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedModelDto {
    pub id: String,
    pub file_name: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub minimum_katago_version: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedCatalogDto {
    pub schema_version: u32,
    pub source_commit: String,
    pub katago_source_commit: String,
    pub katago_version: String,
    pub engine_repository: String,
    pub engine_tag: String,
    pub model_tag: String,
    pub default_model_id: String,
    pub targets: Vec<ManagedTargetDto>,
    pub models: Vec<ManagedModelDto>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedAcquireRequestDto {
    pub profile_id: String,
    pub profile: crate::EngineProfileDto,
    pub target_id: String,
    pub model_id: String,
    pub policy_revision: u64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManagedPhaseDto {
    Starting,
    DownloadingEngine,
    VerifyingEngine,
    DownloadingModel,
    VerifyingModel,
    Publishing,
    Succeeded,
    Failed,
    Cancelled,
}
impl ManagedPhaseDto {
    pub fn terminal(self) -> bool { matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled) }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedInstallationDto {
    pub target_id: String,
    pub model_id: String,
    pub program: String,
    pub model_path: String,
    pub config_path: String,
    pub manifest_sha256: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedOperationDto {
    pub operation_id: String,
    pub profile_id: String,
    pub target_id: String,
    pub model_id: String,
    pub phase: ManagedPhaseDto,
    pub transferred_bytes: u64,
    pub total_bytes: u64,
    pub message: Option<String>,
    pub installation: Option<ManagedInstallationDto>,
    pub routes: Vec<crate::NetworkRouteDto>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagedResourcesDto {
    pub catalog: ManagedCatalogDto,
    pub operation: Option<ManagedOperationDto>,
}
