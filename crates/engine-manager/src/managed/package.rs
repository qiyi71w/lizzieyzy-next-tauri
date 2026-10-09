use super::catalog::{Asset, KATAGO_SOURCE};
use provider_core::network::NetworkOperation;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

pub(super) fn safe_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 240 && name.is_ascii()
        && !name.contains(['/', '\\', ':', '\0']) && !name.chars().any(char::is_control)
        && name != "." && name != ".." && !name.ends_with(['.', ' '])
        && !matches!(name.split('.').next().unwrap_or("").to_ascii_uppercase().as_str(), "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9" | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9")
}
fn safe_path(name: &str) -> bool {
    name.len() <= 1024 && name.split('/').all(safe_name)
}
pub(super) fn hash_file(path: &Path, network: &NetworkOperation) -> Result<(u64, String), String> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "managed_file_unreadable")?;
    if !metadata.is_file() || metadata.len() > 2 * 1024 * 1024 * 1024 { return Err("managed_file_invalid".into()); }
    let mut file = File::open(path).map_err(|_| "managed_file_unreadable")?;
    let deadline = Instant::now() + Duration::from_secs(60);
    let mut buffer = [0; 65536];
    let mut hasher = Sha256::new();
    let mut total = 0;
    loop {
        network.lease().check().map_err(|_| "managed_cancelled")?;
        if Instant::now() >= deadline { return Err("managed_verification_timeout".into()); }
        let count = file.read(&mut buffer).map_err(|_| "managed_file_unreadable")?;
        if count == 0 { break; }
        total += count as u64;
        if total > metadata.len() { return Err("managed_file_changed".into()); }
        hasher.update(&buffer[..count]);
    }
    if total != metadata.len() { return Err("managed_file_changed".into()); }
    Ok((total, format!("{:x}", hasher.finalize())))
}
pub(super) fn verify_file(path: &Path, size: u64, digest: &str, network: &NetworkOperation) -> Result<(), String> {
    let actual = hash_file(path, network)?;
    if actual.0 != size || actual.1 != digest { return Err("managed_digest_mismatch".into()); }
    Ok(())
}
pub(super) fn extract(archive: &Path, destination: &Path, network: &NetworkOperation) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(File::open(archive).map_err(|_| "managed_archive_unreadable")?).map_err(|_| "managed_archive_invalid")?;
    if archive.len() > 4096 { return Err("managed_archive_limit".into()); }
    let mut names = BTreeSet::new();
    let mut total: u64 = 0;
    for index in 0..archive.len() {
        network.lease().check().map_err(|_| "managed_cancelled")?;
        let mut entry = archive.by_index(index).map_err(|_| "managed_archive_invalid")?;
        let name = entry.name().trim_end_matches('/').to_owned();
        if !safe_path(&name) || !names.insert(name.to_ascii_lowercase()) { return Err("managed_archive_unsafe_path".into()); }
        let kind = entry.unix_mode().unwrap_or(0) & 0o170000;
        if !matches!(kind, 0 | 0o100000 | 0o040000) { return Err("managed_archive_unsafe_type".into()); }
        total = total.checked_add(entry.size()).ok_or("managed_archive_limit")?;
        if total > 2 * 1024 * 1024 * 1024 || entry.size() > 1024 * 1024 * 1024 { return Err("managed_archive_limit".into()); }
        let path = destination.join(&name);
        if entry.is_dir() {
            fs::create_dir_all(&path).map_err(|_| "managed_extraction_failed")?;
            continue;
        }
        fs::create_dir_all(path.parent().ok_or("managed_archive_unsafe_path")?).map_err(|_| "managed_extraction_failed")?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(&path).map_err(|_| "managed_extraction_failed")?;
        let mut buffer = [0; 65536];
        let mut bytes = 0;
        loop {
            network.lease().check().map_err(|_| "managed_cancelled")?;
            let count = entry.read(&mut buffer).map_err(|_| "managed_archive_invalid")?;
            if count == 0 { break; }
            bytes += count as u64;
            if bytes > entry.size() { return Err("managed_archive_limit".into()); }
            file.write_all(&buffer[..count]).map_err(|_| "managed_extraction_failed")?;
        }
        if bytes != entry.size() { return Err("managed_archive_invalid".into()); }
        file.sync_all().map_err(|_| "managed_extraction_failed")?;
        #[cfg(unix)]
        if name == "katago" {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).map_err(|_| "managed_extraction_failed")?;
        }
    }
    Ok(())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema_version: u32,
    origin: String,
    source_repository: String,
    source_commit: String,
    target: String,
    backend: String,
    executable: ManifestFile,
    files: Vec<ManifestFile>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestFile { file: String, size_bytes: u64, sha256: String }
