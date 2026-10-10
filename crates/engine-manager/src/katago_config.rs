//! Frozen KataGo 47aadc config grammar: ordered roots, in-place includes, then CLI overrides.
//! Shared by resource inventory and thread provenance; never changes source bytes.
use crate::{diagnostics::AttemptCapture, CommandSpec};
use app_model::{EngineBackend, ThreadSourceEntryDto, ThreadSourcesDto};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};

const MAX_BYTES: u64 = 4 * 1024 * 1024;
const THREAD_KEYS: [&str; 3] = [
    "numSearchThreads",
    "numSearchThreadsPerAnalysisThread",
    "numAnalysisThreads",
];

#[derive(Debug)]
pub(crate) struct ConfigurationError {
    path: Option<PathBuf>,
    message: String,
}
impl From<String> for ConfigurationError {
    fn from(message: String) -> Self {
        Self { path: None, message }
    }
}
impl From<&str> for ConfigurationError {
    fn from(message: &str) -> Self {
        message.to_owned().into()
    }
}
impl ConfigurationError {
    pub(crate) fn diagnostic(&self, capture: &AttemptCapture) -> String {
        match &self.path {
            Some(path) => format!("{} ({})", self.message, capture.alias(&path.to_string_lossy())),
            None => self.message.clone(),
        }
    }
}

pub(crate) struct Assignment {
    key: String,
    value: String,
    path: Option<PathBuf>,
    layer: &'static str,
}

pub(crate) struct Configuration {
    /// Ordered visits, not globally deduplicated: revisiting a root can reapply its values.
    pub(crate) files: Vec<PathBuf>,
    pub(crate) file_digests: Vec<String>,
    assignments: Vec<Assignment>,
    analysis_threads_flag: Option<String>,
}

impl Configuration {
    pub(crate) fn from_command(spec: &CommandSpec, deadline: Instant) -> Result<Self, ConfigurationError> {
        let mut result = Self {
            files: Vec::new(),
            file_digests: Vec::new(),
            assignments: Vec::new(),
            analysis_threads_flag: None,
        };
        let cwd = Path::new(spec.working_dir.as_deref().unwrap_or("."));
        for pair in spec.args.windows(2).filter(|pair| pair[0] == "-config") {
            result.visit(&cwd.join(&pair[1]), deadline, &mut Vec::new(), "saved")?;
        }
        if result.files.is_empty() {
            return Err("KataGo configuration roots are missing".into());
        }
        // KataGo applies all config files before all override-config arguments, irrespective
        // of their textual interleaving with -config. Empty CLI values delete that key.
        for pair in spec.args.windows(2).filter(|pair| pair[0] == "-override-config") {
            for entry in pair[1].split(',') {
                let (key, value) = entry
                    .split_once('=')
                    .ok_or("invalid KataGo CLI config override")?;
                let key = key.trim();
                if THREAD_KEYS.contains(&key) {
                    result.push(key, value.trim().to_owned(), None, "launch_override")?;
                }
            }
        }
        result.analysis_threads_flag = spec
            .args
            .windows(2)
            .find(|pair| pair[0] == "-analysis-threads")
            .map(|pair| pair[1].clone());
        Ok(result)
    }

    /// Resource-only consumers (including managed adoption) use exactly the same traversal.
    pub(crate) fn from_file(path: &Path, deadline: Instant) -> Result<Self, ConfigurationError> {
        let mut result = Self {
            files: Vec::new(),
            file_digests: Vec::new(),
            assignments: Vec::new(),
            analysis_threads_flag: None,
        };
        result.visit(path, deadline, &mut Vec::new(), "saved")?;
        Ok(result)
    }

    fn push(
        &mut self,
        key: &str,
        value: String,
        path: Option<PathBuf>,
        layer: &'static str,
    ) -> Result<(), String> {
        if self.assignments.len() >= 4096 {
            return Err("thread source exceeds 4096 assignments".into());
        }
        self.assignments.push(Assignment {
            key: key.into(),
            value,
            path,
            layer,
        });
        Ok(())
    }

    fn visit(
        &mut self,
        path: &Path,
        deadline: Instant,
        stack: &mut Vec<PathBuf>,
        layer: &'static str,
    ) -> Result<(), ConfigurationError> {
        self.visit_file(path, deadline, stack, layer)
            .map_err(|mut error| {
                if error.path.is_none() {
                    error.path = Some(path.to_owned());
                }
                error
            })
    }

