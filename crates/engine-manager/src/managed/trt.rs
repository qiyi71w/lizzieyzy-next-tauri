//! Version-pinned Windows TensorRT maintenance, not a foreground engine or a driver installer.
use super::{catalog, package};
use app_model::ManagedHardwareDto;
use provider_core::network::NetworkOperation;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

pub(super) const TARGET: &str = "windows-tensorrt";
pub(super) const VERSION: &str = "TensorRT 10.9.0.34 / CUDA 12.8 / cuDNN 9.8.0.87";
const ORIGINS: &[&str] = &["https://developer.download.nvidia.com", "https://github.com", "https://release-assets.githubusercontent.com", "https://objects.githubusercontent.com"];
// These are the frozen endpoint's dependency locks, excluding SDK/compiler-only archives.
#[derive(Deserialize)]
struct RuntimePackage { name: String, version: String, url: String, size: u64, sha256: String }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeFile { file: String, size_bytes: u64, sha256: String }
static PACKAGES: LazyLock<Vec<RuntimePackage>> = LazyLock::new(|| serde_json::from_str(include_str!("trt-packages.json")).expect("frozen TRT packages"));
// Exact DLL identities from the hash-authenticated frozen source-package.json, not a PE scan.
static FILES: LazyLock<Vec<RuntimeFile>> = LazyLock::new(|| serde_json::from_str(include_str!("trt-files.json")).expect("frozen TRT files"));
pub(super) fn download_bytes() -> u64 { PACKAGES.iter().map(|p| p.size).sum() }
pub(super) fn disk_bytes() -> u64 {
    // Sequential archive retirement: all final DLLs plus the largest in-flight archive.
    FILES.iter().map(|f| f.size_bytes).sum::<u64>() + PACKAGES.iter().map(|p| p.size).max().unwrap_or(0)
}
pub(super) fn runtime_hashes() -> BTreeMap<String, String> {
    FILES.iter().map(|f| (f.file.clone(), f.sha256.clone())).collect()
}

fn cache_path_units(cache: &Path, model_name: &str) -> usize {
    // Frozen trtbackend.cpp:1487 builds this ONNX plan key; :55 adds a uint64 random
    // hex suffix and a counter. This single-thread, one-model probe writes at most two caches.
    // Count the terminating NUL too: this frozen Windows build uses legacy file streams.
    let file = format!("trtcache/trt-100900_gpu-ffffffff_net-{model_name}_s9_onnxnh_mx19x19_b1_fp32.tmp_ffffffffffffffff_1");
    cache.join(file).as_os_str().to_string_lossy().encode_utf16().count() + 1
}

pub(super) fn validate_cache_capacity(cache: &Path, model_name: &str) -> Result<(), String> {
    if cfg!(windows) && cache_path_units(cache, model_name) > 260 {
        return Err("managed_cache_path_too_long_for_frozen_runtime".into());
    }
    Ok(())
}

fn qualification_error(error: crate::EngineManagerError) -> String {
    use crate::EngineManagerError;
    let (kind, exit_code, stderr) = match error {
        EngineManagerError::NonZeroExit { exit_code, stderr, .. } => ("process_failed", exit_code, stderr),
        EngineManagerError::MissingStdout { exit_code, stderr } => ("no_response", exit_code, stderr),
        EngineManagerError::InsufficientStdout { exit_code, stderr, .. } => ("incomplete_response", exit_code, stderr),
        EngineManagerError::Timeout { exit_code, stderr, .. } => ("timeout", exit_code, stderr),
        EngineManagerError::Cancelled { exit_code, stderr, .. } => ("cancelled", exit_code, stderr),
        other => return format!("managed_runtime_probe_failed\n{other}"),
    };
    let start = stderr.char_indices().rev().nth(2047).map_or(0, |(index, _)| index);
    // The existing operation boundary remains the sole privacy sanitizer. Preserve the actual
    // terminal reason, not only the startup config preamble; explicitly mark omitted output.
    format!("managed_runtime_{kind} exit_code={exit_code:?}\n{}{}", if start > 0 { "[earlier stderr omitted]\n" } else { "" }, &stderr[start..])
}

