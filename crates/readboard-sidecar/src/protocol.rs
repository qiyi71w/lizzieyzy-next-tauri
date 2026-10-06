use std::path::Path;
use std::process::{Command, Stdio};

pub(crate) const ADMITTED_WIRE_VERSION: &str = "220430";
const VERSION_COMMAND: &[u8] = b"version\n";

/// Builds the `std::process::Command` to start the native readboard executable.
///
/// Pinned runtime cdcc7b3 `LaunchOptions.cs` requires at least seven positional argv items:
/// `['yzy', ' ', ' ', ' ', '1', 'cn', portString]`
/// - argv[0]: "yzy"
/// - argv[1..3]: " ", " ", " " (analysis budgets: AiTime, Playouts, FirstPolicy)
/// - argv[4]: "1" (TCP transport)
/// - argv[5]: "cn" (language)
/// - argv[6]: portString (TCP port)
///
/// Current directory is set to `path.parent()`. Stdin, stdout, and stderr are set to null.
pub(crate) fn command(path: &Path, port: u16) -> Command {
    let mut cmd = Command::new(path);
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        cmd.current_dir(parent);
    }
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.args(["yzy", " ", " ", " ", "1", "cn", &port.to_string()]);
    cmd
}

/// Inbound startup protocol state machine for the readboard sidecar.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub(crate) struct Handshake {
    ready_received: bool,
    is_ready: bool,
    wire_version: Option<String>,
}

impl Handshake {
    /// Whether the sidecar has completed the handshake sequence and is admitted as ready.
    pub(crate) fn is_ready(&self) -> bool {
        self.is_ready
    }

    /// The wire version observed from the sidecar (admitted or incompatible).
    pub(crate) fn wire_version(&self) -> Option<&str> {
        self.wire_version.as_deref()
    }

    /// Process a single complete decoded protocol line (without LF/CR).
    ///
    /// - First exact `ready` line returns `Some(b"version\n")` and marks the version request outstanding.
    /// - Duplicate `ready` lines are ignored (`Ok(None)`).
    /// - Version lines arriving before `ready` cannot grant readiness or be cached; they are ignored.
    /// - Once `ready` has been received, version responses are checked:
    ///   - Exact admitted version (`version: 220430`) grants readiness (`is_ready() == true`).
    ///   - Other versions (greater future versions, older versions, malformed/empty) retain the observed
    ///     version for presentation and return `Err` explaining incompatibility.
    /// - Once ready, duplicate admitted versions or duplicate `ready` lines do not regress readiness.
    /// - Once ready, unsupported later version values fail.
    /// - Unrelated startup state lines (e.g. `playponder`, `bothSync`, `syncPlatform`) are ignored.
    pub(crate) fn receive(&mut self, line: &str) -> Result<Option<&'static [u8]>, String> {
        if line == "ready" {
            if !self.ready_received {
                self.ready_received = true;
                return Ok(Some(VERSION_COMMAND));
            }
            return Ok(None);
        }

        if let Some(observed) = parse_version_response(line) {
            if !self.ready_received {
                return Ok(None);
            }

            if observed == ADMITTED_WIRE_VERSION {
                self.is_ready = true;
                self.wire_version = Some(ADMITTED_WIRE_VERSION.to_string());
                return Ok(None);
            }

            self.wire_version = Some(observed.to_string());
            self.is_ready = false;
            let msg = if observed.is_empty() {
                format!(
                    "incompatible readboard wire version: missing or empty version, expected {ADMITTED_WIRE_VERSION}"
                )
            } else {
                format!(
                    "incompatible readboard wire version: expected {ADMITTED_WIRE_VERSION}, observed '{observed}'"
                )
            };
            return Err(msg);
        }

        Ok(None)
    }
}

