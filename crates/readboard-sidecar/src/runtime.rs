use crate::frame::{FrameDecoder, ReadboardInbound};
use crate::protocol::{command, Handshake};
use app_model::{ReadboardPhaseDto as Phase, ReadboardRuntimeDto};
use std::{
    io::{self, Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Child,
    sync::{mpsc, Arc, Condvar, Mutex},
    thread,
    time::{Duration, Instant},
};

const STARTUP_BUDGET: Duration = Duration::from_secs(10);
const POLL: Duration = Duration::from_millis(10);
const QUIT_GRACE: Duration = Duration::from_millis(500);
const MAX_LINE: usize = 8192;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReadboardInboundEvent {
    pub generation: u64,
    pub inbound: ReadboardInbound,
}

/// One subscriber stream in publication order: a generation's Ready snapshot precedes its
/// inbound lines, and no inbound line of a generation follows the snapshot that sealed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadboardRuntimeEvent {
    Lifecycle(ReadboardRuntimeDto),
    Inbound(ReadboardInboundEvent),
}

/// One owned process/socket, independent of the current game and engine owners.
/// Start returns immediately. Stop seals publication before waiting for actual reaping.
#[derive(Clone)]
pub struct ReadboardRuntime {
    owner: Arc<Owner>,
}

struct Owner {
    inner: Arc<Inner>,
}

struct Inner {
    state: Mutex<State>,
    changed: Condvar,
    supported: bool,
    startup_budget: Duration,
}

struct State {
    snapshot: ReadboardRuntimeDto,
    // Resource identity survives a publication-generation change until confirmed reaping.
    owned_generation: Option<u64>,
    cancelled: bool,
    exiting: bool,
    subscribers: Vec<mpsc::Sender<ReadboardRuntimeEvent>>,
    focus_requests: Vec<u64>,
}

impl Default for ReadboardRuntime {
    fn default() -> Self {
        Self::new(cfg!(windows), STARTUP_BUDGET)
    }
}

impl ReadboardRuntime {
    fn new(supported: bool, startup_budget: Duration) -> Self {
        Self {
            owner: Arc::new(Owner {
                inner: Arc::new(Inner {
                    state: Mutex::new(State {
                        snapshot: ReadboardRuntimeDto {
                            generation: 0,
                            revision: 0,
                            phase: if supported {
                                Phase::Idle
                            } else {
                                Phase::Unavailable
                            },
                            executable_path: None,
                            process_id: None,
                            endpoint: None,
                            wire_version: None,
                            resources_held: false,
                            message: if supported {
                                "Select and save the Windows readboard executable, then Start."
                            } else {
                                "The admitted readboard runtime requires Windows."
                            }
                            .into(),
                        },
                        owned_generation: None,
                        cancelled: false,
                        exiting: false,
                        subscribers: Vec::new(),
                        focus_requests: Vec::new(),
                    }),
                    changed: Condvar::new(),
                    supported,
                    startup_budget,
                }),
            }),
        }
    }

    pub fn snapshot(&self) -> ReadboardRuntimeDto {
        self.owner
            .inner
            .state
            .lock()
            .expect("readboard state")
            .snapshot
            .clone()
    }

    pub fn subscribe(&self) -> mpsc::Receiver<ReadboardRuntimeEvent> {
        let (tx, rx) = mpsc::channel();
        let mut state = self.owner.inner.state.lock().expect("readboard state");
        let _ = tx.send(ReadboardRuntimeEvent::Lifecycle(state.snapshot.clone()));
        state.subscribers.push(tx);
        rx
    }

    /// Writes `loss\n` on the owned stream of `generation` if it is the current Ready connection.
    pub fn request_focus(&self, generation: u64) -> Result<(), String> {
        let inner = &self.owner.inner;
        let mut state = inner.state.lock().expect("readboard state");
        if state.exiting {
            return Err("Application teardown has sealed readboard.".into());
        }
        if state.cancelled || state.snapshot.generation != generation {
            return Err(format!(
                "Readboard focus request for generation {generation} is stale (current: {}).",
                state.snapshot.generation
            ));
        }
        if state.snapshot.phase != Phase::Ready {
            return Err(format!(
                "Readboard is not ready for focus request (current phase: {:?}).",
                state.snapshot.phase
            ));
        }
        state.focus_requests.push(generation);
        inner.changed.notify_all();
        Ok(())
    }

    pub fn start(&self, path: Option<&str>) -> Result<ReadboardRuntimeDto, String> {
        self.start_if_current(path, None)
    }