pub(super) fn verify_package(directory: &Path, target: &str, asset: &Asset, network: &NetworkOperation) -> Result<(), String> {
    for name in ["source-package.json", "source-release.json"] {
        let path = directory.join(name);
        if fs::metadata(&path).map_err(|_| "managed_manifest_missing")?.len() > 1024 * 1024 { return Err("managed_manifest_limit".into()); }
        let manifest: Manifest = serde_json::from_slice(&fs::read(path).map_err(|_| "managed_manifest_unreadable")?).map_err(|_| "managed_manifest_invalid")?;
        if manifest.schema_version != 1 || manifest.origin != "project-source-build" || manifest.source_repository != "https://github.com/lightvector/KataGo"
            || manifest.source_commit != KATAGO_SOURCE || manifest.target != target || !manifest.backend.eq_ignore_ascii_case(&asset.backend)
            || manifest.executable.sha256 != asset.executable_sha256 || !safe_name(&manifest.executable.file)
            || manifest.executable.file != if target.starts_with("windows-") { "katago.exe" } else { "katago" } {
            return Err("managed_manifest_identity_mismatch".into());
        }
        verify_file(&directory.join(&manifest.executable.file), manifest.executable.size_bytes, &asset.executable_sha256, network)?;
        if manifest.files.len() > 4096 { return Err("managed_manifest_limit".into()); }
        let mut names = BTreeSet::new();
        for file in &manifest.files {
            // Frozen Windows build receipts use native separators; ZIP entries remain strictly portable.
            let path = if target.starts_with("windows-") && file.file.contains('\\') {
                std::borrow::Cow::Owned(file.file.replace('\\', "/"))
            } else { std::borrow::Cow::Borrowed(file.file.as_str()) };
            if !safe_path(&path) || !names.insert(path.to_ascii_lowercase()) { return Err("managed_manifest_unsafe_path".into()); }
            verify_file(&directory.join(path.as_ref()), file.size_bytes, &file.sha256, network)?;
        }
        if name == "source-release.json" && (!names.contains("analysis_example.cfg") || !names.contains("default_gtp.cfg")) {
            return Err("managed_config_identity_missing".into());
        }
    }
    Ok(())
}
pub(super) fn content_hashes(directory: &Path, network: &NetworkOperation) -> Result<BTreeMap<String, String>, String> {
    let mut result = BTreeMap::new();
    let mut pending = vec![directory.to_path_buf()];
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).map_err(|_| "managed_file_unreadable")? {
            let entry = entry.map_err(|_| "managed_file_unreadable")?;
            let path = entry.path();
            let name = path.strip_prefix(directory).map_err(|_| "managed_path_invalid")?.to_string_lossy().replace('\\', "/");
            if !safe_path(&name) || entry.file_type().map_err(|_| "managed_file_unreadable")?.is_symlink() { return Err("managed_path_invalid".into()); }
            if entry.file_type().map_err(|_| "managed_file_unreadable")?.is_dir() {
                pending.push(path);
            } else if name != "installed-manifest.json" {
                if result.len() >= 4096 { return Err("managed_manifest_limit".into()); }
                result.insert(name, hash_file(&path, network)?.1);
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use provider_core::network::NetworkState;

    #[test]
    fn unsafe_archive_paths_and_case_collisions_cannot_escape_staging() {
        for names in [vec!["../escape"], vec!["C:/escape"], vec!["a\\evil"], vec!["CON"], vec!["a/../escape"], vec!["KataGo", "katago"]] {
            let root = std::env::temp_dir().join(format!("archive-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(root.join("stage")).unwrap();
            let path = root.join("archive.zip");
            let mut zip = zip::ZipWriter::new(File::create(&path).unwrap());
            for name in names {
                zip.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
                zip.write_all(b"untrusted").unwrap();
            }
            zip.finish().unwrap();
            let network = NetworkState::default().begin_resource(0).unwrap();
            assert!(extract(&path, &root.join("stage"), &network).is_err());
            assert!(!root.join("escape").exists());
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    fn same_target_forged_source_manifest_and_wrong_archive_bytes_are_rejected() {
        let root = std::env::temp_dir().join(format!("manifest-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).unwrap();
        let network = NetworkState::default().begin_resource(0).unwrap();
        let asset = &crate::managed::catalog::frozen().assets["linux-cpu"];
        fs::write(root.join("archive.zip"), b"forged").unwrap();
        assert!(verify_file(&root.join("archive.zip"), 6, &asset.sha256, &network).is_err());
        fs::write(root.join("source-package.json"), serde_json::to_vec(&serde_json::json!({
            "schemaVersion": 1, "origin": "official", "sourceRepository": "https://github.com/attacker/KataGo",
            "sourceCommit": KATAGO_SOURCE, "target": "linux-cpu", "backend": "EIGEN",
            "executable": {"file": "katago", "sizeBytes": 6, "sha256": asset.executable_sha256}, "files": []
        })).unwrap()).unwrap();
        assert_eq!(verify_package(&root, "linux-cpu", asset, &network).unwrap_err(), "managed_manifest_identity_mismatch");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "requires read-only frozen CPU archives in LIZZIEYZY_MANAGED_ARCHIVES; package evidence, not download or Windows runtime evidence"]
    fn real_frozen_cpu_packages_validate_native_manifest_paths() {
        let inputs = std::path::PathBuf::from(std::env::var("LIZZIEYZY_MANAGED_ARCHIVES").unwrap());
        for target in ["linux-cpu", "windows-cpu"] {
            let root = std::env::temp_dir().join(format!("frozen-package-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(&root).unwrap();
            let network = NetworkState::default().begin_resource(0).unwrap();
            let asset = &crate::managed::catalog::frozen().assets[target];
            let archive = inputs.join(&asset.asset_name);
            verify_file(&archive, asset.size_bytes, &asset.sha256, &network).unwrap();
            extract(&archive, &root, &network).unwrap();
            let result = verify_package(&root, target, asset, &network);
            fs::remove_dir_all(root).unwrap();
            assert_eq!(result, Ok(()), "{target}");
        }
    }
}