fn parse_version_response(line: &str) -> Option<&str> {
    if let Some(rest) = line.strip_prefix("version: ") {
        Some(rest.trim())
    } else if let Some(rest) = line.strip_prefix("version:") {
        Some(rest.trim())
    } else if let Some(rest) = line.strip_prefix("version ") {
        Some(rest.trim())
    } else if line == "version" {
        Some("")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handshake_ready_then_version_sequence_succeeds() {
        let mut hs = Handshake::default();
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), None);

        let out = hs.receive("ready").expect("receive ready should succeed");
        assert_eq!(out, Some(b"version\n".as_slice()));
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), None);

        let out = hs
            .receive("version: 220430")
            .expect("receive version should succeed");
        assert_eq!(out, None);
        assert!(hs.is_ready());
        assert_eq!(hs.wire_version(), Some("220430"));
    }

    #[test]
    fn handshake_unrelated_startup_state_lines_ignored() {
        let mut hs = Handshake::default();
        let out = hs.receive("ready").expect("receive ready should succeed");
        assert_eq!(out, Some(b"version\n".as_slice()));

        assert_eq!(hs.receive("playponder on").unwrap(), None);
        assert_eq!(hs.receive("bothSync").unwrap(), None);
        assert_eq!(hs.receive("syncPlatform generic").unwrap(), None);
        assert_eq!(hs.receive("roomToken 123456").unwrap(), None);
        assert!(!hs.is_ready());

        assert_eq!(hs.receive("version: 220430").unwrap(), None);
        assert!(hs.is_ready());

        assert_eq!(hs.receive("recordCurrentMove 0").unwrap(), None);
        assert!(hs.is_ready());
    }

    #[test]
    fn handshake_early_version_cannot_grant_readiness_or_be_cached() {
        let mut hs = Handshake::default();

        assert_eq!(hs.receive("version: 220430").unwrap(), None);
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), None);

        assert_eq!(hs.receive("version: 999999").unwrap(), None);
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), None);

        let out = hs.receive("ready").unwrap();
        assert_eq!(out, Some(b"version\n".as_slice()));
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), None);

        assert_eq!(hs.receive("version: 220430").unwrap(), None);
        assert!(hs.is_ready());
        assert_eq!(hs.wire_version(), Some("220430"));
    }

    #[test]
    fn handshake_duplicate_ready_ignored() {
        let mut hs = Handshake::default();
        let out = hs.receive("ready").unwrap();
        assert_eq!(out, Some(b"version\n".as_slice()));

        assert_eq!(hs.receive("ready").unwrap(), None);
        assert!(!hs.is_ready());

        assert_eq!(hs.receive("version: 220430").unwrap(), None);
        assert!(hs.is_ready());

        assert_eq!(hs.receive("ready").unwrap(), None);
        assert!(hs.is_ready());
        assert_eq!(hs.wire_version(), Some("220430"));
    }

    #[test]
    fn handshake_incompatible_greater_future_version_fails() {
        let mut hs = Handshake::default();
        assert_eq!(hs.receive("ready").unwrap(), Some(b"version\n".as_slice()));

        let err = hs.receive("version: 220431").unwrap_err();
        assert!(err.contains("incompatible"));
        assert!(err.contains("220431"));
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), Some("220431"));
    }

    #[test]
    fn handshake_incompatible_older_or_unknown_version_fails() {
        let mut hs = Handshake::default();
        assert_eq!(hs.receive("ready").unwrap(), Some(b"version\n".as_slice()));

        let err = hs.receive("version: 210101").unwrap_err();
        assert!(err.contains("incompatible"));
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), Some("210101"));

        let mut hs2 = Handshake::default();
        assert_eq!(hs2.receive("ready").unwrap(), Some(b"version\n".as_slice()));
        let err2 = hs2.receive("version: unknown-future").unwrap_err();
        assert!(err2.contains("incompatible"));
        assert_eq!(hs2.wire_version(), Some("unknown-future"));
    }

    #[test]
    fn handshake_malformed_and_empty_version_fails() {
        let mut hs = Handshake::default();
        assert_eq!(hs.receive("ready").unwrap(), Some(b"version\n".as_slice()));
        let err = hs.receive("version: ").unwrap_err();
        assert!(err.contains("incompatible"));
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), Some(""));

        let mut hs2 = Handshake::default();
        assert_eq!(hs2.receive("ready").unwrap(), Some(b"version\n".as_slice()));
        let err2 = hs2.receive("version:").unwrap_err();
        assert!(err2.contains("incompatible"));
        assert!(!hs2.is_ready());
        assert_eq!(hs2.wire_version(), Some(""));

        let mut hs3 = Handshake::default();
        assert_eq!(hs3.receive("ready").unwrap(), Some(b"version\n".as_slice()));
        let err3 = hs3.receive("version").unwrap_err();
        assert!(err3.contains("incompatible"));
        assert!(!hs3.is_ready());
        assert_eq!(hs3.wire_version(), Some(""));
    }

    #[test]
    fn handshake_once_ready_duplicate_admitted_does_not_regress() {
        let mut hs = Handshake::default();
        assert_eq!(hs.receive("ready").unwrap(), Some(b"version\n".as_slice()));
        assert_eq!(hs.receive("version: 220430").unwrap(), None);
        assert!(hs.is_ready());

        assert_eq!(hs.receive("version: 220430").unwrap(), None);
        assert!(hs.is_ready());
        assert_eq!(hs.wire_version(), Some("220430"));
    }

    #[test]
    fn handshake_once_ready_unsupported_later_version_fails() {
        let mut hs = Handshake::default();
        assert_eq!(hs.receive("ready").unwrap(), Some(b"version\n".as_slice()));
        assert_eq!(hs.receive("version: 220430").unwrap(), None);
        assert!(hs.is_ready());

        let err = hs.receive("version: 230000").unwrap_err();
        assert!(err.contains("incompatible"));
        assert!(!hs.is_ready());
        assert_eq!(hs.wire_version(), Some("230000"));
    }
}
