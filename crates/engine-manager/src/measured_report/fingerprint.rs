use super::{parameters, MeasuredSceneDto, Report};
use crate::{katago_config::Configuration, CommandSpec};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::Metadata,
    io::Read,
    path::Path,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GpuSample {
    pub uuid: String,
    pub name: String,
    pub driver: String,
    pub physical_mib: f64,
    pub used_mib: f64,
}
pub fn sample_gpu() -> Result<GpuSample, String> {
    #[cfg(windows)]
    let program =
        std::path::PathBuf::from(std::env::var_os("SystemRoot").ok_or("GPU query SystemRoot unavailable")?)
            .join("System32/nvidia-smi.exe");
    #[cfg(not(windows))]
    let program = std::path::PathBuf::from("nvidia-smi");
    let mut command = std::process::Command::new(program);
    command.args([
        "--query-gpu=uuid,name,driver_version,memory.total,memory.used",
        "--format=csv,noheader,nounits",
    ]);
    parse_gpu(&crate::managed::trt::inventory_output(command).map_err(str::to_owned)?)
}
pub fn parse_gpu(output: &str) -> Result<GpuSample, String> {
    let mut rows = output.lines().filter(|s| !s.trim().is_empty());
    let line = rows.next().ok_or("GPU query has no device")?;
    if rows.next().is_some() {
        return Err("GPU identity requires exactly one NVIDIA device".into());
    }
    let f: Vec<_> = line.split(',').map(str::trim).collect();
    if f.len() != 5
        || !f[0].starts_with("GPU-")
        || f.iter()
            .any(|s| s.is_empty() || s.len() > 160 || s.chars().any(char::is_control))
    {
        return Err("GPU query has invalid identity".into());
    }
    let physical_mib: f64 = f[3].parse().map_err(|_| "GPU physical memory unavailable")?;
    let used_mib: f64 = f[4].parse().map_err(|_| "GPU sampled memory unavailable")?;
    if !physical_mib.is_finite()
        || physical_mib <= 0.0
        || !used_mib.is_finite()
        || used_mib < 0.0
        || used_mib > physical_mib
    {
        return Err("GPU memory sample invalid".into());
    }
    Ok(GpuSample {
        uuid: f[0].into(),
        name: f[1].into(),
        driver: f[2].into(),
        physical_mib,
        used_mib,
    })
}
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    if !a.is_file() || !b.is_file() || a.len() != b.len() || a.modified().ok() != b.modified().ok() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if a.dev() != b.dev()
            || a.ino() != b.ino()
            || a.ctime() != b.ctime()
            || a.ctime_nsec() != b.ctime_nsec()
        {
            return false;
        }
    }
    true
}
pub fn hash_file(path: &Path, deadline: Instant, cancelled: &dyn Fn() -> bool) -> Result<String, String> {
    #[cfg(windows)]
    let mut file = {
        use std::os::windows::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .share_mode(windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ)
            .open(path)
            .map_err(|_| "report resource unreadable or concurrently writable")?
    };
    #[cfg(unix)]
    let mut file = {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
            .map_err(|_| "report resource unreadable")?
    };
    #[cfg(not(any(unix, windows)))]
    let mut file = std::fs::File::open(path).map_err(|_| "report resource unreadable")?;
    let before = file
        .metadata()
        .map_err(|_| "report resource metadata unavailable")?;
    if !before.is_file() {
        return Err("report resource must be a regular file".into());
    }
    let mut hash = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        if cancelled() {
            return Err("report operation retired".into());
        }
        if Instant::now() >= deadline {
            return Err("report resource hash timed out".into());
        }
        let n = file
            .read(&mut buffer)
            .map_err(|_| "report resource read failed")?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    let after = file
        .metadata()
        .map_err(|_| "report resource metadata unavailable")?;
    let current = std::fs::metadata(path).map_err(|_| "report resource replaced")?;
    if !same_file(&before, &after) || !same_file(&after, &current) {
        return Err("report resource changed during hashing".into());
    }
    Ok(format!("{:x}", hash.finalize()))
}
const EXCLUDED: [&str; 8] = [
    "numSearchThreads",
    "numAnalysisThreads",
    "numSearchThreadsPerAnalysisThread",
    "nnMaxBatchSize",
    "homeDataDir",
    "logDir",
    "logFile",
    "logDirDated",
];
pub fn capture(
    spec: &CommandSpec,
    scene: MeasuredSceneDto,
    gpu: &GpuSample,
    cancelled: &dyn Fn() -> bool,
) -> Result<(Value, std::collections::BTreeMap<String, u32>), String> {
    let mode = match scene {
        MeasuredSceneDto::Live => "gtp",
        MeasuredSceneDto::WholeGame => "analysis",
    };
    if spec.args.first().map(String::as_str) != Some(mode) {
        return Err("unsupported report launch scene".into());
    }
    let executable = Path::new(&spec.program)
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if matches!(
        executable.as_str(),
        "ssh" | "java" | "bash" | "sh" | "cmd" | "powershell" | "pwsh" | "python" | "python3" | "node"
    ) {
        return Err("Reports require direct local KataGo; remote and wrapper targets are unsupported".into());
    }
    let mut index = 1;
    let mut model = None;
    while index < spec.args.len() {
        let flag = spec.args[index].as_str();
        if flag == "-quit-without-waiting" {
            index += 1;
            continue;
        }
        if !matches!(
            flag,
            "-model"
                | "--model"
                | "-weights"
                | "--weights"
                | "-config"
                | "--config"
                | "-override-config"
                | "--override-config"
        ) {
            return Err(format!("unsupported report launch argument: {flag}"));
        }
        let value = spec.args.get(index + 1).ok_or("report launch flag lacks value")?;
        if matches!(flag, "-model" | "--model" | "-weights" | "--weights") {
            model = Some(value);
        }
        index += 2;
    }
    let cwd = Path::new(spec.working_dir.as_deref().unwrap_or("."));
    let deadline = Instant::now() + Duration::from_secs(30);
    let configuration =
        Configuration::from_command(spec, deadline).map_err(|_| "report configuration is invalid")?;
    if configuration.files.is_empty() {
        return Err("report configuration root missing".into());
    }
    if configuration.files.len() > 64 {
        return Err("report configuration exceeds 64 unique files".into());
    }
    let mut unique = HashSet::new();
    let mut digests = Vec::new();
    for (i, path) in configuration.files.iter().enumerate() {
        let canonical = path.canonicalize().map_err(|_| "report include missing")?;
        if !unique.insert(canonical) {
            return Err("report forbids repeated config roots or includes".into());
        }
        let digest = hash_file(path, deadline, cancelled)?;
        if digest != configuration.file_digests[i] {
            return Err("report configuration changed during capture".into());
        }
        digests.push(digest);
    }
    let engine = hash_file(&cwd.join(&spec.program), deadline, cancelled)?;
    let model = hash_file(
        &cwd.join(model.ok_or("report model missing")?),
        deadline,
        cancelled,
    )?;
    let semantics = configuration
        .effective_values(true)
        .into_iter()
        .filter(|(k, _)| !EXCLUDED.contains(&k.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let values = configuration.effective_values(false);
    let keys: &[&str] = match scene {
        MeasuredSceneDto::Live => &super::LIVE_KEYS,
        MeasuredSceneDto::WholeGame => &super::WHOLE_KEYS,
    };
    let mut baseline = serde_json::Map::new();
    for key in keys {
        let raw = if scene == MeasuredSceneDto::WholeGame && *key == "numSearchThreadsPerAnalysisThread" {
            values
                .get("numSearchThreads")
                .filter(|value| !value.is_empty())
                .or_else(|| values.get(*key))
        } else {
            values.get(*key)
        }
        .ok_or_else(|| format!("baseline {key} is unknown"))?;
        let value = raw
            .parse::<u32>()
            .map_err(|_| format!("baseline {key} is unknown or deleted"))?;
        baseline.insert(key.to_string(), json!(value));
    }
    let baseline = parameters(&Value::Object(baseline), scene)?;
    Ok((
        json!({"engineSha256":engine,"modelSha256":model,"configSha256":digests[0],"configIncludes":&digests[1..],"gpuName":gpu.name,"driverVersion":gpu.driver,"memoryMiB":gpu.physical_mib,"commandSemantics":semantics}),
        baseline,
    ))
}
pub fn matches(
    report: &Report,
    fingerprint: &Value,
    baseline: &std::collections::BTreeMap<String, u32>,
) -> Result<(), String> {
    if !report
        .raw
        .get("fingerprint")
        .is_some_and(|actual| semantic_equal(actual, fingerprint))
    {
        return Err("report fingerprint differs from current resources, GPU or command semantics".into());
    }
    if &report.baseline != baseline {
        return Err("report baseline differs from current effective launch policy".into());
    }
    Ok(())
}

fn decimal(number: &serde_json::Number) -> Option<(bool, String, i64)> {
    let text = number.to_string();
    let (mantissa, exponent) = text.split_once(['e', 'E']).unwrap_or((&text, "0"));
    let (whole, fraction) = mantissa
        .trim_start_matches('-')
        .split_once('.')
        .unwrap_or((mantissa.trim_start_matches('-'), ""));
    let mut digits = format!("{whole}{fraction}").trim_start_matches('0').to_owned();
    if digits.is_empty() {
        return Some((false, String::new(), 0));
    }
    let mut power = exponent.parse::<i64>().ok()?.checked_sub(fraction.len() as i64)?;
    while digits.ends_with('0') {
        digits.pop();
        power = power.checked_add(1)?;
    }
    Some((mantissa.starts_with('-'), digits, power))
}
pub fn semantic_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => decimal(a).zip(decimal(b)).is_some_and(|(a, b)| a == b),
        (Value::Object(a), Value::Object(b)) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(k, v)| b.get(k).is_some_and(|other| semantic_equal(v, other)))
        }
        (Value::Array(a), Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| semantic_equal(a, b))
        }
        _ => left == right,
    }
}