fn cache_defaults(cache: &Path) -> Result<String, String> {
    let path = cache.to_str().ok_or("managed_cache_path_invalid")?;
    if path.len() > 4096 || path.chars().any(char::is_control) { return Err("managed_cache_path_invalid".into()); }
    // Config-file quoting supports commas, spaces and Unicode, unlike override-config's CSV.
    Ok(format!("homeDataDir = \"{}\"\n", path.replace('\\', "\\\\").replace('"', "\\\"")))
}
pub(super) fn write_cache_defaults(staging: &Path, cache: &Path, operation: &str, hardware: &ManagedHardwareDto) -> Result<(), String> {
    let defaults = cache_defaults(cache)?;
    let outcome = serde_json::to_vec_pretty(&serde_json::json!({
        "operation_id": operation, "target_id": TARGET, "runtime_version": VERSION,
        "source_commit": catalog::KATAGO_SOURCE, "hardware": hardware,
        "cache_path": cache, "post_repair_inference": true
    })).map_err(|_| "managed_cache_identity_invalid")?;
    for (name, bytes) in [("defaults.cfg", defaults.as_bytes()), ("qualification.json", outcome.as_slice())] {
        let mut file = OpenOptions::new().write(true).create_new(true).open(staging.join(name)).map_err(|_| "managed_cache_write_failed")?;
        file.write_all(bytes).and_then(|_| file.sync_all()).map_err(|_| "managed_cache_write_failed")?;
    }
    Ok(())
}
pub(super) fn adoption_profile(mut profile: app_model::EngineProfileDto, installation: &app_model::ManagedInstallationDto, cache: &Path) -> Result<app_model::EngineProfileDto, String> {
    let defaults = cache.join("defaults.cfg");
    let expected = cache_defaults(cache)?;
    if fs::metadata(&defaults).map_err(|_| "managed_cache_defaults_missing")?.len() != expected.len() as u64
        || fs::read(&defaults).map_err(|_| "managed_cache_defaults_missing")? != expected.as_bytes() {
        return Err("managed_cache_defaults_changed".into());
    }
    let settings = match &mut profile.adapter {
        app_model::EngineAdapterSettings::KataGoAnalysis(settings) | app_model::EngineAdapterSettings::KataGoGtp(settings) => settings,
        _ => return Err("managed_repair_requires_katago_profile".into()),
    };
    let config = match &settings.config_path {
        Some(path) if path.trim().is_empty() => return Err("managed_original_config_unreadable".into()),
        Some(path) => std::path::PathBuf::from(profile.working_dir.as_deref().unwrap_or(".")).join(path),
        None => std::path::PathBuf::from(&installation.config_path),
    };
    let deadline = Instant::now() + Duration::from_secs(30);
    // Upstream applies config files in order, then all CLI overrides. This default is lowest
    // precedence, so custom homeDataDir (including includes and aliases) remains authoritative.
    let mut argv = Vec::with_capacity(profile.argv.len() + 2);
    argv.push("-config".into());
    argv.push(config.to_string_lossy().into_owned());
    argv.append(&mut profile.argv);
    profile.argv = argv;
    profile.program = installation.program.clone();
    settings.model_path = Some(installation.model_path.clone());
    settings.config_path = Some(defaults.to_string_lossy().into_owned());
    let spec = crate::build_command_spec(&profile)
        .map_err(|_| "managed_original_config_unreadable".to_string())?;
    crate::katago_config::Configuration::from_command(&spec, deadline)
        .map_err(|_| "managed_original_config_unreadable".to_string())?;
    Ok(profile)
}
pub(super) fn prepare(directory: &Path, staging: &Path, network: &NetworkOperation, mut progress: impl FnMut(u64, &str)) -> Result<(), String> {
    let mut completed = 0;
    let mut installed = BTreeSet::new();
    for package in PACKAGES.iter() {
        let archive = staging.join(format!("{}.zip", package.name));
        let label = format!("{} {}", package.name, package.version);
        network.download(&package.url, &archive, package.size, ORIGINS, |bytes| progress(completed + bytes, &label)).map_err(|e| e.message)?;
        package::verify_file(&archive, package.size, &package.sha256, network)?;
        extract_runtime(&archive, directory, network, &mut installed, package.name == "baseline-runtime")?;
        fs::remove_file(&archive).map_err(|_| "managed_runtime_archive_retirement_failed")?;
        completed += package.size;
    }
    if installed.len() != FILES.len() { return Err("managed_runtime_component_missing".into()); }
    verify(directory, network)
}
pub(super) fn verify(directory: &Path, network: &NetworkOperation) -> Result<(), String> {
    for file in FILES.iter() {
        package::verify_file(&directory.join(&file.file), file.size_bytes, &file.sha256, network)
            .map_err(|error| format!("{}: {error}", file.file))?;
    }
    Ok(())
}
fn extract_runtime(archive: &Path, destination: &Path, network: &NetworkOperation, installed: &mut BTreeSet<String>, baseline: bool) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(File::open(archive).map_err(|_| "managed_archive_unreadable")?).map_err(|_| "managed_archive_invalid")?;
    // SDK archives contain headers/samples; only the 31 authenticated runtime DLLs are materialized.
    if archive.len() > 65536 { return Err("managed_archive_limit".into()); }
    let mut names = BTreeSet::new();
    for index in 0..archive.len() {
        network.lease().check().map_err(|_| "managed_cancelled")?;
        let mut entry = archive.by_index(index).map_err(|_| "managed_archive_invalid")?;
        let path = entry.name().trim_end_matches('/');
        if !package::safe_path(path) || !names.insert(path.to_ascii_lowercase()) { return Err("managed_archive_unsafe_path".into()); }
        let kind = entry.unix_mode().unwrap_or(0) & 0o170000;
        if !matches!(kind, 0 | 0o100000 | 0o040000) { return Err("managed_archive_unsafe_type".into()); }
        if entry.is_dir() { continue; }
        let name = path.rsplit('/').next().ok_or("managed_archive_unsafe_path")?;
        if baseline && !name.starts_with("msvcp") && !name.starts_with("vcruntime") { continue; }
        let Some(expected) = FILES.iter().find(|f| f.file.eq_ignore_ascii_case(name)) else { continue; };
        // Packages can include an import stub or another architecture with the same basename.
        // Never accept it as the runtime: size and digest must match the frozen build receipt.
        if entry.size() != expected.size_bytes { return Err(format!("{}: managed_runtime_size_mismatch", expected.file)); }
        if !installed.insert(expected.file.clone()) { return Err("managed_runtime_duplicate".into()); }
        let output = destination.join(&expected.file);
        let mut file = OpenOptions::new().write(true).create_new(true).open(&output).map_err(|_| "managed_extraction_failed")?;
        let mut buffer = [0; 65536];
        let mut count = 0u64;
        loop {
            network.lease().check().map_err(|_| "managed_cancelled")?;
            let bytes = entry.read(&mut buffer).map_err(|_| "managed_archive_invalid")?;
            if bytes == 0 { break; }
            count += bytes as u64;
            if count > expected.size_bytes { return Err("managed_archive_limit".into()); }
            file.write_all(&buffer[..bytes]).map_err(|_| "managed_extraction_failed")?;
        }
        file.sync_all().map_err(|_| "managed_extraction_failed")?;
        package::verify_file(&output, expected.size_bytes, &expected.sha256, network)
            .map_err(|error| format!("{}: {error}", expected.file))?;
    }
    Ok(())
}

