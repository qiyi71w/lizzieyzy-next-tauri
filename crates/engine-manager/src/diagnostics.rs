use app_model::{
    EngineDiagnosticMetricDto, EngineDiagnosticRecordDto, EngineDiagnosticSnapshotDto, EngineRunDto,
};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::io::{self, Read};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

// Diagnostic copies never backpressure protocol delivery. The per-record cap
// prevents a missing newline from retaining an unbounded child write.
pub const RECORD_BYTES: usize = 4096;
pub const CAPTURE_BYTES: usize = 64 * 1024;
pub const CAPTURE_RECORDS: usize = 256;

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

mod metrics;

/// One attempt's alias namespace is retained in its frozen snapshots. No hashes
/// of enumerable room/session identifiers are exposed.
#[derive(Default)]
pub struct DiagnosticSanitizer {
    aliases: Vec<(String, String)>,
    continued_secret: Option<char>,
}
impl DiagnosticSanitizer {
    pub fn sanitize(&mut self, input: &str) -> String {
        if input.len() > RECORD_BYTES {
            return "[overlong record omitted]".into();
        }
        let mut output = String::new();
        for (index, line) in input.split('\n').enumerate() {
            if index > 0 {
                output.push('\n');
            }
            if let Some(quote) = self.continued_secret {
                output.push_str(&self.alias(line));
                if quote == '\0' || has_closing_quote(line, quote) {
                    self.continued_secret = None;
                }
                continue;
            }
            let lower = line.to_ascii_lowercase();
            let sensitive = [
                "token",
                "password",
                "passwd",
                "secret",
                "authorization",
                "session",
                "room",
                "-pw",
                "credential",
            ]
            .iter()
            .filter_map(|word| lower.find(word).map(|at| (at, word.len())))
            .min_by_key(|(at, _)| *at);
            if let Some((at, key_len)) = sensitive {
                let key_len = key_len
                    + line[at + key_len..]
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                        .map(char::len_utf8)
                        .sum::<usize>();
                // Keep the context/key, never guess a shell argument's end from spaces.
                let value = line[at + key_len..]
                    .trim_start_matches(|c: char| c.is_whitespace() || matches!(c, ':' | '=' | '"' | '\''));
                let suffix = line[at + key_len..]
                    .trim_start_matches(|c: char| c.is_whitespace() || matches!(c, ':' | '='));
                if let Some(quote @ ('"' | '\'')) = suffix.chars().next() {
                    if !has_closing_quote(&suffix[quote.len_utf8()..], quote) {
                        self.continued_secret = Some(quote);
                    }
                } else if value.is_empty() {
                    self.continued_secret = Some('\0');
                }
                let prefix = self.sanitize_public(&line[..at + key_len]);
                output.push_str(&prefix);
                output.push('=');
                output.push_str(&self.alias(value.trim_end_matches(['"', '\''])));
            } else {
                output.push_str(&self.sanitize_public(line));
            }
        }
        truncate(&mut output, RECORD_BYTES);
        output
    }
    fn sanitize_public(&mut self, line: &str) -> String {
        // A quoted path can contain spaces. Redact the complete record rather
        // than leaking the trailing components through whitespace tokenization.
        if line.contains('/') || line.contains('\\') {
            return self.alias(line);
        }
        if let Some((raw, alias)) = self
            .aliases
            .iter()
            .find(|(raw, _)| !raw.is_empty() && raw.contains(char::is_whitespace) && line.contains(raw))
        {
            let replaced = line.replace(raw, alias);
            return self.sanitize_public(&replaced);
        }
        let mut output = String::new();
        for word in line.split_inclusive(char::is_whitespace) {
            let lower = word.to_ascii_lowercase();
            let private = lower.contains("://") || word.contains('/') || word.contains('\\');
            if private {
                output.push_str(&self.alias(word.trim()));
                if word.ends_with(char::is_whitespace) {
                    output.push(' ');
                }
            } else {
                let mut remaining = word;
                while !remaining.is_empty() {
                    let known = self
                        .aliases
                        .iter()
                        .filter(|(raw, _)| !raw.is_empty())
                        .filter_map(|(raw, alias)| remaining.find(raw).map(|at| (at, raw.len(), alias)))
                        .min_by_key(|(at, _, _)| *at);
                    if let Some((at, length, alias)) = known {
                        output.extend(remaining[..at].chars().filter(|c| !c.is_control() || *c == '\t'));
                        output.push_str(alias);
                        remaining = &remaining[at + length..];
                    } else {
                        output.extend(remaining.chars().filter(|c| !c.is_control() || *c == '\t'));
                        break;
                    }
                }
            }
        }
        output
    }
    pub fn alias(&mut self, value: &str) -> String {
        if let Some((_, alias)) = self.aliases.iter().find(|(raw, _)| raw == value) {
            return alias.clone();
        }
        // Bound secret retention too; exhaustion stays private, not a raw fallback.
        if self.aliases.len() == 256 || value.len() > RECORD_BYTES {
            return "[redacted]".into();
        }
        let alias = format!("[private-{}]", self.aliases.len() + 1);
        self.aliases.push((value.into(), alias.clone()));
        alias
    }
}
fn has_closing_quote(value: &str, quote: char) -> bool {
    let mut escaped = false;
    for character in value.chars() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == quote {
            return true;
        }
    }
    false
}

