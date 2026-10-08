use app_model::{EngineFailureKind, EngineResourceIdentityDto, EngineRunDto, QualifiedLocalResourceDto};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;

const OUTPUT_BYTES: usize = 8192;
const CONFIG_BYTES: u64 = 4 * 1024 * 1024;

/// Private paths never cross the diagnostics boundary. The wire identity carries a
/// stable path pseudonym and content digest, not the user's home directory.
pub(crate) struct ResourceSnapshot {
    entries: Vec<(PathBuf, EngineResourceIdentityDto)>,
    revision: String,
}

pub(crate) type ResourceError = (EngineFailureKind, String);

impl ResourceSnapshot {
    pub(crate) fn capture(
        run: &EngineRunDto,
        spec: &crate::CommandSpec,
        deadline: Instant,
    ) -> Result<Self, ResourceError> {
        let revision = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&run.profile_snapshot).expect("profile serialization"))
        );
        let executable = resolve_executable(&spec.program)?;
        let mut snapshot = Self {
            entries: Vec::new(),
            revision,
        };
        snapshot.add(executable, "executable", deadline)?;
        if let app_model::EngineAdapterSettings::KataGoAnalysis(_)
        | app_model::EngineAdapterSettings::KataGoGtp(_) = &run.profile_snapshot.adapter
        {
            snapshot.add(PathBuf::from(&spec.args[4]), "model", deadline)?;
            snapshot.config(PathBuf::from(&spec.args[2]), deadline, 0)?;
        }
        for argument in &run.profile_snapshot.argv {
            let value = argument
                .split_once('=')
                .filter(|(flag, _)| flag.starts_with('-'))
                .map_or(argument.as_str(), |(_, value)| value);
            let path = Path::new(spec.working_dir.as_deref().unwrap_or(".")).join(value);
            if path.is_file() {
                snapshot.add(path, "launch_argument", deadline)?;
            }
        }
        Ok(snapshot)
    }

    fn add(&mut self, path: PathBuf, component: &str, deadline: Instant) -> Result<PathBuf, ResourceError> {
        let kind = component_kind(component);
        path.canonicalize().map_err(|_| {
            (
                kind,
                format!(
                    "{component}: file is missing or inaccessible ({})",
                    display_path(&path)
                ),
            )
        })?;
        let identity = identify(&path, component, deadline)?;
        self.entries.push((path.clone(), identity));
        Ok(path)
    }

    fn config(&mut self, path: PathBuf, deadline: Instant, depth: usize) -> Result<(), ResourceError> {
        if depth >= 32 || self.entries.len() >= 128 {
            return Err((
                EngineFailureKind::Config,
                "config include graph exceeds 32 levels or 128 resources".into(),
            ));
        }
        let canonical = path.canonicalize().map_err(|_| {
            (
                EngineFailureKind::Config,
                format!("config is missing ({})", display_path(&path)),
            )
        })?;
        if self
            .entries
            .iter()
            .any(|(p, _)| p.canonicalize().ok().as_ref() == Some(&canonical))
        {
            return Ok(());
        }
        let path = self.add(path, "config", deadline)?;
        let file =
            File::open(&path).map_err(|_| (EngineFailureKind::Config, "config cannot be read".into()))?;
        if file.metadata().map(|m| m.len()).unwrap_or(u64::MAX) > CONFIG_BYTES {
            return Err((EngineFailureKind::Config, "config exceeds 4 MiB".into()));
        }
        for line in BufReader::new(file.take(CONFIG_BYTES + 1)).lines() {
            let line =
                line.map_err(|_| (EngineFailureKind::Config, "config is not readable UTF-8".into()))?;
            let line = line.split('#').next().unwrap_or("").trim();
            if let Some(include) = line.strip_prefix("@include ") {
                let include = include.trim().trim_matches('"');
                self.config(
                    path.parent().unwrap_or(Path::new(".")).join(include),
                    deadline,
                    depth + 1,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn revalidate(&self, deadline: Instant) -> Result<(), ResourceError> {
        for (path, expected) in &self.entries {
            let actual = identify(path, &expected.component, deadline).map_err(|_| {
                (
                    EngineFailureKind::ResourceChanged,
                    format!(
                        "{} changed or became unreadable during qualification",
                        expected.component
                    ),
                )
            })?;
            if &actual != expected {
                return Err((
                    EngineFailureKind::ResourceChanged,
                    format!(
                        "{} content changed during qualification; Start again explicitly",
                        expected.component
                    ),
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn qualified(
        self,
        version: Option<String>,
        backend: Option<String>,
    ) -> QualifiedLocalResourceDto {
        QualifiedLocalResourceDto {
            profile_revision: self.revision,
            resources: self.entries.into_iter().map(|(_, identity)| identity).collect(),
            origin: "local_unknown".into(),
            version,
            source_commit: None,
            backend,
            // No managed trust is inferred from a pathname or a local manifest.
            static_zlib_exemption: false,
        }
    }
}

fn component_kind(component: &str) -> EngineFailureKind {
    match component {
        "model" => EngineFailureKind::Model,
        "config" => EngineFailureKind::Config,
        _ => EngineFailureKind::Executable,
    }
}

fn identify(
    path: &Path,
    component: &str,
    deadline: Instant,
) -> Result<EngineResourceIdentityDto, ResourceError> {
    let fail = || {
        (
            component_kind(component),
            format!(
                "{component}: cannot read a regular resource file ({})",
                display_path(path)
            ),
        )
    };
    let mut file = File::open(path).map_err(|_| fail())?;
    let before = file.metadata().map_err(|_| fail())?;
    if !before.is_file() {
        return Err(fail());
    }
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    let mut bytes = 0;
    loop {
        if Instant::now() >= deadline {
            return Err((
                EngineFailureKind::Timeout,
                format!("{component} content qualification timed out"),
            ));
        }
        let count = file.read(&mut buffer).map_err(|_| fail())?;
        if count == 0 {
            break;
        }
        bytes += count as u64;
        hasher.update(&buffer[..count]);
    }
    let after = file.metadata().map_err(|_| fail())?;
    if bytes != before.len() || after.len() != before.len() || after.modified().ok() != before.modified().ok()
    {
        return Err((
            EngineFailureKind::ResourceChanged,
            format!("{component} changed while hashing"),
        ));
    }
    let resolved = path.canonicalize().map_err(|_| fail())?;
    Ok(EngineResourceIdentityDto {
        component: component.into(),
        resolved_path: display_path(&resolved),
        sha256: format!("{:x}", hasher.finalize()),
        bytes,
    })
}

fn resolve_executable(program: &str) -> Result<PathBuf, ResourceError> {
    let path = Path::new(program);
    if path.components().count() > 1 || path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(program);
            if candidate.is_file() {
                return Ok(candidate);
            }
            #[cfg(windows)]
            if path.extension().is_none() {
                let candidate = candidate.with_extension("exe");
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }
    Err((
        EngineFailureKind::Executable,
        "executable was not found on PATH".into(),
    ))
}

fn display_path(path: &Path) -> String {
    let hash = format!("{:x}", Sha256::digest(path.to_string_lossy().as_bytes()));
    format!(
        "<path:{}>/{}",
        &hash[..12],
        redact(&path.file_name().unwrap_or_default().to_string_lossy())
    )
}

/// Startup stderr is drained continuously; a prefix retains identity and a tail
/// retains the actual loader failure. Neither probe output nor raw bytes is emitted.
#[derive(Default)]
pub(crate) struct StartupOutput {
    prefix: Vec<u8>,
    tail: Vec<u8>,
}

impl StartupOutput {
    pub(crate) fn capture(mut stream: impl Read + Send + 'static) -> Arc<Mutex<Self>> {
        let output = Arc::new(Mutex::new(Self::default()));
        let target = output.clone();
        std::thread::spawn(move || {
            let mut buffer = [0; 4096];
            loop {
                match stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let mut out = target.lock().unwrap_or_else(|e| e.into_inner());
                        let keep = n.min(OUTPUT_BYTES - out.prefix.len());
                        out.prefix.extend_from_slice(&buffer[..keep]);
                        let remove = (out.tail.len() + n).saturating_sub(OUTPUT_BYTES);
                        out.tail.drain(..remove);
                        out.tail.extend_from_slice(&buffer[..n]);
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(_) => break,
                }
            }
        });
        output
    }

    pub(crate) fn identity(&self) -> (Option<String>, Option<String>) {
        let text = String::from_utf8_lossy(&self.prefix);
        let version = text.lines().find_map(|line| {
            let (_, rest) = line.split_once("KataGo v")?;
            let version = rest.split_whitespace().next()?;
            (version.len() <= 64
                && version
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || ".-_+".contains(c)))
            .then(|| version.to_owned())
        });
        let backend = [
            "Eigen", "CUDA", "OpenCL", "TensorRT", "OpenVINO", "DirectML", "ROCm", "Metal",
        ]
        .into_iter()
        .find(|name| text.contains(name))
        .map(str::to_owned);
        (version, backend)
    }

    pub(crate) fn summary(&self) -> String {
        redact(&String::from_utf8_lossy(&self.tail))
    }
    pub(crate) fn failure_kind(&self) -> Option<EngineFailureKind> {
        let text = String::from_utf8_lossy(&self.tail).to_ascii_lowercase();
        if ![
            "error",
            "failed",
            "missing",
            "not found",
            "cannot",
            "could not",
            "unable",
        ]
        .iter()
        .any(|marker| text.contains(marker))
        {
            return None;
        }
        if text.contains("nvonnxparser") || text.contains("tensorrt parser") {
            Some(EngineFailureKind::TensorRtParser)
        } else if text.contains("cudnn") {
            Some(EngineFailureKind::Cudnn)
        } else if text.contains("nvrtc") {
            Some(EngineFailureKind::Nvrtc)
        } else if text.contains("cudart") || text.contains("cuda error") || text.contains("cuda driver") {
            Some(EngineFailureKind::Cuda)
        } else if text.contains("zlib") || text.contains("zlib1.dll") {
            Some(EngineFailureKind::Zlib)
        } else if text.contains("model") && (text.contains("error") || text.contains("failed")) {
            Some(EngineFailureKind::Model)
        } else if text.contains("config") && (text.contains("error") || text.contains("failed")) {
            Some(EngineFailureKind::Config)
        } else {
            None
        }
    }
}

pub(crate) fn redact(text: &str) -> String {
    use regex::Regex;
    use std::sync::LazyLock;
    static PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
        [
        r#"(?i)(?:https?|wss?|ssh)://[^\s\"'<>]+"#,
        r#"(?i)(?:token|password|passwd|secret|authorization|cookie|room[_-]?id|session[_-]?id)\s*[:=]\s*(?:\"[^\"]*\"|'[^']*'|[^\s,;]+)"#,
        r#"(?:[A-Za-z]:[\\/]|/)[^\s\"'<>]*"#,
    ].into_iter().map(|p| Regex::new(p).expect("redaction pattern")).collect()
    });
    let mut text = text.chars().take(OUTPUT_BYTES).collect::<String>();
    for pattern in PATTERNS.iter() {
        text = pattern.replace_all(&text, "<redacted>").into_owned();
    }
    let mut result = String::new();
    for c in text.chars() {
        let escaped = match c {
            '\n' => "\\n".into(),
            '\r' => "\\r".into(),
            '\t' => "\\t".into(),
            '"' => "\\\"".into(),
            '\'' => "\\'".into(),
            c if c.is_control() => "?".into(),
            c => c.to_string(),
        };
        if result.len() + escaped.len() > OUTPUT_BYTES {
            break;
        }
        result.push_str(&escaped);
    }
    result
}