    fn start_if_current(
        &self,
        path: Option<&str>,
        expected: Option<u64>,
    ) -> Result<ReadboardRuntimeDto, String> {
        let inner = &self.owner.inner;
        let mut state = inner.state.lock().expect("readboard state");
        if state.exiting {
            return Err("Application teardown has sealed readboard startup.".into());
        }
        if expected.is_some_and(|generation| generation != state.snapshot.generation) {
            return Err("Readboard Restart was superseded by another operation.".into());
        }
        if state.owned_generation.is_some() {
            return Err("Stop and reap the owned readboard process before starting another.".into());
        }
        state.snapshot.generation += 1;
        state.snapshot.executable_path = path.map(str::to_owned);
        state.snapshot.process_id = None;
        state.snapshot.endpoint = None;
        state.snapshot.wire_version = None;
        let target = if inner.supported {
            validated_path(path)
        } else {
            Err("The admitted readboard runtime requires Windows.".into())
        };
        let target = match target {
            Ok(target) => target,
            Err(error) => {
                state.snapshot.phase = Phase::Unavailable;
                state.snapshot.message = error;
                publish(inner, &mut state);
                return Ok(state.snapshot.clone());
            }
        };
        let generation = state.snapshot.generation;
        state.cancelled = false;
        state.owned_generation = Some(generation);
        state.snapshot.resources_held = true;
        state.snapshot.phase = Phase::Starting;
        state.snapshot.message = "Waiting for readboard ready and wire version 220430.".into();
        state.focus_requests.clear();
        publish(inner, &mut state);
        let worker = Arc::clone(inner);
        if let Err(error) = thread::Builder::new()
            .name("readboard-runtime".into())
            .spawn(move || run(worker, generation, target))
        {
            state.owned_generation = None;
            state.snapshot.resources_held = false;
            state.snapshot.phase = Phase::Unavailable;
            state.snapshot.message = format!("Could not start readboard worker: {error}");
            publish(inner, &mut state);
        }
        Ok(state.snapshot.clone())
    }

    pub fn stop(&self, budget: Duration) -> ReadboardRuntimeDto {
        self.stop_owned(budget, false).0
    }

    pub fn restart(&self, path: Option<&str>, budget: Duration) -> Result<ReadboardRuntimeDto, String> {
        let (stopped, operation) = self.stop_owned(budget, false);
        if stopped.resources_held {
            return Ok(stopped);
        }
        self.start_if_current(path, Some(operation))
    }

    pub fn begin_teardown(&self) {
        let inner = &self.owner.inner;
        let mut state = inner.state.lock().expect("readboard state");
        state.exiting = true;
        seal(inner, &mut state);
    }

    /// Permanently closes startup admission and shares the caller's remaining exit budget.
    pub fn teardown(&self, budget: Duration) -> Vec<String> {
        let (snapshot, _) = self.stop_owned(budget, true);
        if snapshot.resources_held {
            vec![format!("readboard owned process/socket: {}", snapshot.message)]
        } else {
            Vec::new()
        }
    }

    fn stop_owned(&self, budget: Duration, exiting: bool) -> (ReadboardRuntimeDto, u64) {
        let inner = &self.owner.inner;
        let deadline = Instant::now() + budget;
        let mut state = inner.state.lock().expect("readboard state");
        state.exiting |= exiting;
        seal(inner, &mut state);
        let operation = state.snapshot.generation;
        while state.owned_generation.is_some() {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                state.snapshot.phase = Phase::CleanupFailed;
                state.snapshot.message =
                    "Readboard cleanup is not yet confirmed; Retry Stop or application teardown.".into();
                publish(inner, &mut state);
                break;
            }
            state = inner
                .changed
                .wait_timeout(state, remaining)
                .expect("readboard cleanup wait")
                .0;
        }
        (state.snapshot.clone(), operation)
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        let mut state = self.inner.state.lock().expect("readboard state");
        state.exiting = true;
        seal(&self.inner, &mut state);
    }
}

fn validated_path(path: Option<&str>) -> Result<PathBuf, String> {
    let path = path
        .filter(|path| !path.trim().is_empty())
        .ok_or("Select and save a readboard EXE first.")?;
    let path = Path::new(path);
    if !path.is_absolute()
        || !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
    {
        return Err("Readboard requires an absolute native .exe path, not a script or JAR.".into());
    }
    let path = path
        .canonicalize()
        .map_err(|error| format!("Readboard executable is unavailable: {error}"))?;
    if !path.is_file() {
        return Err("Readboard executable is not a file.".into());
    }
    Ok(path)
}

