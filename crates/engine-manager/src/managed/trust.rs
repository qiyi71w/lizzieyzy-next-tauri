use super::{catalog, package, InstalledManifest};
use provider_core::network::NetworkState;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

// The frozen TRT payload is 4.585 GB, authenticated by two complete content/package passes.
// Six <=2 GiB verification chunks at the existing 60-second/file allowance bound this stage.
const AUTHENTICATION_TIMEOUT: Duration = Duration::from_secs(360);

fn bounded_verification<T>(
    is_retired: &(dyn Fn() -> bool + Sync),
    timeout: Duration,
    verify: impl FnOnce(&provider_core::network::NetworkOperation) -> Result<T, String>,
) -> Result<T, String> {
    if is_retired() { return Err("managed_verification_retired".into()); }
    let network = NetworkState::default().begin_resource(0).map_err(|_| "managed_verification_failed")?;
    let deadline = Instant::now() + timeout;
    let done = AtomicBool::new(false);
    let result = std::thread::scope(|scope| {
        scope.spawn(|| {
            while !done.load(Ordering::Acquire) {
                if is_retired() || Instant::now() >= deadline { network.cancel(); break; }
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        let result = verify(&network);
        done.store(true, Ordering::Release);
        result
    });
    if result.as_ref().is_err_and(|error| error != "managed_cancelled") { return result; }
    if is_retired() { return Err("managed_verification_retired".into()); }
    if Instant::now() >= deadline { return Err("managed_verification_timeout".into()); }
    result
}

pub(crate) struct ManagedIdentity {
    pub source_commit: String,
    pub static_zlib: bool,
    pub receipt_paths: Vec<PathBuf>,
}
/// A copied receipt is never authority: only the application-owned install root, pinned archive
/// bytes and every extracted file matching that archive can confer managed origin.
pub(crate) fn qualify(root: &Path, executable: &Path, is_retired: &(dyn Fn() -> bool + Sync)) -> Result<Option<ManagedIdentity>, String> {
    let installed = root.join("installed");
    if !installed.exists() { return Ok(None); }
    let installed = installed.canonicalize().map_err(|_| "managed_root_unreadable")?;
    let executable = executable.canonicalize().map_err(|_| "managed_executable_unreadable")?;
    if !executable.starts_with(&installed) { return Ok(None); }
    let directory = executable.parent().ok_or("managed_installation_invalid")?;
    if directory.parent() != Some(installed.as_path()) { return Err("managed_installation_invalid".into()); }
    bounded_verification(is_retired, AUTHENTICATION_TIMEOUT, |network| qualify_inner(directory, &executable, network))
}

fn qualify_inner(directory: &Path, executable: &Path, network: &provider_core::network::NetworkOperation) -> Result<Option<ManagedIdentity>, String> {
    let manifest_path = directory.join("installed-manifest.json");
    if fs::metadata(&manifest_path).map_err(|_| "managed_manifest_missing")?.len() > 1024 * 1024 { return Err("managed_manifest_limit".into()); }
    let manifest: InstalledManifest = serde_json::from_slice(&fs::read(&manifest_path).map_err(|_| "managed_manifest_missing")?).map_err(|_| "managed_manifest_invalid")?;
    let catalog = catalog::frozen();
    let asset = catalog.assets.get(&manifest.target_id).ok_or("managed_unknown_target")?;
    let model = catalog.models.get(&manifest.model_id).ok_or("managed_unknown_model")?;
    if manifest.schema_version != 2 || manifest.source_commit != catalog::SOURCE_COMMIT || manifest.katago_source_commit != catalog::KATAGO_SOURCE
        || manifest.engine_tag != catalog::ENGINE_TAG || manifest.engine_repository != catalog.engine_release_repository
        || manifest.origin != catalog.origin || manifest.archive_sha256 != asset.sha256 { return Err("managed_manifest_identity_mismatch".into()); }
    let archive_path = directory.join("resource-archive.zip");
    package::verify_file(&archive_path, asset.size_bytes, &asset.sha256, network)?;
    let hashes = package::content_hashes(directory, network)?;
    if hashes != manifest.files || hashes.get(&model.file_name) != Some(&model.sha256) { return Err("managed_content_changed".into()); }
    let exe_name = if manifest.target_id.starts_with("windows-") { "katago.exe" } else { "katago" };
    if executable.file_name().and_then(|name| name.to_str()) != Some(exe_name) || hashes.get(exe_name) != Some(&asset.executable_sha256) {
        return Err("managed_executable_changed".into());
    }
    let mut archive = zip::ZipArchive::new(File::open(&archive_path).map_err(|_| "managed_archive_unreadable")?).map_err(|_| "managed_archive_invalid")?;
    let mut file_count = 2; // archive and separately pinned model
    for index in 0..archive.len() {
        network.lease().check().map_err(|_| "managed_cancelled")?;
        let mut entry = archive.by_index(index).map_err(|_| "managed_archive_invalid")?;
        if entry.is_dir() { continue; }
        file_count += 1;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 65536];
        loop {
            network.lease().check().map_err(|_| "managed_cancelled")?;
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
    package::verify_package(directory, &manifest.target_id, asset, network)?;
    Ok(Some(ManagedIdentity {
        source_commit: catalog::KATAGO_SOURCE.into(), static_zlib: asset.zlib_linkage.as_deref() == Some("static"),
        receipt_paths: vec![manifest_path, archive_path, directory.join("source-release.json"), directory.join("source-package.json")],
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_stage_expiry_cancels_the_verification_lease() {
        let error = bounded_verification(&|| false, Duration::from_millis(1), |network| {
            while network.lease().check().is_ok() { std::thread::sleep(Duration::from_millis(1)); }
            Err::<(), _>("managed_cancelled".into())
        }).unwrap_err();
        assert_eq!(error, "managed_verification_timeout");
    }

    #[test]
    fn operation_retirement_cancels_an_active_verification_lease() {
        let retired = AtomicBool::new(false);
        let error = bounded_verification(&|| retired.load(Ordering::Acquire), Duration::from_secs(1), |network| {
            retired.store(true, Ordering::Release);
            while network.lease().check().is_ok() { std::thread::sleep(Duration::from_millis(1)); }
            Err::<(), _>("managed_cancelled".into())
        }).unwrap_err();
        assert_eq!(error, "managed_verification_retired");
    }

    #[test]
    fn original_verification_failure_is_preserved() {
        assert_eq!(bounded_verification(&|| false, Duration::from_secs(1), |_| Err::<(), _>("managed_digest_mismatch".into())).unwrap_err(), "managed_digest_mismatch");
    }
}