    fn visit_file(
        &mut self,
        path: &Path,
        deadline: Instant,
        stack: &mut Vec<PathBuf>,
        layer: &'static str,
    ) -> Result<(), ConfigurationError> {
        if Instant::now() >= deadline {
            return Err("KataGo configuration traversal timed out".into());
        }
        if stack.len() >= 32 || self.files.len() >= 128 {
            return Err("KataGo config exceeds 32 levels or 128 file visits".into());
        }
        let canonical = path
            .canonicalize()
            .map_err(|_| "KataGo configuration file is inaccessible")?;
        if stack.contains(&canonical) {
            return Err("KataGo configuration include cycle".into());
        }
        let file = File::open(path).map_err(|_| "KataGo configuration cannot be read")?;
        if file
            .metadata()
            .map_err(|_| "KataGo configuration metadata unavailable")?
            .len()
            > MAX_BYTES
        {
            return Err("KataGo configuration exceeds 4 MiB".into());
        }
        let mut text = String::new();
        file.take(MAX_BYTES + 1)
            .read_to_string(&mut text)
            .map_err(|_| "KataGo configuration is not readable UTF-8")?;
        if text.len() as u64 > MAX_BYTES {
            return Err("KataGo configuration grew past 4 MiB".into());
        }
        self.files.push(path.to_owned());
        self.file_digests
            .push(format!("{:x}", Sha256::digest(text.as_bytes())));
        stack.push(canonical);
        let mut local_keys = HashSet::new();
        for raw in text.lines() {
            if Instant::now() >= deadline {
                return Err("KataGo configuration traversal timed out".into());
            }
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('@') {
                let directive = line.split('#').next().unwrap_or("").trim();
                let end = directive
                    .find(|c: char| c.is_ascii_whitespace() || c == '=')
                    .ok_or("invalid KataGo include directive")?;
                if &directive[..end] != "@include" {
                    return Err("unsupported KataGo config directive".into());
                }
                let value = directive[end..]
                    .trim_start_matches(|c: char| c.is_ascii_whitespace() || c == '=')
                    .trim()
                    .trim_matches('\'')
                    .trim_matches('"');
                if value.is_empty() || Path::new(value).is_absolute() {
                    return Err("invalid KataGo relative include path".into());
                }
                self.visit(
                    &path.parent().unwrap_or(Path::new(".")).join(value),
                    deadline,
                    stack,
                    "include",
                )?;
                continue;
            }
            let (key, value) = parse_assignment(line)?;
            if !local_keys.insert(key.to_owned()) {
                return Err("duplicate key within a KataGo configuration file".into());
            }
            if THREAD_KEYS.contains(&key) {
                self.push(key, value, Some(path.to_owned()), layer)?;
            }
        }
        stack.pop();
        Ok(())
    }

    pub(crate) fn thread_sources(
        &self,
        backend: EngineBackend,
        capture: &AttemptCapture,
    ) -> ThreadSourcesDto {
        let mut result = ThreadSourcesDto::default();
        let mut values = HashMap::new();
        for assignment in &self.assignments {
            if assignment.value.is_empty() {
                values.remove(assignment.key.as_str());
            } else {
                values.insert(assignment.key.as_str(), assignment.value.as_str());
            }
            result.entries.push(ThreadSourceEntryDto {
                key: assignment.key.clone(),
                value: assignment.value.parse().ok(),
                source: assignment
                    .path
                    .as_ref()
                    .map_or_else(|| "argv".into(), |path| capture.alias(&path.to_string_lossy())),
                layer: assignment.layer.into(),
            });
        }
        // Derive saved separately so empty/deleted overrides cannot retroactively change it.
        let mut saved = HashMap::new();
        for a in self.assignments.iter().filter(|a| a.layer != "launch_override") {
            saved.insert(a.key.as_str(), a.value.as_str());
        }
        result.saved = effective(&saved, backend).ok().flatten();
        let mut overrides = HashMap::new();
        for a in self.assignments.iter().filter(|a| a.layer == "launch_override") {
            if a.value.is_empty() {
                overrides.remove(a.key.as_str());
            } else {
                overrides.insert(a.key.as_str(), a.value.as_str());
            }
        }
        result.launch_override = effective(&overrides, backend).ok().flatten();
        match effective(&values, backend) {
            Ok(value) => result.effective = value,
            Err(error) => result.error = Some(error),
        }
        result.analysis_threads = values
            .get("numAnalysisThreads")
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|n| (1..=16384).contains(n));
        if backend == EngineBackend::KataGoAnalysis {
            if let Some(flag) = &self.analysis_threads_flag {
                let value = flag.parse::<u32>().ok().filter(|n| (1..=16384).contains(n));
                result.entries.push(ThreadSourceEntryDto {
                    key: "-analysis-threads".into(),
                    value,
                    source: "argv".into(),
                    layer: "launch_flag".into(),
                });
                if values.contains_key("numAnalysisThreads") || value.is_none() {
                    result.analysis_threads = None;
                    result.error = Some(
                        "JSONL -analysis-threads conflicts with numAnalysisThreads or has an invalid value"
                            .into(),
                    );
                } else {
                    result.analysis_threads = value;
                }
            }
        }
        result
    }
}

fn effective(values: &HashMap<&str, &str>, backend: EngineBackend) -> Result<Option<u32>, String> {
    let legacy = values.get("numSearchThreads");
    let alias = (backend == EngineBackend::KataGoAnalysis)
        .then(|| values.get("numSearchThreadsPerAnalysisThread"))
        .flatten();
    if legacy.is_some() && alias.is_some() {
        return Err("JSONL config cannot contain both search-thread aliases".into());
    }
    legacy
        .or(alias)
        .map(|value| {
            value
                .parse::<u32>()
                .ok()
                .filter(|n| (1..=4096).contains(n))
                .ok_or_else(|| "effective thread source is outside 1..4096".to_owned())
        })
        .transpose()
}

