use super::{catalog, package, InstalledManifest};
use provider_core::network::NetworkState;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

pub(crate) struct ManagedIdentity {
    pub source_commit: String,
    pub static_zlib: bool,
    pub receipt_paths: Vec<PathBuf>,
}
/// A copied receipt is never authority: only the application-owned install root, pinned archive
/// bytes and every extracted file matching that archive can confer managed origin.
pub(crate) fn qualify(root: &Path, executable: &Path) -> Result<Option<ManagedIdentity>, String> {
    let installed = root.join("installed");
    if !installed.exists() { return Ok(None); }
    let installed = installed.canonicalize().map_err(|_| "managed_root_unreadable")?;
    let executable = executable.canonicalize().map_err(|_| "managed_executable_unreadable")?;
    if !executable.starts_with(&installed) { return Ok(None); }
    let directory = executable.parent().ok_or("managed_installation_invalid")?;
    if directory.parent() != Some(installed.as_path()) { return Err("managed_installation_invalid".into()); }
    let manifest_path = directory.join("installed-manifest.json");
    if fs::metadata(&manifest_path).map_err(|_| "managed_manifest_missing")?.len() > 1024 * 1024 { return Err("managed_manifest_limit".into()); }
    let manifest: InstalledManifest = serde_json::from_slice(&fs::read(&manifest_path).map_err(|_| "managed_manifest_missing")?).map_err(|_| "managed_manifest_invalid")?;
    let catalog = catalog::frozen();
    let asset = catalog.assets.get(&manifest.target_id).ok_or("managed_unknown_target")?;
    let model = catalog.models.get(&manifest.model_id).ok_or("managed_unknown_model")?;
    if manifest.schema_version != 2 || manifest.source_commit != catalog::SOURCE_COMMIT || manifest.katago_source_commit != catalog::KATAGO_SOURCE
        || manifest.engine_tag != catalog::ENGINE_TAG || manifest.engine_repository != catalog.engine_release_repository
        || manifest.origin != catalog.origin || manifest.archive_sha256 != asset.sha256 { return Err("managed_manifest_identity_mismatch".into()); }
    let network = NetworkState::default().begin_resource(0).map_err(|_| "managed_verification_failed")?;
    let archive_path = directory.join("resource-archive.zip");
    package::verify_file(&archive_path, asset.size_bytes, &asset.sha256, &network)?;
    let hashes = package::content_hashes(directory, &network)?;
    if hashes != manifest.files || hashes.get(&model.file_name) != Some(&model.sha256) { return Err("managed_content_changed".into()); }
    let exe_name = if manifest.target_id.starts_with("windows-") { "katago.exe" } else { "katago" };
    if executable.file_name().and_then(|name| name.to_str()) != Some(exe_name) || hashes.get(exe_name) != Some(&asset.executable_sha256) {
        return Err("managed_executable_changed".into());
    }
    let mut archive = zip::ZipArchive::new(File::open(&archive_path).map_err(|_| "managed_archive_unreadable")?).map_err(|_| "managed_archive_invalid")?;
    let mut file_count = 2; // archive and separately pinned model
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|_| "managed_archive_invalid")?;
        if entry.is_dir() { continue; }
        file_count += 1;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 65536];
        loop {
            let count = entry.read(&mut buffer).map_err(|_| "managed_archive_invalid")?;
            if count == 0 { break; }
            hasher.update(&buffer[..count]);
        }
        if hashes.get(entry.name()) != Some(&format!("{:x}", hasher.finalize())) { return Err("managed_archive_content_changed".into()); }
    }
    if manifest.target_id == super::trt::TARGET {
        let runtime = super::trt::runtime_hashes();
        if runtime.iter().any(|(name, digest)| hashes.get(name) != Some(digest)) { return Err("managed_runtime_content_changed".into()); }
        file_count += runtime.len();
    }
    if hashes.len() != file_count { return Err("managed_unlisted_content".into()); }
    package::verify_package(directory, &manifest.target_id, asset, &network)?;
    Ok(Some(ManagedIdentity {
        source_commit: catalog::KATAGO_SOURCE.into(), static_zlib: asset.zlib_linkage.as_deref() == Some("static"),
        receipt_paths: vec![manifest_path, archive_path, directory.join("source-release.json"), directory.join("source-package.json")],
    }))
}
