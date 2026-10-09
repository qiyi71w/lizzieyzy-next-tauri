//! The one Run reader routes a complete response to its registered request.
//! Registration precedes writing; retirement never gives a late response a new owner.
use std::collections::HashMap;
use std::sync::mpsc::{self, Receiver, SyncSender};

const MAX_PENDING: usize = 16;
const QUEUE_LINES: usize = 64;

struct Pending {
    request: String,
    stream: bool,
    sender: SyncSender<String>,
}

#[derive(Default)]
pub(super) struct Dispatch {
    pending: HashMap<(String, u32), Pending>,
    active: HashMap<String, u32>,
    issued_through: HashMap<String, u32>,
}

impl Dispatch {
    pub(super) fn register(
        &mut self,
        run: &str,
        id: u32,
        request: &str,
        stream: bool,
    ) -> Result<Receiver<String>, String> {
        if self.pending.len() >= MAX_PENDING {
            return Err("GTP pending command limit reached".into());
        }
        let key = (run.to_owned(), id);
        if self.pending.contains_key(&key) {
            return Err("GTP command ID was already registered".into());
        }
        let (sender, receiver) = mpsc::sync_channel(QUEUE_LINES);
        self.pending.insert(
            key,
            Pending {
                request: request.into(),
                stream,
                sender,
            },
        );
        self.issued_through.entry(run.into()).and_modify(|last| *last = (*last).max(id)).or_insert(id);
        Ok(receiver)
    }

    pub(super) fn retire(&mut self, run: &str, request: &str) {
        self.pending
            .retain(|(owner, _), pending| owner != run || pending.request != request);
        // Keep the framing cursor until its delimiter: its remaining lines must be discarded.
    }

    pub(super) fn retire_run(&mut self, run: &str) {
        self.pending.retain(|(owner, _), _| owner != run);
        self.active.remove(run);
        self.issued_through.remove(run);
    }

    pub(super) fn route(&mut self, run: &str, line: &str) -> Result<(), String> {
        if line.starts_with(['=', '?']) {
            if self.active.contains_key(run) {
                return Err("GTP response header arrived before the previous delimiter".into());
            }
            let digits = line[1..].bytes().take_while(u8::is_ascii_digit).count();
            let id = line[1..1 + digits]
                .parse::<u32>()
                .map_err(|_| "GTP response has no valid numbered identity")?;
            if !self.pending.keys().any(|(owner, _)| owner == run)
                && self.issued_through.get(run).is_none_or(|last| id > *last) {
                return Err("unsolicited GTP response on an idle reader".into());
            }
            self.active.insert(run.into(), id);
        }
        let Some(id) = self.active.get(run).copied() else {
            // An unnumbered record outside a registered response has no publication authority.
            if line.is_empty() || crate::gtp::is_analysis_stream_record(line) {
                return Ok(());
            }
            return Err("GTP output outside a numbered response".into());
        };
        let key = (run.to_owned(), id);
        if let Some(pending) = self.pending.get(&key) {
            if pending.stream || !crate::gtp::is_analysis_stream_record(line) {
                pending
                    .sender
                    .try_send(line.to_owned())
                    .map_err(|_| "GTP request exceeded its bounded response queue")?;
            }
        }
        if line.is_empty() {
            self.active.remove(run);
            self.pending.remove(&key);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn late_retired_reply_on_idle_reader_is_discarded_without_failing_run() {
        let mut dispatch = Dispatch::default();
        let old = dispatch.register("run", 101, "timed-out", false).unwrap();
        dispatch.retire("run", "timed-out");
        for line in ["=101 2", ""] { dispatch.route("run", line).unwrap(); }
        assert!(old.try_recv().is_err());
        assert!(dispatch.route("run", "=102 unsolicited").is_err());
    }

    #[test]
    fn request_owns_multiline_ack_and_stream_cannot_complete_control() {
        let mut dispatch = Dispatch::default();
        let a = dispatch.register("run", 101, "rules", false).unwrap();
        let b = dispatch.register("run", 102, "threads", false).unwrap();
        for line in ["=101", "info move D4 visits 1", "play D4", "rules-body", ""] {
            dispatch.route("run", line).unwrap();
        }
        assert_eq!(a.try_iter().collect::<Vec<_>>(), ["=101", "rules-body", ""]);
        assert!(b.try_recv().is_err());
        for line in ["=102", "thread-body", ""] {
            dispatch.route("run", line).unwrap();
        }
        assert_eq!(b.try_iter().collect::<Vec<_>>(), ["=102", "thread-body", ""]);
    }

    #[test]
    fn retired_and_other_reader_replies_cannot_complete_current_request() {
        let mut dispatch = Dispatch::default();
        let retired = dispatch.register("old-run", 101, "old", false).unwrap();
        dispatch.retire_run("old-run");
        let current = dispatch.register("new-run", 102, "new", false).unwrap();
        for line in ["=101", "old-body", ""] {
            let _ = dispatch.route("old-run", line);
        }
        for line in ["=101", "unknown-body", ""] {
            dispatch.route("new-run", line).unwrap();
        }
        assert!(retired.try_recv().is_err());
        assert!(current.try_recv().is_err());
        for line in ["=102", "new-body", ""] {
            dispatch.route("new-run", line).unwrap();
        }
        assert_eq!(current.try_iter().collect::<Vec<_>>(), ["=102", "new-body", ""]);
    }

    #[test]
    fn stream_delimiter_precedes_independent_stop_ack() {
        let mut dispatch = Dispatch::default();
        let stream = dispatch.register("run", 101, "analysis", true).unwrap();
        let stop = dispatch.register("run", 102, "analysis", false).unwrap();
        for line in ["=101", "info move D4 visits 1"] {
            dispatch.route("run", line).unwrap();
        }
        assert!(stop.try_recv().is_err());
        dispatch.route("run", "").unwrap();
        assert_eq!(
            stream.try_iter().collect::<Vec<_>>(),
            ["=101", "info move D4 visits 1", ""]
        );
        assert!(stop.try_recv().is_err());
        for line in ["=102", ""] {
            dispatch.route("run", line).unwrap();
        }
        assert_eq!(stop.try_iter().collect::<Vec<_>>(), ["=102", ""]);
    }

    #[test]
    fn duplicate_registration_does_not_replace_owner_and_retirement_preserves_framing() {
        let mut dispatch = Dispatch::default();
        let first = dispatch.register("run", 101, "first", true).unwrap();
        assert!(dispatch.register("run", 101, "second", false).is_err());
        dispatch.route("run", "=101").unwrap();
        assert_eq!(first.try_recv().unwrap(), "=101");
        dispatch.retire("run", "first");
        dispatch.route("run", "info old").unwrap();
        assert!(dispatch.route("run", "=102").is_err());
        dispatch.route("run", "").unwrap();
        assert!(first.try_recv().is_err());
    }
}