fn unknown(reason: &str) -> ManagedHardwareDto {
    ManagedHardwareDto { status: "unknown".into(), gpu_name: None, gpu_uuid: None, driver_version: None, compute_capability: None, reason: reason.into() }
}
pub(super) fn hardware() -> ManagedHardwareDto {
    if !cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        let mut result = unknown("managed_trt_requires_windows_x64");
        result.status = "unsupported".into();
        return result;
    }
    let Some(root) = std::env::var_os("SystemRoot") else { return unknown("managed_gpu_probe_unavailable"); };
    let program = std::path::PathBuf::from(root).join("System32").join("nvidia-smi.exe");
    let mut command = std::process::Command::new(program);
    command.args(["--query-gpu=uuid,name,driver_version,compute_cap", "--format=csv,noheader,nounits"]);
    match inventory_output(command) { Ok(output) => classify(&output), Err(reason) => unknown(reason) }
}
fn classify(output: &str) -> ManagedHardwareDto {
    let rows: Vec<_> = output.lines().filter(|s| !s.trim().is_empty()).collect();
    if rows.len() != 1 { return unknown("managed_gpu_requires_one_unambiguous_device"); }
    let fields: Vec<_> = rows[0].split(',').map(str::trim).collect();
    if fields.len() != 4 || !fields[0].starts_with("GPU-") || fields.iter().any(|s| s.len() > 160 || s.chars().any(char::is_control)) {
        return unknown("managed_gpu_probe_invalid");
    }
    let Some(driver) = fields[2].split('.').next().and_then(|s| s.parse::<u32>().ok()) else { return unknown("managed_gpu_driver_unknown"); };
    let Some((major, minor)) = fields[3].split_once('.').and_then(|(a,b)| Some((a.parse::<u32>().ok()?, b.parse::<u32>().ok()?))) else { return unknown("managed_gpu_compute_unknown"); };
    // Version-specific NVIDIA TRT 10.9 / CUDA 12.x minimum. This admits a repair attempt,
    // never runtime readiness; the actual frozen executable/model must subsequently infer.
    let (status, reason) = if driver < 535 || (major, minor) < (7, 5) {
        ("unsupported", "managed_gpu_version_incompatible")
    } else if matches!((major, minor), (7,5) | (8,0) | (8,6) | (8,7) | (8,9) | (9,0) | (10,0) | (10,1) | (12,0)) {
        ("supported", "managed_gpu_requires_post_repair_inference")
    } else { ("unknown", "managed_gpu_compute_unqualified") };
    ManagedHardwareDto { status: status.into(), gpu_uuid: Some(fields[0].into()), gpu_name: Some(fields[1].into()), driver_version: Some(fields[2].into()), compute_capability: Some(fields[3].into()), reason: reason.into() }
}
fn inventory_output(mut command: std::process::Command) -> Result<String, &'static str> {
    command.stdin(std::process::Stdio::null()).stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null());
    #[cfg(windows)] { use std::os::windows::process::CommandExt; command.creation_flags(0x08000000); }
    let mut child = command.spawn().map_err(|_| "managed_gpu_probe_unavailable")?;
    let stdout = child.stdout.take().ok_or("managed_gpu_probe_unavailable")?;
    // The query is a small inventory, bounded independently from runtime compilation/inference.
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.take(16385).read_to_end(&mut bytes).map(|_| bytes)
    });
    let deadline = Instant::now() + Duration::from_secs(5);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            _ => { let _ = child.kill(); let _ = child.wait(); break Err("managed_gpu_probe_failed"); }
        }
    };
    let bytes = reader.join().map_err(|_| "managed_gpu_probe_failed")?.map_err(|_| "managed_gpu_probe_failed")?;
    if !status?.success() || bytes.len() > 16384 { return Err("managed_gpu_probe_failed"); }
    String::from_utf8(bytes).map_err(|_| "managed_gpu_probe_invalid")
}