fn publish(inner: &Inner, state: &mut State) {
    state.snapshot.revision += 1;
    let event = ReadboardRuntimeEvent::Lifecycle(state.snapshot.clone());
    state.subscribers.retain(|tx| tx.send(event.clone()).is_ok());
    inner.changed.notify_all();
}

fn seal(inner: &Inner, state: &mut State) {
    state.snapshot.generation += 1;
    state.cancelled = true;
    state.focus_requests.clear();
    state.snapshot.phase = if state.owned_generation.is_some() {
        Phase::Stopping
    } else {
        Phase::Stopped
    };
    state.snapshot.message = "Readboard stopped; current game is unchanged.".into();
    publish(inner, state);
}

fn cancelled(inner: &Inner, generation: u64) -> bool {
    let state = inner.state.lock().expect("readboard state");
    state.cancelled || state.snapshot.generation != generation
}

fn update(inner: &Inner, generation: u64, change: impl FnOnce(&mut ReadboardRuntimeDto)) {
    let mut state = inner.state.lock().expect("readboard state");
    if state.snapshot.generation == generation && !state.cancelled {
        change(&mut state.snapshot);
        publish(inner, &mut state);
    }
}
fn publish_inbound(inner: &Inner, generation: u64, inbound: ReadboardInbound) {
    let mut state = inner.state.lock().expect("readboard state");
    if state.snapshot.generation == generation && !state.cancelled {
        let event = ReadboardRuntimeEvent::Inbound(ReadboardInboundEvent { generation, inbound });
        state.subscribers.retain(|tx| tx.send(event.clone()).is_ok());
    }
}

fn pause(inner: &Inner) {
    let state = inner.state.lock().expect("readboard state");
    drop(inner.changed.wait_timeout(state, POLL).expect("readboard poll"));
}

fn run(inner: Arc<Inner>, generation: u64, path: PathBuf) {
    let deadline = Instant::now() + inner.startup_budget;
    let mut child = None;
    let mut stream = None;
    let outcome = serve(&inner, generation, &path, deadline, &mut child, &mut stream);
    if let Err((phase, message)) = outcome {
        update(&inner, generation, |snapshot| {
            snapshot.phase = phase;
            snapshot.message = message;
        });
    }
    // No new publication can escape the generation gate while resources drain.
    if let Some(mut stream) = stream.take() {
        let _ = stream.write_all(b"quit\n");
        let _ = stream.shutdown(Shutdown::Both);
    }
    if let Some(mut child) = child {
        let grace = Instant::now() + QUIT_GRACE;
        let mut kill_sent = false;
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) if Instant::now() >= grace && !kill_sent => match child.kill() {
                    Ok(()) => kill_sent = true,
                    Err(error) => cleanup_error(&inner, generation, &error),
                },
                Err(error) => cleanup_error(&inner, generation, &error),
                _ => {}
            }
            // Retain the Child if the OS cannot confirm exit; Retry/Exit anyway remains honest.
            pause(&inner);
        }
    }
    let mut state = inner.state.lock().expect("readboard state");
    if state.owned_generation == Some(generation) {
        state.owned_generation = None;
        state.snapshot.resources_held = false;
        state.snapshot.process_id = None;
        state.snapshot.endpoint = None;
        if state.cancelled {
            state.snapshot.phase = Phase::Stopped;
            state.snapshot.message =
                "Readboard process reaped and socket closed; current game is unchanged.".into();
        }
        publish(&inner, &mut state);
    }
}

fn cleanup_error(inner: &Inner, generation: u64, error: &io::Error) {
    let mut state = inner.state.lock().expect("readboard state");
    if state.owned_generation == Some(generation) && state.snapshot.phase != Phase::CleanupFailed {
        state.snapshot.phase = Phase::CleanupFailed;
        state.snapshot.message = format!("Could not confirm owned readboard cleanup: {error}");
        publish(inner, &mut state);
    }
}

type RuntimeResult = Result<(), (Phase, String)>;