fn truncate(text: &mut String, limit: usize) {
    if text.len() > limit {
        let mut end = limit;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        text.truncate(end);
    }
}

/// A public diagnostic copy: static product guidance or text redacted by its owner.
/// Dynamic strings cannot implicitly cross this boundary.
#[derive(Clone)]
pub(crate) struct FailureText(String);
impl From<&'static str> for FailureText {
    fn from(text: &'static str) -> Self {
        Self(text.into())
    }
}
impl FailureText {
    /// Compose already-public fragments without passing trusted guidance through redaction.
    pub(crate) fn then(mut self, detail: impl Into<Self>) -> Self {
        self.0.push_str(&detail.into().0);
        truncate(&mut self.0, RECORD_BYTES);
        self
    }
    pub(crate) fn prefixed(self, prefix: &'static str) -> Self {
        Self::from(prefix).then(self)
    }
    pub(crate) fn into_string(self) -> String {
        self.0
    }
    pub(crate) fn from_failure(failure: &app_model::EngineFailureDto) -> Self {
        Self(failure.message.clone())
    }
    pub(crate) fn unscoped(text: &str) -> Self {
        Self(DiagnosticSanitizer::default().sanitize(text))
    }
    pub(crate) fn dynamic(sanitizer: &mut DiagnosticSanitizer, text: &str) -> Self {
        Self(sanitizer.sanitize(text))
    }
}