pub(super) fn available_disk(path: &Path) -> Option<u64> {
    let mut existing = path;
    while !existing.exists() { existing = existing.parent()?; }
    #[cfg(windows)] {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = existing.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut available = 0;
        let ok = unsafe { windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(wide.as_ptr(), &mut available, std::ptr::null_mut(), std::ptr::null_mut()) };
        if ok != 0 { Some(available) } else { None }
    }
    #[cfg(unix)] {
        use std::os::unix::ffi::OsStrExt;
        let path = std::ffi::CString::new(existing.as_os_str().as_bytes()).ok()?;
        let mut status = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        if unsafe { libc::statvfs(path.as_ptr(), status.as_mut_ptr()) } != 0 { return None; }
        let status = unsafe { status.assume_init() };
        Some(status.f_bavail * status.f_frsize)
    }
    #[cfg(not(any(windows, unix)))] { None }
}

pub(super) fn qualify(contents: &Path, scratch: &Path, model: &str, expected: &ManagedHardwareDto, network: &NetworkOperation) -> Result<(), String> {
    if hardware() != *expected { return Err("managed_gpu_changed".into()); }
    let cancel = crate::AnalysisCancelToken::new();
    // The established bounded analysis runner owns and reaps this explicit maintenance probe.
    // It never promotes a Run, touches the current game, or serves foreground parameter reads.
    let spec = crate::CommandSpec {
        program: contents.join("katago.exe").to_string_lossy().into_owned(),
        args: vec!["analysis".into(), "-model".into(), contents.join(model).to_string_lossy().into_owned(), "-config".into(), contents.join("analysis_example.cfg").to_string_lossy().into_owned(), "-override-config".into(),
            "numAnalysisThreads=1,numSearchThreadsPerAnalysisThread=1,numNNServerThreadsPerModel=1,nnMaxBatchSize=1,nnCacheSizePowerOfTwo=16,nnMutexPoolSizePowerOfTwo=10,maxBoardSizeForNNBuffer=19,trtDeviceToUse=0,homeDataDir=.,logToStderr=true".into()],
        working_dir: Some(scratch.to_string_lossy().into_owned()),
        env: vec![("CUDA_VISIBLE_DEVICES".into(), expected.gpu_uuid.clone().ok_or("managed_gpu_unknown")?)],
    };
    let done = std::sync::atomic::AtomicBool::new(false);
    let result = std::thread::scope(|scope| {
        scope.spawn(|| {
            while !done.load(std::sync::atomic::Ordering::Acquire) {
                if network.lease().check().is_err() { cancel.cancel(); break; }
                std::thread::sleep(Duration::from_millis(20));
            }
        });
        // TRT's first model load builds a GPU-specific plan before the one-visit response.
        // A separate explicit 15-minute maintenance deadline does not alter normal Run budgets.
        let result = crate::run_katago_analysis_batch_with_options(&spec,
            r#"{"id":"managed-trt-qualification","moves":[],"rules":"chinese","komi":7.5,"boardXSize":19,"boardYSize":19,"maxVisits":1}"#,
            crate::AnalysisBatchRunOptions { expected_responses: 1, timeout: Duration::from_secs(900), cancel_token: Some(&cancel), on_progress: None });
        done.store(true, std::sync::atomic::Ordering::Release);
        result
    }).map_err(qualification_error)?;
    network.lease().check().map_err(|_| "managed_cancelled")?;
    let response: serde_json::Value = serde_json::from_str(result.response_jsonl_lines.first().ok_or("managed_runtime_no_response")?).map_err(|_| "managed_runtime_invalid_response")?;
    validate_inference(&response)?;
    if hardware() != *expected { return Err("managed_gpu_changed".into()); }
    verify(contents, network)?;
    package::verify_package(contents, TARGET, &catalog::frozen().assets[TARGET], network)
}