fn parse_assignment(line: &str) -> Result<(&str, String), String> {
    let (key, value) = line
        .split_once('=')
        .ok_or("invalid KataGo key/value assignment")?;
    let key = key.trim();
    if key.is_empty()
        || !key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err("invalid KataGo configuration key".into());
    }
    let value = value.trim_start();
    let parsed = if let Some(quoted) = value.strip_prefix('"') {
        let mut result = String::new();
        let mut chars = quoted.char_indices();
        let mut closed = false;
        while let Some((index, ch)) = chars.next() {
            match ch {
                '\\' => result.push(chars.next().ok_or("invalid KataGo config escape")?.1),
                '"' => {
                    let rest = quoted[index + 1..].trim();
                    if !rest.is_empty() && !rest.starts_with('#') {
                        return Err("invalid text after KataGo quoted value".into());
                    }
                    closed = true;
                    break;
                }
                _ => result.push(ch),
            }
        }
        if !closed {
            return Err("unterminated KataGo quoted value".into());
        }
        result
    } else {
        value.split('#').next().unwrap_or("").trim().to_owned()
    };
    if parsed.is_empty() {
        return Err("empty KataGo config value".into());
    }
    Ok((key, parsed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::{EngineAdapterSettings, EngineProfileDto, EngineRunDto, KataGoSettings};
    use std::time::Duration;

    #[test]
    fn actual_ordered_files_alias_deletion_and_cli_precedence_preserve_sources() {
        let dir = std::env::temp_dir().join(format!("config-sources-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let samples = [("first", "numSearchThreads=2\n@include = \"shared\"\n"),
            ("shared", "numSearchThreads=3\n"),
            ("second", "numSearchThreads=4\n@include 'shared'\nnumSearchThreadsPerAnalysisThread=7\nnumAnalysisThreads=2\n")];
        for (name, text) in samples {
            std::fs::write(dir.join(name), text).unwrap();
        }
        let mut spec = CommandSpec {
            program: "katago".into(),
            working_dir: Some(dir.to_string_lossy().into()),
            env: vec![],
            args: [
                "analysis",
                "-config",
                "first",
                "-override-config",
                "numSearchThreads=,numAnalysisThreads=5",
                "-config",
                "second",
            ]
            .map(str::to_owned)
            .to_vec(),
        };
        let profile = EngineProfileDto {
            name: "sources".into(),
            program: "katago".into(),
            argv: vec![],
            working_dir: None,
            adapter: EngineAdapterSettings::KataGoAnalysis(KataGoSettings {
                model_path: None,
                config_path: None,
                max_visits: 4,
            }),
        };
        let capture = AttemptCapture::new(&EngineRunDto {
            run_id: "source-run".into(),
            profile_id: "source-profile".into(),
            adapter_kind: EngineBackend::KataGoAnalysis,
            profile_snapshot: profile,
            capability_snapshot: None,
            qualified_resource: None,
        });
        let config = Configuration::from_command(&spec, Instant::now() + Duration::from_secs(2)).unwrap();
        assert_eq!(
            config
                .files
                .iter()
                .map(|path| path.file_name().unwrap().to_str().unwrap())
                .collect::<Vec<_>>(),
            ["first", "shared", "second", "shared"]
        );
        let result = config.thread_sources(EngineBackend::KataGoAnalysis, &capture);
        assert_eq!(result.saved, None); // Both JSONL aliases coexist before explicit deletion.
        assert_eq!(result.effective, Some(7));
        assert_eq!(result.analysis_threads, Some(5));
        assert_eq!(result.error, None);
        let gtp = config.thread_sources(EngineBackend::KataGoGtp, &capture);
        assert_eq!(gtp.effective, None); // The per-analysis alias is not a GTP parameter.
        spec.args.extend([
            "-override-config".into(),
            "numSearchThreadsPerAnalysisThread=,numSearchThreads=6".into(),
        ]);
        let result = Configuration::from_command(&spec, Instant::now() + Duration::from_secs(2))
            .unwrap()
            .thread_sources(EngineBackend::KataGoAnalysis, &capture);
        assert_eq!(result.effective, Some(6));
        assert_eq!(result.launch_override, Some(6));
        for (name, text) in samples {
            assert_eq!(std::fs::read_to_string(dir.join(name)).unwrap(), text);
        }
        std::fs::write(dir.join("shared"), "@include first\n").unwrap();
        assert!(
            Configuration::from_command(&spec, Instant::now() + Duration::from_secs(2))
                .err()
                .unwrap()
                .message
                .contains("cycle")
        );
        assert!(Configuration::from_command(&spec, Instant::now())
            .err()
            .unwrap()
            .message
            .contains("timed out"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
