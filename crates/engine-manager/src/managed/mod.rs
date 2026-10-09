//! Explicit resource acquisition. This owner never edits a profile or starts a process.
mod catalog;
mod package;
pub(crate) mod trust;
pub use catalog::{catalog_snapshot, DEFAULT_MODEL_FILE};
use app_model::*;
use provider_core::network::{NetworkOperation, NetworkState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

struct Operation {
    request: ManagedAcquireRequestDto,
    view: ManagedOperationDto,
    network: NetworkOperation,
    preparing: bool,
}
pub struct ManagedResources {
    root: PathBuf,
    operation: Mutex<Option<Operation>>,
    artifact_availability: Mutex<BTreeMap<String, String>>,
}
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstalledManifest {
    schema_version: u32,
    source_commit: String,
    katago_source_commit: String,
    engine_repository: String,
    engine_tag: String,
    origin: String,
    target_id: String,
    model_id: String,
    archive_sha256: String,
    files: BTreeMap<String, String>,
}
pub struct PreparedAcquisition {
    operation_id: String,
    directory: PathBuf,
    manifest: InstalledManifest,
}
impl Drop for PreparedAcquisition {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.directory); }
}
impl ManagedResources {
    pub fn new(root: PathBuf) -> Self { Self { root, operation: Mutex::new(None), artifact_availability: Mutex::new(BTreeMap::new()) } }
    pub fn snapshot(&self) -> ManagedResourcesDto {
        let operation = self.operation.lock().expect("managed operation");
        let mut catalog = catalog_snapshot();
        let availability = self.artifact_availability.lock().expect("managed availability");
        for target in &mut catalog.targets {
            if let Some(observed) = availability.get(&target.id) { target.artifact_availability = observed.clone(); }
        }
        let view = operation.as_ref().map(|op| {
            let mut view = op.view.clone();
            view.routes = op.network.routes();
            view
        });
        ManagedResourcesDto { catalog, operation: view }
    }
    pub fn begin(&self, request: ManagedAcquireRequestDto, network: &NetworkState) -> Result<String, String> {
        let mut current = self.operation.lock().map_err(|_| "managed_state_unavailable")?;
        if current.as_ref().is_some_and(|op| !op.view.phase.terminal() || op.preparing) { return Err("managed_busy".into()); }
        let catalog = catalog::frozen();
        let asset = catalog.assets.get(&request.target_id).ok_or("managed_unknown_target")?;
        let model = catalog.models.get(&request.model_id).ok_or("managed_unknown_model")?;
        if catalog::host_target() != Some(request.target_id.as_str()) { return Err("managed_hardware_unqualified".into()); }
        if catalog.schema_version != 2 || catalog.katago_source_commit != catalog::KATAGO_SOURCE
            || catalog.origin != "project-source-build" || catalog.engine_release_repository != "wimi321/lizzieyzy-next"
            || asset.asset_name != format!("katago-source-47aadc08518b-{}.zip", request.target_id)
            || !package::safe_name(&asset.asset_name) || !package::safe_name(&model.file_name) {
            return Err("managed_catalog_untrusted".into());
        }
        let operation = network.begin_resource(request.policy_revision).map_err(|_| "managed_network_policy_stale")?;
        let id = Uuid::new_v4().to_string();
        *current = Some(Operation {
            view: ManagedOperationDto {
                operation_id: id.clone(), profile_id: request.profile_id.clone(), target_id: request.target_id.clone(), model_id: request.model_id.clone(),
                phase: ManagedPhaseDto::Starting, transferred_bytes: 0, total_bytes: asset.size_bytes + model.size_bytes,
                message: None, installation: None, routes: Vec::new(),
            }, request, network: operation, preparing: false,
        });
        Ok(id)
    }
    pub fn request(&self, id: &str) -> Result<ManagedAcquireRequestDto, String> {
        let state = self.operation.lock().map_err(|_| "managed_state_unavailable")?;
        Ok(state.as_ref().filter(|op| op.view.operation_id == id).ok_or("managed_stale")?.request.clone())
    }
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let mut state = self.operation.lock().map_err(|_| "managed_state_unavailable")?;
        let op = state.as_mut().filter(|op| op.view.operation_id == id).ok_or("managed_stale")?;
        if op.view.phase == ManagedPhaseDto::Publishing { return Err("managed_commit_in_progress".into()); }
        if !op.view.phase.terminal() {
            op.network.cancel();
            op.view.message = Some("managed_cancellation_requested".into());
        }
        Ok(())
    }
    pub fn fail(&self, id: &str, message: &str) {
        let mut state = self.operation.lock().expect("managed operation");
        if let Some(op) = state.as_mut().filter(|op| op.view.operation_id == id && !op.view.phase.terminal()) {
            op.view.phase = if op.network.lease().check().is_err() { ManagedPhaseDto::Cancelled } else { ManagedPhaseDto::Failed };
            // Domain callers pass bounded codes or transport's sanitized errors, never filesystem paths.
            op.view.message = Some(message.chars().take(256).collect());
            op.preparing = false;
        }
    }
    fn phase(&self, id: &str, phase: ManagedPhaseDto, bytes: u64) {
        let mut state = self.operation.lock().expect("managed operation");
        if let Some(op) = state.as_mut().filter(|op| op.view.operation_id == id && !op.view.phase.terminal()) {
            op.view.phase = phase;
            op.view.transferred_bytes = bytes;
        }
    }
    pub fn prepare(&self, id: &str) -> Result<PreparedAcquisition, String> {
        let result = self.prepare_inner(id);
        if let Err(error) = &result {
            if error != "managed_busy" { self.fail(id, error); }
        }
        result
    }
    fn prepare_inner(&self, id: &str) -> Result<PreparedAcquisition, String> {
        let (request, network) = {
            let mut state = self.operation.lock().map_err(|_| "managed_state_unavailable")?;
            let op = state.as_mut().filter(|op| op.view.operation_id == id && !op.view.phase.terminal()).ok_or("managed_stale")?;
            if op.preparing { return Err("managed_busy".into()); }
            op.preparing = true;
            (op.request.clone(), op.network.clone())
        };
        network.lease().check().map_err(|_| "managed_cancelled")?;
        fs::create_dir_all(&self.root).map_err(|_| "managed_storage_unavailable")?;
        let directory = self.root.join(format!("staging-{id}"));
        fs::create_dir(&directory).map_err(|_| "managed_staging_unavailable")?;
        let catalog = catalog::frozen();
        let asset = &catalog.assets[&request.target_id];
        let model = &catalog.models[&request.model_id];
        let mut prepared = PreparedAcquisition {
            operation_id: id.into(), directory,
            manifest: InstalledManifest {
                schema_version: 2, source_commit: catalog::SOURCE_COMMIT.into(), katago_source_commit: catalog::KATAGO_SOURCE.into(),
                engine_repository: catalog.engine_release_repository.clone(), engine_tag: catalog::ENGINE_TAG.into(), origin: catalog.origin.clone(),
                target_id: request.target_id.clone(), model_id: request.model_id.clone(), archive_sha256: asset.sha256.clone(), files: BTreeMap::new(),
            },
        };
        let archive = prepared.directory.join("download.zip");
        self.phase(id, ManagedPhaseDto::DownloadingEngine, 0);
        network.download(&catalog::asset_url(asset), &archive, asset.size_bytes, catalog::ORIGINS,
            |bytes| self.phase(id, ManagedPhaseDto::DownloadingEngine, bytes)).map_err(|error| {
                if error.kind == ProviderErrorKind::NotFound {
                    self.artifact_availability.lock().expect("managed availability").insert(request.target_id.clone(), "unavailable".into());
                }
                error.message
            })?;
        self.phase(id, ManagedPhaseDto::VerifyingEngine, asset.size_bytes);
        package::verify_file(&archive, asset.size_bytes, &asset.sha256, &network)?;
        let contents = prepared.directory.join("contents");
        fs::create_dir(&contents).map_err(|_| "managed_storage_unavailable")?;
        package::extract(&archive, &contents, &network)?;
        package::verify_package(&contents, &request.target_id, asset, &network)?;
        self.artifact_availability.lock().expect("managed availability").insert(request.target_id.clone(), "verified".into());
        fs::rename(&archive, contents.join("resource-archive.zip")).map_err(|_| "managed_storage_unavailable")?;
        self.phase(id, ManagedPhaseDto::DownloadingModel, asset.size_bytes);
        let model_path = contents.join(&model.file_name);
        network.download(&catalog::model_url(model), &model_path, model.size_bytes, catalog::ORIGINS,
            |bytes| self.phase(id, ManagedPhaseDto::DownloadingModel, asset.size_bytes + bytes)).map_err(|error| error.message)?;
        self.phase(id, ManagedPhaseDto::VerifyingModel, asset.size_bytes + model.size_bytes);
        package::verify_file(&model_path, model.size_bytes, &model.sha256, &network)?;
        let inspection = crate::models::inspect_model(&model_path);
        if inspection.status != ModelInspectionStatusDto::HeaderRecognized || inspection.sha256.as_deref() != Some(model.sha256.as_str()) {
            return Err("managed_model_invalid".into());
        }
        prepared.manifest.files = package::content_hashes(&contents, &network)?;
        let bytes = serde_json::to_vec(&prepared.manifest).map_err(|_| "managed_manifest_invalid")?;
        let mut manifest_file = fs::File::create(contents.join("installed-manifest.json")).map_err(|_| "managed_manifest_write_failed")?;
        std::io::Write::write_all(&mut manifest_file, &bytes).map_err(|_| "managed_manifest_write_failed")?;
        manifest_file.sync_all().map_err(|_| "managed_manifest_write_failed")?;
        Ok(prepared)
    }
    /// Caller holds the existing saved-profile transaction lock and has revalidated the captured
    /// profile and selected identity. The lease serializes publication with Cancel/policy/exit.
    pub fn publish(&self, prepared: PreparedAcquisition, inventory: &crate::models::ModelInventory) -> Result<ManagedInstallationDto, String> {
        let id = prepared.operation_id.clone();
        let result = self.publish_inner(&prepared, inventory);
        if let Err(error) = &result { self.fail(&id, error); }
        result
    }
    fn publish_inner(&self, prepared: &PreparedAcquisition, inventory: &crate::models::ModelInventory) -> Result<ManagedInstallationDto, String> {
        let network = {
            let state = self.operation.lock().map_err(|_| "managed_state_unavailable")?;
            state.as_ref().filter(|op| op.view.operation_id == prepared.operation_id && !op.view.phase.terminal()).ok_or("managed_stale")?.network.clone()
        };
        let contents = prepared.directory.join("contents");
        if package::content_hashes(&contents, &network)? != prepared.manifest.files { return Err("managed_content_changed".into()); }
        let actual: InstalledManifest = serde_json::from_slice(&fs::read(contents.join("installed-manifest.json")).map_err(|_| "managed_manifest_missing")?).map_err(|_| "managed_manifest_invalid")?;
        if actual != prepared.manifest { return Err("managed_manifest_changed".into()); }
        self.phase(&prepared.operation_id, ManagedPhaseDto::Publishing, catalog::frozen().assets[&prepared.manifest.target_id].size_bytes + catalog::frozen().models[&prepared.manifest.model_id].size_bytes);
        let destination = self.root.join("installed").join(&prepared.operation_id);
        let model = &catalog::frozen().models[&prepared.manifest.model_id];
        let executable = if prepared.manifest.target_id.starts_with("windows-") { "katago.exe" } else { "katago" };
        let manifest_bytes = serde_json::to_vec(&prepared.manifest).map_err(|_| "managed_manifest_invalid")?;
        let installation = ManagedInstallationDto {
            target_id: prepared.manifest.target_id.clone(), model_id: prepared.manifest.model_id.clone(),
            program: destination.join(executable).to_string_lossy().into_owned(),
            model_path: destination.join(&model.file_name).to_string_lossy().into_owned(),
            config_path: destination.join("analysis_example.cfg").to_string_lossy().into_owned(),
            manifest_sha256: format!("{:x}", Sha256::digest(&manifest_bytes)),
        };
        network.lease().with_valid(|| {
            fs::create_dir_all(destination.parent().expect("installed parent")).map_err(|_| "managed_storage_unavailable".to_string())?;
            fs::rename(&contents, &destination).map_err(|_| "managed_publish_failed".to_string())?;
            if let Err(error) = inventory.retain_managed(&destination.join(&model.file_name), &prepared.manifest.model_id, &model.sha256) {
                let _ = fs::remove_dir_all(&destination);
                return Err(error);
            }
            Ok(())
        }).map_err(|_| "managed_cancelled")??;
        let mut state = self.operation.lock().map_err(|_| "managed_state_unavailable")?;
        let op = state.as_mut().filter(|op| op.view.operation_id == prepared.operation_id).ok_or("managed_stale")?;
        op.view.phase = ManagedPhaseDto::Succeeded;
        op.view.installation = Some(installation.clone());
        op.view.message = Some("managed_installed_not_started".into());
        op.preparing = false;
        Ok(installation)
    }
}