fn serve(
    inner: &Inner,
    generation: u64,
    path: &Path,
    deadline: Instant,
    child: &mut Option<Child>,
    stream: &mut Option<TcpStream>,
) -> RuntimeResult {
    let unavailable = |error| (Phase::Unavailable, format!("Readboard startup failed: {error}"));
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(unavailable)?;
    listener.set_nonblocking(true).map_err(unavailable)?;
    let address = listener.local_addr().map_err(unavailable)?;
    if cancelled(inner, generation) {
        return Ok(());
    }
    *child = Some(command(path, address.port()).spawn().map_err(unavailable)?);
    let process = child.as_mut().expect("owned readboard child");
    update(inner, generation, |snapshot| {
        snapshot.process_id = Some(process.id());
        snapshot.endpoint = Some(address.to_string());
    });
    loop {
        if cancelled(inner, generation) {
            return Ok(());
        }
        check_child(process)?;
        if Instant::now() >= deadline {
            return Err((
                Phase::Timeout,
                "Readboard did not connect and complete ready/version before the startup deadline.".into(),
            ));
        }
        match listener.accept() {
            Ok((socket, _)) => {
                socket.set_nonblocking(true).map_err(unavailable)?;
                *stream = Some(socket);
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => pause(inner),
            Err(error) => return Err(unavailable(error)),
        }
    }
    // Only the accepted stream is needed; do not leave a listener open after admission.
    drop(listener);
    let stream = stream.as_mut().expect("owned readboard stream");
    let mut handshake = Handshake::default();
    let mut decoder = FrameDecoder::default();
    let mut pending = Vec::new();
    let mut bytes = [0u8; 4096];
    loop {
        if cancelled(inner, generation) {
            return Ok(());
        }
        check_child(process)?;
        if !handshake.is_ready() && Instant::now() >= deadline {
            return Err((
                Phase::Timeout,
                "Readboard connected but did not complete ready + requested wire version 220430.".into(),
            ));
        }
        if handshake.is_ready() {
            let focus_count = {
                let mut state = inner.state.lock().expect("readboard state");
                if state.snapshot.generation == generation && !state.cancelled {
                    let count = state.focus_requests.len();
                    state.focus_requests.clear();
                    count
                } else {
                    state.focus_requests.clear();
                    0
                }
            };
            for _ in 0..focus_count {
                stream.write_all(b"loss\n").map_err(|error| {
                    (
                        Phase::Disconnected,
                        format!("Could not send readboard focus request: {error}"),
                    )
                })?;
            }
        }
        match stream.read(&mut bytes) {
            Ok(0) => {
                return Err((
                    Phase::Disconnected,
                    "Readboard closed its protocol connection.".into(),
                ))
            }
            Ok(count) => {
                for byte in &bytes[..count] {
                    if *byte != b'\n' {
                        if pending.len() == MAX_LINE {
                            return Err((
                                Phase::Incompatible,
                                "Readboard protocol line exceeded its bounded size.".into(),
                            ));
                        }
                        pending.push(*byte);
                        continue;
                    }
                    let line = std::str::from_utf8(&pending)
                        .map_err(|_| (Phase::Incompatible, "Readboard protocol is not UTF-8.".into()))?
                        .trim_end_matches('\r');
                    let was_ready = handshake.is_ready();
                    let action = handshake.receive(line);
                    if line.starts_with("version") {
                        if let Some(version) = handshake.wire_version() {
                            update(inner, generation, |snapshot| {
                                snapshot.wire_version = Some(version.into())
                            });
                        }
                    }
                    match action {
                        Ok(Some(request)) => stream.write_all(request).map_err(|error| {
                            (
                                Phase::Disconnected,
                                format!("Could not request readboard version: {error}"),
                            )
                        })?,
                        Ok(None) => {}
                        Err(error) => return Err((Phase::Incompatible, error)),
                    }
                    if !was_ready && handshake.is_ready() {
                        update(inner, generation, |snapshot| {
                            snapshot.phase = Phase::Ready;
                            snapshot.message =
                                "Readboard ready (wire 220430). Lifecycle only; current game is unchanged."
                                    .into();
                        });
                    }
                    if was_ready {
                        if let Some(inbound) = decoder.receive(line) {
                            publish_inbound(inner, generation, inbound);
                        }
                    }
                    pending.clear();
                }
            }
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => pause(inner),
            Err(error) => {
                return Err((
                    Phase::Disconnected,
                    format!("Readboard connection failed: {error}"),
                ))
            }
        }
    }
}

fn check_child(child: &mut Child) -> RuntimeResult {
    match child.try_wait() {
        Ok(Some(status)) => Err((Phase::Exited, format!("Readboard process exited: {status}"))),
        Ok(None) => Ok(()),
        Err(error) => Err((
            Phase::Exited,
            format!("Could not observe readboard process: {error}"),
        )),
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