#[derive(Clone)]
pub(crate) struct AttemptCapture(Arc<Mutex<Capture>>);
struct Capture {
    run_id: String,
    profile_id: String,
    started: Instant,
    records: VecDeque<EngineDiagnosticRecordDto>,
    bytes: usize,
    sequence: u64,
    dropped: u64,
    sanitizer: DiagnosticSanitizer,
    trace: bool,
    readiness_confirmed: bool,
    command: String,
    failure: Option<String>,
    stdout_complete: bool,
    stderr_complete: bool,
    process_exited: bool,
    exit_code: Option<i32>,
}
impl AttemptCapture {
    pub(crate) fn new(run: &EngineRunDto) -> Self {
        let capture = Self(Arc::new(Mutex::new(Capture {
            run_id: run.run_id.clone(),
            profile_id: run.profile_id.clone(),
            started: Instant::now(),
            records: VecDeque::new(),
            bytes: 0,
            sequence: 0,
            dropped: 0,
            sanitizer: DiagnosticSanitizer::default(),
            trace: false,
            readiness_confirmed: false,
            command: String::new(),
            failure: None,
            stdout_complete: false,
            stderr_complete: false,
            process_exited: false,
            exit_code: None,
        })));
        // argv is structured, never reconstructed with a quoting-sensitive shell parser.
        let mut state = capture.0.lock();
        let program = state.sanitizer.alias(&run.profile_snapshot.program);
        let args: Vec<_> = run
            .profile_snapshot
            .argv
            .iter()
            .take(32)
            .map(|arg| state.sanitizer.alias(arg))
            .collect();
        state.command = format!(
            "program={program} argv={args:?} omitted-args={} adapter={:?}",
            run.profile_snapshot.argv.len().saturating_sub(32),
            run.adapter_kind
        );
        drop(state);
        capture
    }
    pub(crate) fn command(&self, spec: &crate::CommandSpec) {
        let mut state = self.0.lock();
        let program = state.sanitizer.alias(&spec.program);
        let args: Vec<_> = spec
            .args
            .iter()
            .take(32)
            .map(|arg| state.sanitizer.alias(arg))
            .collect();
        let directory = spec
            .working_dir
            .as_deref()
            .map(|path| state.sanitizer.alias(path));
        state.command = format!(
            "program={program} argv={args:?} omitted-args={} working-directory={directory:?}",
            spec.args.len().saturating_sub(32)
        );
    }
    pub(crate) fn alias(&self, value: &str) -> String {
        self.0.lock().sanitizer.alias(value)
    }
    pub(crate) fn sanitize(&self, value: &str) -> String {
        self.0.lock().sanitizer.sanitize(value)
    }
    pub(crate) fn failure_text(&self, value: &str) -> FailureText {
        FailureText(self.sanitize(value))
    }
    // Failure DTOs have already crossed the trusted/redacted text boundary.
    // Re-sanitizing them would interpret product prose and aliases as new secrets.
    pub(crate) fn record_failure(&self, failure: &app_model::EngineFailureDto) {
        let mut state = self.0.lock();
        let mut text = format!("{:?}: {}", failure.kind, failure.message);
        truncate(&mut text, RECORD_BYTES);
        state.failure = Some(text.clone());
        state.push("failure", text);
    }
    pub(crate) fn readiness_confirmed(&self) {
        let mut state = self.0.lock();
        state.readiness_confirmed = true;
        state.push("lifecycle", "readiness confirmed".into());
    }
    pub(crate) fn exited(&self, code: Option<i32>) {
        let mut state = self.0.lock();
        state.process_exited = true;
        state.exit_code = code;
    }
    fn source_finished(&self, source: &str) {
        let mut state = self.0.lock();
        match source {
            "stdout" => state.stdout_complete = true,
            "stderr" => state.stderr_complete = true,
            _ => {}
        }
    }
    pub(crate) fn matches_run(&self, run_id: &str) -> bool {
        self.0.lock().run_id == run_id
    }
    pub(crate) fn trace_enabled(&self) -> bool {
        self.0.lock().trace
    }
    pub(crate) fn set_trace(&self, enabled: bool) {
        self.0.lock().trace = enabled;
    }
    pub(crate) fn trace(&self, text: impl FnOnce() -> String) {
        let mut state = self.0.lock();
        if !state.trace {
            return;
        }
        let text = state.sanitizer.sanitize(&text());
        state.push("cache-adoption", text);
    }
    pub(crate) fn record(&self, source: &str, text: &str) {
        let mut state = self.0.lock();
        let text = state.sanitizer.sanitize(text);
        if source == "failure" {
            state.failure = Some(text.clone());
        }
        let source = match (state.readiness_confirmed, source) {
            (false, "stdout") => "startup-probe-stdout",
            (false, "stderr") => "startup-probe-stderr",
            _ => source,
        };
        state.push(source, text);
    }
    pub(crate) fn snapshot(&self) -> EngineDiagnosticSnapshotDto {
        let mut metrics = metrics::collect();
        let state = self.0.lock();
        let time = now_ms();
        let metric = |name: &str, unit: &str, value, missing: Option<&str>| EngineDiagnosticMetricDto {
            role: "engine-attempt".into(),
            name: name.into(),
            unit: unit.into(),
            at_ms: time,
            value,
            missing: missing.map(str::to_owned),
        };
        metrics.push(metric(
            "elapsed",
            "ms",
            Some(state.started.elapsed().as_millis() as f64),
            None,
        ));
        metrics.push(metric(
            "stack",
            "frames",
            None,
            Some("stack walking is not enabled"),
        ));
        EngineDiagnosticSnapshotDto {
            attempt_id: state.run_id.clone(),
            run_id: state.run_id.clone(),
            profile_id: state.profile_id.clone(),
            captured_at_ms: time,
            full_trace: state.trace,
            records: state.records.iter().cloned().collect(),
            dropped_records: state.dropped,
            retained_bytes: state.bytes,
            command: state.command.clone(),
            failure: state.failure.clone(),
            stdout_complete: state.stdout_complete,
            stderr_complete: state.stderr_complete,
            process_exited: state.process_exited,
            exit_code: state.exit_code,
            metrics,
        }
    }
}
impl Capture {
    fn push(&mut self, source: &str, text: String) {
        while self.records.len() >= CAPTURE_RECORDS || self.bytes + text.len() > CAPTURE_BYTES {
            if let Some(record) = self.records.pop_front() {
                self.bytes -= record.text.len();
                self.dropped += 1;
            } else {
                break;
            }
        }
        self.sequence += 1;
        self.bytes += text.len();
        self.records.push_back(EngineDiagnosticRecordDto {
            sequence: self.sequence,
            at_ms: now_ms(),
            source: source.into(),
            text,
        });
    }
}

