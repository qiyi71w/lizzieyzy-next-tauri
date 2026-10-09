use app_model::{ManagedCatalogDto, ManagedModelDto, ManagedTargetDto};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::sync::LazyLock;

pub const SOURCE_COMMIT: &str = "af0e07a7386483f3bfc8a15780de72ffc2f0de4c";
pub const KATAGO_SOURCE: &str = "47aadc08518b3e121f22539796c911002f699584";
pub const ENGINE_TAG: &str = "next-2026-10-08.1";
pub const DEFAULT_MODEL: &str = "b11-flagship";
pub const DEFAULT_MODEL_FILE: &str = "kata1-tf3-b11c768-s12002M-d6304M.bin.gz";
pub const ORIGINS: &[&str] = &["https://github.com", "https://release-assets.githubusercontent.com", "https://objects.githubusercontent.com", "https://media.katagotraining.org"];

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Catalog {
    pub schema_version: u32,
    pub katago_version: String,
    pub katago_source_commit: String,
    pub model_release_tag: String,
    pub engine_release_repository: String,
    pub origin: String,
    pub assets: BTreeMap<String, Asset>,
    pub models: BTreeMap<String, Model>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Asset {
    pub platform: String,
    pub backend: String,
    pub asset_name: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub executable_sha256: String,
    pub zlib_linkage: Option<String>,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Model {
    pub file_name: String,
    pub download_url: Option<String>,
    pub size_bytes: u64,
    pub sha256: String,
    pub minimum_kata_go_version: String,
}
static CATALOG: LazyLock<Catalog> = LazyLock::new(|| serde_json::from_str(include_str!("catalog.json")).expect("embedded frozen catalog"));
pub(super) fn frozen() -> &'static Catalog { &CATALOG }
pub(super) fn host_target() -> Option<&'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) { Some("windows-cpu") }
    else if cfg!(all(target_os = "linux", target_arch = "x86_64")) { Some("linux-cpu") }
    else { None }
}
pub(super) fn asset_url(asset: &Asset) -> String {
    format!("https://github.com/{}/releases/download/{ENGINE_TAG}/{}", frozen().engine_release_repository, asset.asset_name)
}
pub(super) fn model_url(model: &Model) -> String {
    model.download_url.clone().unwrap_or_else(|| format!("https://github.com/lightvector/KataGo/releases/download/{}/{}", frozen().model_release_tag, model.file_name))
}
pub fn catalog_snapshot() -> ManagedCatalogDto {
    let catalog = frozen();
    ManagedCatalogDto {
        schema_version: catalog.schema_version, source_commit: SOURCE_COMMIT.into(),
        katago_source_commit: catalog.katago_source_commit.clone(), katago_version: catalog.katago_version.clone(),
        engine_repository: catalog.engine_release_repository.clone(), engine_tag: ENGINE_TAG.into(),
        model_tag: catalog.model_release_tag.clone(), default_model_id: DEFAULT_MODEL.into(),
        targets: catalog.assets.iter().map(|(id, asset)| ManagedTargetDto {
            id: id.clone(), platform: asset.platform.clone(), backend: asset.backend.clone(),
            archive: asset.asset_name.clone(), size_bytes: asset.size_bytes, sha256: asset.sha256.clone(),
            executable_sha256: asset.executable_sha256.clone(), source_availability: "frozen".into(),
            artifact_availability: "not_checked".into(),
            hardware_qualification: if host_target() == Some(id.as_str()) { "host_cpu" } else { "unknown" }.into(),
            runtime_acceptance: "requires_explicit_start".into(), acquisition_allowed: host_target() == Some(id.as_str()),
        }).collect(),
        models: catalog.models.iter().map(|(id, model)| ManagedModelDto {
            id: id.clone(), file_name: model.file_name.clone(), sha256: model.sha256.clone(),
            size_bytes: model.size_bytes, minimum_katago_version: model.minimum_kata_go_version.clone(),
        }).collect(),
    }
}