fn validate_inference(response: &serde_json::Value) -> Result<(), String> {
    // At maxVisits=1 KataGo evaluates the root without expanding a child: moveInfos may be empty.
    if response["id"] != "managed-trt-qualification" || response.get("error").is_some()
        || response["rootInfo"]["visits"].as_u64().unwrap_or(0) == 0
        || !response["rootInfo"]["winrate"].as_f64().is_some_and(|v| v.is_finite() && (0.0..=1.0).contains(&v))
        || !response["rootInfo"]["scoreLead"].as_f64().is_some_and(f64::is_finite)
        || !response["moveInfos"].is_array() {
        return Err("managed_runtime_inference_failed".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frozen_native_cache_path_regression_preserves_full_installation_identity() {
        let app = Path::new("C:/Users/admin/AppData/Roaming/org.lizzieyzy.next.acceptance.refae8b9a56eb4e739dd454047760fc20");
        let id = "a5366b1c-1df7-472f-af45-508a9257f1ec";
        let model = "kata1-tf3-b11c768-s12002M-d6304M";
        let old = app.join("managed-resources").join(format!("staging-{id}")).join("qualification");
        assert!(cache_path_units(&old, model) > 260);
        for prefix in ["p", "i"] {
            let corrected = app.join("trt").join(format!("{prefix}{}", id.replace('-', "")));
            assert!(cache_path_units(&corrected, model) <= 260);
        }
    }

    #[test]
    fn qualification_failure_retains_bounded_terminal_reason() {
        let stderr = format!("{}\nERROR: cache temporary file write failed", "startup config\n".repeat(500));
        let error = qualification_error(crate::EngineManagerError::NonZeroExit { exit_code: Some(-1073740791), stdout: None, stderr });
        assert!(error.starts_with("managed_runtime_process_failed exit_code=Some(-1073740791)"));
        assert!(error.contains("[earlier stderr omitted]"));
        assert!(error.ends_with("ERROR: cache temporary file write failed"));
        assert!(error.chars().count() < 2200);
    }

    #[test]
    fn one_visit_root_inference_is_qualified_without_child_expansion() {
        let mut response = serde_json::json!({ "id": "managed-trt-qualification", "moveInfos": [], "rootInfo": { "visits": 1, "winrate": 0.336831525, "scoreLead": -0.773554206 }});
        assert_eq!(validate_inference(&response), Ok(()));
        response["rootInfo"]["visits"] = 0.into();
        assert!(validate_inference(&response).is_err());
        response["rootInfo"]["visits"] = 1.into();
        response["rootInfo"]["winrate"] = serde_json::Value::Null;
        assert!(validate_inference(&response).is_err());
    }

    #[test]
    fn exact_size_forged_runtime_cannot_acquire_trusted_identity() {
        let root = std::env::temp_dir().join(format!("trt-forged-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(root.join("stage")).unwrap();
        let expected = FILES.iter().min_by_key(|file| file.size_bytes).unwrap();
        let archive = root.join("forged.zip");
        let mut zip = zip::ZipWriter::new(File::create(&archive).unwrap());
        zip.start_file(&expected.file, zip::write::SimpleFileOptions::default()).unwrap();
        zip.write_all(&vec![0; expected.size_bytes as usize]).unwrap();
        zip.finish().unwrap();
        let network = provider_core::network::NetworkState::default().begin_resource(0).unwrap();
        let error = extract_runtime(&archive, &root.join("stage"), &network, &mut BTreeSet::new(), false).unwrap_err();
        assert!(error.ends_with("managed_digest_mismatch"));
        assert!(!root.join("installed").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn repair_adoption_preserves_custom_sources_and_rejects_unreadable_includes() {
        let root = std::env::temp_dir().join(format!("trt-配置, space-{}", uuid::Uuid::new_v4()));
        let cache = root.join("cache");
        fs::create_dir_all(&cache).unwrap();
        fs::write(cache.join("defaults.cfg"), cache_defaults(&cache).unwrap()).unwrap();
        fs::write(root.join("original.cfg"), "@include \"custom.cfg\"\nnumSearchThreads = 2\n").unwrap();
        fs::write(root.join("custom.cfg"), "homeDataDir = \"custom,缓存\"\n").unwrap();
        fs::write(root.join("katago.exe"), b"fixture executable").unwrap();
        fs::write(root.join("model.bin.gz"), b"fixture model").unwrap();
        let mut original = crate::default_engine_profiles_settings().profiles[0].profile.clone();
        original.working_dir = Some(root.to_string_lossy().into_owned());
        original.argv = vec!["-override-config".into(), "homeDataDir=explicit,numSearchThreads=3".into()];
        match &mut original.adapter {
            app_model::EngineAdapterSettings::KataGoAnalysis(settings) => settings.config_path = Some("original.cfg".into()),
            _ => panic!("default fixture must be KataGo"),
        }
        let installation = app_model::ManagedInstallationDto {
            target_id: TARGET.into(), model_id: "b11-flagship".into(), program: root.join("katago.exe").to_string_lossy().into_owned(),
            model_path: root.join("model.bin.gz").to_string_lossy().into_owned(), config_path: root.join("frozen.cfg").to_string_lossy().into_owned(),
            manifest_sha256: "fixture".into(), repair_config_path: Some(cache.join("defaults.cfg").to_string_lossy().into_owned()),
        };
        let draft = adoption_profile(original.clone(), &installation, &cache).unwrap();
        assert_eq!(&draft.argv[2..], original.argv.as_slice());
        assert_eq!(draft.argv[0], "-config");
        assert_eq!(Path::new(&draft.argv[1]), root.join("original.cfg"));
        assert_eq!(draft.working_dir, original.working_dir);
        assert_eq!(draft.name, original.name);
        let configuration = crate::katago_config::Configuration::from_command(
            &crate::build_command_spec(&draft).unwrap(), Instant::now() + Duration::from_secs(30)).unwrap();
        assert_eq!(configuration.files, [cache.join("defaults.cfg"), root.join("original.cfg"), root.join("custom.cfg")]);
        assert_eq!(configuration.file_digests.len(), 3);
        let run = app_model::EngineRunDto { run_id: "repair-adoption".into(), profile_id: "original".into(),
            adapter_kind: draft.adapter_kind(), profile_snapshot: draft.clone(),
            capability_snapshot: None, qualified_resource: None };
        let sources = configuration.thread_sources(run.adapter_kind, &crate::diagnostics::AttemptCapture::new(&run));
        assert_eq!(sources.saved, Some(2));
        assert_eq!(sources.launch_override, Some(3));
        assert_eq!(sources.effective, Some(3));
        assert_eq!(fs::read_to_string(root.join("original.cfg")).unwrap(), "@include \"custom.cfg\"\nnumSearchThreads = 2\n");
        assert_eq!(fs::read_to_string(root.join("custom.cfg")).unwrap(), "homeDataDir = \"custom,缓存\"\n");
        assert!(cache_defaults(&cache).unwrap().contains("配置, space"));
        fs::remove_file(root.join("custom.cfg")).unwrap();
        assert!(adoption_profile(original.clone(), &installation, &cache).is_err());
        fs::write(root.join("custom.cfg"), "homeDataDir = existing\n").unwrap();
        fs::write(cache.join("defaults.cfg"), "homeDataDir = forged\n").unwrap();
        assert_eq!(adoption_profile(original, &installation, &cache).unwrap_err(), "managed_cache_defaults_changed");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn runtime_archive_failures_and_cancellation_preserve_last_good() {
        for (name, cancel) in [("../escape", false), ("nvinfer_10.dll", false), ("safe.txt", true)] {
            let root = std::env::temp_dir().join(format!("trt-archive-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(root.join("stage")).unwrap();
            fs::write(root.join("last-good"), b"preserved").unwrap();
            let archive = root.join("runtime.zip");
            let mut zip = zip::ZipWriter::new(File::create(&archive).unwrap());
            zip.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
            zip.write_all(b"untrusted").unwrap();
            zip.finish().unwrap();
            let network = provider_core::network::NetworkState::default().begin_resource(0).unwrap();
            if cancel { network.cancel(); }
            assert!(extract_runtime(&archive, &root.join("stage"), &network, &mut BTreeSet::new(), false).is_err());
            assert_eq!(fs::read(root.join("last-good")).unwrap(), b"preserved");
            assert!(!root.join("escape").exists());
            assert!(!root.join("stage/nvinfer_10.dll").exists());
            fs::remove_dir_all(root).unwrap();
        }
    }

    #[test]
    #[ignore = "downloads pinned NVIDIA runtime into LIZZIEYZY_TRT_ASSEMBLY_ROOT; package and transport evidence only, never GPU qualification"]
    fn real_public_runtime_assembly_matches_frozen_engine_receipt() {
        let root = std::path::PathBuf::from(std::env::var("LIZZIEYZY_TRT_ASSEMBLY_ROOT").unwrap());
        fs::create_dir(&root).unwrap();
        let contents = root.join("contents");
        fs::create_dir(&contents).unwrap();
        let network = provider_core::network::NetworkState::default().begin_resource(0).unwrap();
        let asset = &catalog::frozen().assets[TARGET];
        let archive = root.join("engine.zip");
        network.download(&catalog::asset_url(asset), &archive, asset.size_bytes, catalog::ORIGINS, |_| {}).unwrap();
        package::verify_file(&archive, asset.size_bytes, &asset.sha256, &network).unwrap();
        package::extract(&archive, &contents, &network).unwrap();
        prepare(&contents, &root, &network, |bytes, component| { if bytes % (64 * 1024 * 1024) < 65536 { println!("{component}: {bytes}"); } }).unwrap();
        package::verify_package(&contents, TARGET, asset, &network).unwrap();
        fs::write(root.join("assembly-receipt.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "target": TARGET, "runtime": VERSION, "source": catalog::KATAGO_SOURCE,
            "archiveSha256": asset.sha256, "files": package::content_hashes(&contents, &network).unwrap(),
            "routes": network.routes(), "runtimeQualified": false
        })).unwrap()).unwrap();
    }

    #[test]
    fn hardware_is_version_specific_and_unknown_never_admits_repair() {
        let blackwell = classify("GPU-123, NVIDIA GeForce RTX 5070 Ti Laptop GPU, 591.66, 12.0\n");
        assert_eq!(blackwell.status, "supported");
        assert_eq!(blackwell.reason, "managed_gpu_requires_post_repair_inference");
        assert_eq!(classify("GPU-123, GeForce, 530.0, 8.6").status, "unsupported");
        assert_eq!(classify("GPU-123, GeForce GTX 1080, 591.66, 6.1").status, "unsupported");
        assert_eq!(classify("GPU-123, Future GPU, 591.66, 13.0").status, "unknown");
        assert_eq!(classify("GPU-123, NVIDIA, N/A, 12.0").status, "unknown");
        assert_eq!(classify("").status, "unknown");
        assert_eq!(classify("GPU-1,A,591.66,12.0\nGPU-2,B,591.66,12.0").status, "unknown");
    }
}
