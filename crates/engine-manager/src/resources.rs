use crate::diagnostics::AttemptCapture;
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
    diagnostics: AttemptCapture,
    managed: Option<(crate::managed::trust::ManagedIdentity, PathBuf, PathBuf)>,
}

pub(crate) type ResourceError = (EngineFailureKind, String);

fn managed_authentication_stage<T>(deadline: &mut Instant, verify: impl FnOnce() -> Result<T, String>) -> Result<T, ResourceError> {
    let started = Instant::now();
    if started >= *deadline { return Err((EngineFailureKind::Timeout, "local resource qualification timed out before managed authentication".into())); }
    let result = verify();
    // Managed authentication has its own finite whole-stage/per-file limits and retirement lease.
    // Preserve the unspent local budget: a multi-gigabyte package scan is not local receipt I/O.
    *deadline += started.elapsed();
    result.map_err(|message| {
        let kind = match message.as_str() {
            "managed_verification_timeout" => EngineFailureKind::Timeout,
            "managed_verification_retired" => EngineFailureKind::Cancellation,
            _ => EngineFailureKind::ResourceChanged,
        };
        (kind, message)
    })
}

impl ResourceSnapshot {
    pub(crate) fn capture(
        run: &EngineRunDto,
        spec: &crate::CommandSpec,
        mut deadline: Instant,
        diagnostics: AttemptCapture,
        managed_root: Option<&Path>,
        is_retired: &(dyn Fn() -> bool + Sync),
    ) -> Result<Self, ResourceError> {
        let revision = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&run.profile_snapshot).expect("profile serialization"))
        );
        let executable = resolve_executable(&spec.program)?;
        let mut snapshot = Self {
            entries: Vec::new(),
            revision,
            diagnostics,
            managed: None,
        };
        snapshot.add(executable.clone(), "executable", deadline)?;
        if let app_model::EngineAdapterSettings::KataGoAnalysis(_)
        | app_model::EngineAdapterSettings::KataGoGtp(_) = &run.profile_snapshot.adapter
        {
            snapshot.add(PathBuf::from(&spec.args[4]), "model", deadline)?;
            snapshot.config(PathBuf::from(&spec.args[2]), deadline, 0)?;
        }
        for (index, argument) in run.profile_snapshot.argv.iter().enumerate() {
            let value = argument
                .split_once('=')
                .filter(|(flag, _)| flag.starts_with('-'))
                .map_or(argument.as_str(), |(_, value)| value);
            let path = Path::new(spec.working_dir.as_deref().unwrap_or(".")).join(value);
            let layered_config = argument.starts_with("-config=") || argument.starts_with("--config=")
                || index.checked_sub(1).and_then(|previous| run.profile_snapshot.argv.get(previous)).is_some_and(|flag| flag == "-config" || flag == "--config");
            if layered_config && !matches!(run.profile_snapshot.adapter, app_model::EngineAdapterSettings::GenericGtp(_)) {
                snapshot.config(path, deadline, 0)?;
                continue;
            }
            if path.is_file() {
                snapshot.add(path, "launch_argument", deadline)?;
            }
        }
        if let Some(root) = managed_root {
            if let Some(identity) = managed_authentication_stage(&mut deadline, || crate::managed::trust::qualify(root, &executable, is_retired))? {
                for path in &identity.receipt_paths { snapshot.add(path.clone(), "managed_receipt", deadline)?; }
                snapshot.managed = Some((identity, root.to_path_buf(), executable));
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
                    self.diagnostics.alias(&path.to_string_lossy())
                ),
            )
        })?;
        let identity = identify(&path, component, deadline, &self.diagnostics)?;
        self.entries.push((path.clone(), identity));
        Ok(path)
    }

    fn config(&mut self, path: PathBuf, deadline: Instant, depth: usize) -> Result<(), ResourceError> {
        for path in configuration_files(&path, deadline, depth).map_err(|message| (EngineFailureKind::Config, message))? {
            if self.entries.iter().any(|(existing, _)| existing == &path) { continue; }
            if self.entries.len() >= 128 {
                return Err((EngineFailureKind::Config, "config include graph exceeds 128 resources".into()));
            }
            self.add(path, "config", deadline)?;
        }
        Ok(())
    }

    pub(crate) fn revalidate(&self, mut deadline: Instant, is_retired: &(dyn Fn() -> bool + Sync)) -> Result<(), ResourceError> {
        if let Some((_, root, executable)) = &self.managed {
            managed_authentication_stage(&mut deadline, || crate::managed::trust::qualify(root, executable, is_retired))?
                .ok_or_else(|| (EngineFailureKind::ResourceChanged, "managed identity retired".into()))?;
        }
        for (path, expected) in &self.entries {
            let actual = identify(path, &expected.component, deadline, &self.diagnostics).map_err(|_| {
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
            origin: if self.managed.is_some() { "project-source-build" } else { "local_unknown" }.into(),
            version,
            source_commit: self.managed.as_ref().map(|(identity, _, _)| identity.source_commit.clone()),
            backend,
            static_zlib_exemption: self.managed.as_ref().is_some_and(|(identity, _, _)| identity.static_zlib),
        }
    }
}

/// Shared bounded include traversal for Run identity and explicit managed-resource adoption.
/// This inventories sources; it does not interpret or replace their configuration precedence.
pub(crate) fn configuration_files(path: &Path, deadline: Instant, depth: usize) -> Result<Vec<PathBuf>, String> {
    fn visit(path: &Path, deadline: Instant, depth: usize, files: &mut Vec<PathBuf>) -> Result<(), String> {
        if Instant::now() >= deadline { return Err("config inspection timed out".into()); }
        if depth >= 32 || files.len() >= 128 { return Err("config include graph exceeds 32 levels or 128 resources".into()); }
        let canonical = path.canonicalize().map_err(|_| "config is missing or inaccessible")?;
        if files.contains(&canonical) { return Ok(()); }
        let file = File::open(&canonical).map_err(|_| "config cannot be read")?;
        if file.metadata().map(|m| m.len()).unwrap_or(u64::MAX) > CONFIG_BYTES { return Err("config exceeds 4 MiB".into()); }
        files.push(canonical);
        for line in BufReader::new(file.take(CONFIG_BYTES + 1)).lines() {
            let line = line.map_err(|_| "config is not readable UTF-8")?;
            let line = line.split('#').next().unwrap_or("").trim();
            if line.starts_with('@') {
                let separator = line.find(|c: char| c.is_ascii_whitespace() || c == '=').ok_or("config directive lacks a value")?;
                if &line[..separator] != "@include" { return Err("unsupported config directive".into()); }
                let include = line[separator..].trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '=')
                    .trim().trim_matches('\'').trim_matches('"');
                if include.is_empty() { return Err("config include lacks a path".into()); }
                visit(&path.parent().unwrap_or(Path::new(".")).join(include), deadline, depth + 1, files)?;
            }
        }
        Ok(())
    }
    let mut files = Vec::new();
    visit(path, deadline, depth, &mut files)?;
    Ok(files)
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
    diagnostics: &AttemptCapture,
) -> Result<EngineResourceIdentityDto, ResourceError> {
    let fail = || {
        (
            component_kind(component),
            format!(
                "{component}: cannot read a regular resource file ({})",
                diagnostics.alias(&path.to_string_lossy())
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
        resolved_path: diagnostics.alias(&resolved.to_string_lossy()),
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

    pub(crate) fn summary(&self, diagnostics: &AttemptCapture) -> String {
        diagnostics.sanitize(&String::from_utf8_lossy(&self.tail))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn managed_stage_preserves_only_unspent_local_budget() {
        let remaining = Duration::from_secs(20);
        let original = Instant::now() + remaining;
        let mut deadline = original;
        assert_eq!(managed_authentication_stage(&mut deadline, || {
            std::thread::sleep(Duration::from_millis(5));
            Ok(7)
        }).unwrap(), 7);
        assert!(deadline.duration_since(original) >= Duration::from_millis(5));
        assert!(deadline.saturating_duration_since(Instant::now()) <= remaining);
    }

    #[test]
    fn exhausted_local_budget_cannot_enter_managed_stage() {
        let mut deadline = Instant::now() - Duration::from_secs(1);
        let error = managed_authentication_stage::<()>(&mut deadline, || panic!("expired local admission invoked verifier")).unwrap_err();
        assert_eq!(error.0, EngineFailureKind::Timeout);
    }

    #[test]
    fn managed_stage_preserves_failure_and_retirement_causes() {
        for (message, kind) in [
            ("managed_digest_mismatch", EngineFailureKind::ResourceChanged),
            ("managed_verification_timeout", EngineFailureKind::Timeout),
            ("managed_verification_retired", EngineFailureKind::Cancellation),
        ] {
            let mut deadline = Instant::now() + Duration::from_secs(30);
            assert_eq!(managed_authentication_stage::<()>(&mut deadline, || Err(message.into())).unwrap_err(), (kind, message.into()));
        }
    }
}