/// Observation only: return exactly the bytes read, even if diagnostic retention
/// is exhausted. Startup and live streams keep their original attempt handle.
pub(crate) struct ObservedRead<R> {
    reader: R,
    capture: AttemptCapture,
    source: &'static str,
    pending: Vec<u8>,
    overflow: bool,
}
impl<R> ObservedRead<R> {
    pub(crate) fn new(reader: R, capture: AttemptCapture, source: &'static str) -> Self {
        Self {
            reader,
            capture,
            source,
            pending: Vec::new(),
            overflow: false,
        }
    }
}
impl<R: Read> Read for ObservedRead<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let length = self.reader.read(output)?;
        for byte in &output[..length] {
            if *byte == b'\n' {
                if self.overflow {
                    self.capture.record(self.source, "[overlong record omitted]");
                } else {
                    self.capture
                        .record(self.source, &String::from_utf8_lossy(&self.pending));
                }
                self.pending.clear();
                self.overflow = false;
            } else if self.pending.len() < RECORD_BYTES {
                self.pending.push(*byte);
            } else {
                self.overflow = true;
            }
        }
        if length == 0 && !self.pending.is_empty() {
            if self.overflow {
                self.capture.record(self.source, "[overlong record omitted]");
            } else {
                self.capture
                    .record(self.source, &String::from_utf8_lossy(&self.pending));
            }
            self.pending.clear();
        }
        if length == 0 {
            self.capture.source_finished(self.source);
        }
        Ok(length)
    }
}

#[cfg(test)]
mod failure_text_tests {
    use super::*;

    #[test]
    fn trusted_guidance_survives_alias_collision_and_dynamic_fragments_remain_private() {
        let mut sanitizer = DiagnosticSanitizer::default();
        let analysis = sanitizer.alias("analysis");
        let secret = sanitizer.alias("fixture-private-value");
        let text = FailureText::dynamic(&mut sanitizer, "analysis fixture-private-value")
            .prefixed("required analysis setting was ignored: ")
            .into_string();
        assert!(text.starts_with("required analysis setting was ignored: "));
        assert!(text.contains(&analysis) && text.contains(&secret));
        assert!(!text.contains("fixture-private-value"));
        assert_ne!(analysis, secret);
        let bounded = FailureText::from("analysis response: ")
            .then(FailureText::dynamic(&mut sanitizer, &"界".repeat(RECORD_BYTES)))
            .then("; retry Stop")
            .into_string();
        assert!(bounded.len() <= RECORD_BYTES);
        assert!(bounded.starts_with("analysis response: "));
    }
}
