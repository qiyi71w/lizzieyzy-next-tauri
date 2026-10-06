use super::*;
use go_core::{ReadboardControl, ReadboardPlatform};
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture {
    root: PathBuf,
    path: PathBuf,
}

impl Fixture {
    fn new(mode: &str) -> Self {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let root = std::env::temp_dir().join(format!("next-readboard-{}-{unique}", std::process::id()));
        fs::create_dir(&root).unwrap();
        let path = root.join(format!("{mode}.exe"));
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = Command::new(compiler)
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/controlled_readboard.rs"))
            .args(["--edition=2021", "-o"])
            .arg(&path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self { root, path }
    }
    fn start(&self, runtime: &ReadboardRuntime) -> ReadboardRuntimeDto {
        runtime.start(self.path.to_str()).unwrap()
    }
    fn pid(&self) -> u32 {
        fs::read_to_string(self.path.with_extension("pid"))
            .unwrap()
            .parse()
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn controlled() -> ReadboardRuntime {
    ReadboardRuntime::new(true, Duration::from_millis(500))
}

fn wait_for(
    runtime: &ReadboardRuntime,
    condition: impl Fn(&ReadboardRuntimeDto) -> bool,
) -> ReadboardRuntimeDto {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = runtime.snapshot();
        if condition(&snapshot) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "readboard did not reach expected state: {snapshot:?}"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

/// The ordered runtime stream; every inbound line must arrive while its own generation is the
/// latest Ready snapshot already delivered on the same stream.
struct Events {
    rx: mpsc::Receiver<ReadboardRuntimeEvent>,
    open: Option<u64>,
}

impl Events {
    fn new(runtime: &ReadboardRuntime) -> Self {
        Self {
            rx: runtime.subscribe(),
            open: None,
        }
    }

    fn accept(&mut self, event: ReadboardRuntimeEvent) -> Option<ReadboardInboundEvent> {
        match event {
            ReadboardRuntimeEvent::Lifecycle(snapshot) => {
                self.open = (snapshot.phase == Phase::Ready).then_some(snapshot.generation);
                None
            }
            ReadboardRuntimeEvent::Inbound(event) => {
                assert_eq!(
                    self.open,
                    Some(event.generation),
                    "inbound line outside its Ready generation: {event:?}"
                );
                Some(event)
            }
        }
    }

    fn next_inbound(&mut self) -> ReadboardInboundEvent {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let event = self
                .rx
                .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                .expect("inbound event");
            if let Some(inbound) = self.accept(event) {
                return inbound;
            }
        }
    }

    fn drained_without_inbound(&mut self) -> bool {
        let pending: Vec<_> = self.rx.try_iter().collect();
        pending.into_iter().all(|event| self.accept(event).is_none())
    }
}

fn assert_reaped(pid: u32, endpoint: Option<&str>) {
    #[cfg(unix)]
    assert!(
        !Path::new(&format!("/proc/{pid}")).exists(),
        "owned child {pid} not reaped"
    );
    #[cfg(not(unix))]
    let _ = pid;
    if let Some(endpoint) = endpoint {
        assert!(
            TcpStream::connect_timeout(&endpoint.parse().unwrap(), Duration::from_millis(50)).is_err(),
            "listener remains open"
        );
    }
}

#[test]
fn ready_requires_requested_version_and_stop_reaps_real_child_socket() {
    let fixture = Fixture::new("split");
    let runtime = controlled();
    fixture.start(&runtime);
    let ready = wait_for(&runtime, |s| s.phase == Phase::Ready);
    assert_eq!(ready.wire_version.as_deref(), Some("220430"));
    assert!(fixture.path.with_extension("requested").exists());
    let stopped = runtime.stop(Duration::from_secs(2));
    assert_eq!(stopped.phase, Phase::Stopped);
    assert!(!stopped.resources_held);
    assert_reaped(fixture.pid(), ready.endpoint.as_deref());
}

#[test]
fn unknown_wrong_and_empty_versions_are_incompatible_and_reaped() {
    for mode in ["wrong", "future", "empty_version"] {
        let fixture = Fixture::new(mode);
        let runtime = controlled();
        fixture.start(&runtime);
        let terminal = wait_for(&runtime, |s| !s.resources_held);
        assert_eq!(terminal.phase, Phase::Incompatible, "{mode}: {terminal:?}");
        assert_reaped(fixture.pid(), None);
    }
}

#[test]
fn missing_version_bare_tcp_and_pre_ready_version_cannot_admit() {
    for mode in ["missing_version", "tcp_only", "version_first", "no_connect"] {
        let fixture = Fixture::new(mode);
        let runtime = controlled();
        let events = runtime.subscribe();
        fixture.start(&runtime);
        let terminal = wait_for(&runtime, |s| !s.resources_held);
        assert_eq!(terminal.phase, Phase::Timeout, "{mode}: {terminal:?}");
        assert!(events
            .try_iter()
            .all(|event| !matches!(event, ReadboardRuntimeEvent::Lifecycle(s) if s.phase == Phase::Ready)));
        assert_reaped(fixture.pid(), None);
    }
}

#[test]
fn process_exit_before_ready_is_typed_and_reaped() {
    let fixture = Fixture::new("early_exit");
    let runtime = controlled();
    fixture.start(&runtime);
    let terminal = wait_for(&runtime, |s| !s.resources_held);
    assert_eq!(terminal.phase, Phase::Exited);
    assert!(terminal.message.contains("23"));
    assert_reaped(fixture.pid(), None);
}

#[test]
fn stop_seals_late_ready_before_reaping_and_restart_uses_new_generation() {
    let late = Fixture::new("late_ready");
    let good = Fixture::new("split");
    let runtime = controlled();
    let first = late.start(&runtime);
    wait_for(&runtime, |s| s.process_id.is_some());
    wait_for(&runtime, |_| late.path.with_extension("connected").exists());
    let stopped = runtime.stop(Duration::ZERO);
    assert!(stopped.generation > first.generation);
    let events = runtime.subscribe();
    // The obsolete process is now permitted to emit its delayed ready/version.
    fs::write(late.path.with_extension("release"), "release").unwrap();
    let final_stop = runtime.stop(Duration::from_secs(2));
    assert!(!final_stop.resources_held);
    assert_reaped(late.pid(), None);
    assert!(events
        .try_iter()
        .all(|event| !matches!(event, ReadboardRuntimeEvent::Lifecycle(s) if s.phase == Phase::Ready)));
    runtime
        .restart(good.path.to_str(), Duration::from_secs(2))
        .unwrap();
    let ready = wait_for(&runtime, |s| s.phase == Phase::Ready);
    assert!(ready.generation > stopped.generation);
    assert_ne!(ready.process_id, Some(late.pid()));
    assert!(!runtime.stop(Duration::from_secs(2)).resources_held);
    assert_reaped(good.pid(), ready.endpoint.as_deref());
}

#[test]
fn teardown_shares_budget_retains_cleanup_and_prevents_new_start() {
    let fixture = Fixture::new("ignore_quit");
    let runtime = controlled();
    fixture.start(&runtime);
    let ready = wait_for(&runtime, |s| s.phase == Phase::Ready);
    let began = Instant::now();
    assert!(!runtime.teardown(Duration::ZERO).is_empty());
    assert!(began.elapsed() < Duration::from_millis(200));
    assert!(runtime.snapshot().resources_held);
    assert!(fixture.start_result(&runtime).is_err());
    assert!(runtime.teardown(Duration::from_secs(2)).is_empty());
    assert_reaped(fixture.pid(), ready.endpoint.as_deref());
}

impl Fixture {
    fn start_result(&self, runtime: &ReadboardRuntime) -> Result<ReadboardRuntimeDto, String> {
        runtime.start(self.path.to_str())
    }
}

#[test]
fn disconnect_and_oversized_stream_are_terminal_without_reconnect() {
    for mode in ["disconnect", "oversized"] {
        let fixture = Fixture::new(mode);
        let runtime = controlled();
        fixture.start(&runtime);
        let terminal = wait_for(&runtime, |s| !s.resources_held);
        if mode == "oversized" {
            assert_eq!(terminal.phase, Phase::Incompatible);
        } else {
            assert_eq!(terminal.phase, Phase::Disconnected);
        }
        assert_reaped(fixture.pid(), None);
        let generation = terminal.generation;
        thread::sleep(Duration::from_millis(30));
        assert_eq!(runtime.snapshot().generation, generation);
        assert!(!runtime.snapshot().resources_held);
    }
}

#[test]
fn unsupported_platform_and_missing_path_do_not_spawn() {
    let unsupported = ReadboardRuntime::new(false, STARTUP_BUDGET);
    let snapshot = unsupported.start(Some("C:\\readboard.exe")).unwrap();
    assert_eq!(snapshot.phase, Phase::Unavailable);
    assert!(!snapshot.resources_held);
    let supported = controlled();
    assert_eq!(supported.start(None).unwrap().phase, Phase::Unavailable);
    assert_eq!(
        supported.start(Some("/no/such/readboard.exe")).unwrap().phase,
        Phase::Unavailable
    );
    assert_eq!(
        supported.start(Some("/tmp/readboard.bat")).unwrap().phase,
        Phase::Unavailable
    );
    assert!(!supported.snapshot().resources_held);
}

#[test]
fn multi_frames_in_order_with_malformed_batch_and_request_focus() {
    let fixture = Fixture::new("multi_frames");
    let runtime = controlled();
    let mut events = Events::new(&runtime);
    fixture.start(&runtime);
    let ready = wait_for(&runtime, |s| s.phase == Phase::Ready);
    let generation = ready.generation;

    // 1. Control(Start)
    let event1 = events.next_inbound();
    assert_eq!(event1.generation, generation);
    assert_eq!(
        event1.inbound,
        ReadboardInbound::Control(ReadboardControl::Start { size: Some((3, 3)) })
    );

    // 2. Frame 1 (Fox frame split across writes)
    let event2 = events.next_inbound();
    assert_eq!(event2.generation, generation);
    match event2.inbound {
        ReadboardInbound::Frame(frame) => {
            assert_eq!(frame.width, 3);
            assert_eq!(frame.height, 3);
            assert_eq!(frame.context.platform, ReadboardPlatform::Fox);
            assert_eq!(frame.context.room_token.as_deref(), Some("room-1"));
            assert_eq!(frame.context.live_title_move, Some(5));
            assert_eq!(frame.codes, vec![0, 0, 0, 0, 1, 0, 0, 0, 2]);
        }
        other => panic!("expected frame 1, got {other:?}"),
    }

    // 3. Rejected batch
    let event3 = events.next_inbound();
    assert_eq!(event3.generation, generation);
    match event3.inbound {
        ReadboardInbound::Rejected(reason) => {
            assert!(!reason.is_empty());
        }
        other => panic!("expected rejected, got {other:?}"),
    }

    // 4. Frame 2 (coalesced in one write, arriving after Rejected without stopping)
    let event4 = events.next_inbound();
    assert_eq!(event4.generation, generation);
    match event4.inbound {
        ReadboardInbound::Frame(frame) => {
            assert_eq!(frame.width, 3);
            assert_eq!(frame.height, 3);
            assert_eq!(frame.context.platform, ReadboardPlatform::Fox);
            assert_eq!(frame.context.room_token.as_deref(), Some("room-2"));
            assert_eq!(frame.context.live_title_move, Some(7));
            assert_eq!(frame.codes, vec![1, 0, 0, 0, 2, 0, 0, 0, 3]);
        }
        other => panic!("expected frame 2, got {other:?}"),
    }

    // 5. Outbound request_focus:
    // Stale generation returns Err and writes nothing
    let stale_res = runtime.request_focus(generation + 99);
    assert!(stale_res.is_err());
    assert!(!fixture.path.with_extension("loss").exists());

    // Current generation succeeds and delivers exactly one loss line
    runtime.request_focus(generation).expect("request_focus ok");
    let deadline = Instant::now() + Duration::from_secs(2);
    let loss_path = fixture.path.with_extension("loss");
    loop {
        if loss_path.exists() {
            let content = fs::read_to_string(&loss_path).unwrap();
            if content == "loss\n" {
                break;
            }
        }
        assert!(
            Instant::now() < deadline,
            "loss line was not delivered to fixture"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(fs::read_to_string(&loss_path).unwrap(), "loss\n");

    let stopped = runtime.stop(Duration::from_secs(2));
    assert_eq!(stopped.phase, Phase::Stopped);
    assert_reaped(fixture.pid(), ready.endpoint.as_deref());
}

#[test]
fn child_disconnect_after_frames_becomes_disconnected_and_no_further_events() {
    let fixture = Fixture::new("frames_disconnect");
    let runtime = controlled();
    let mut events = Events::new(&runtime);
    fixture.start(&runtime);
    let ready = wait_for(&runtime, |s| s.phase == Phase::Ready);

    // Initial frame arrives
    let event1 = events.next_inbound();
    assert_eq!(event1.generation, ready.generation);
    let event2 = events.next_inbound();
    assert_eq!(event2.generation, ready.generation);
    assert!(matches!(event2.inbound, ReadboardInbound::Frame(_)));

    // After child disconnects, phase becomes Disconnected
    let terminal = wait_for(&runtime, |s| !s.resources_held);
    assert_eq!(terminal.phase, Phase::Disconnected);
    assert_reaped(fixture.pid(), None);

    // No further events arrive
    thread::sleep(Duration::from_millis(50));
    assert!(events.drained_without_inbound());
}

#[test]
fn restart_yields_new_generation_whose_frames_are_delivered_while_late_old_lines_are_not() {
    let late = Fixture::new("late_frames");
    let good = Fixture::new("multi_frames");
    let runtime = controlled();
    let mut events = Events::new(&runtime);

    late.start(&runtime);
    let ready1 = wait_for(&runtime, |s| s.phase == Phase::Ready);
    let g1 = ready1.generation;

    // Process 1 emits start and frame 1
    let event1 = events.next_inbound();
    assert_eq!(event1.generation, g1);
    let event2 = events.next_inbound();
    assert_eq!(event2.generation, g1);

    // Stop with Duration::ZERO seals generation g1 immediately without waiting for child exit
    let stopped = runtime.stop(Duration::ZERO);
    assert!(stopped.generation > g1);

    // Release obsolete process to emit late lines
    fs::write(late.path.with_extension("release"), "release").unwrap();

    // Reap obsolete process
    let reaped = runtime.stop(Duration::from_secs(2));
    assert!(!reaped.resources_held);
    assert_reaped(late.pid(), None);

    // Confirm no late frames from old generation were delivered
    assert!(events.drained_without_inbound());

    // Restart with good fixture
    runtime
        .restart(good.path.to_str(), Duration::from_secs(2))
        .expect("restart ok");
    let ready2 = wait_for(&runtime, |s| s.phase == Phase::Ready);
    let g2 = ready2.generation;
    assert!(g2 > stopped.generation);

    // Good fixture's start and frames arrive with generation g2
    let event3 = events.next_inbound();
    assert_eq!(event3.generation, g2);

    let event4 = events.next_inbound();
    assert_eq!(event4.generation, g2);

    // Stale generation g1 focus request fails
    assert!(runtime.request_focus(g1).is_err());
    // Stale intermediate stopped generation focus request fails
    assert!(runtime.request_focus(stopped.generation).is_err());

    // Current generation g2 focus request succeeds
    runtime.request_focus(g2).expect("focus g2 ok");
    let deadline = Instant::now() + Duration::from_secs(2);
    let loss_path = good.path.with_extension("loss");
    loop {
        if loss_path.exists() {
            let content = fs::read_to_string(&loss_path).unwrap();
            if content == "loss\n" {
                break;
            }
        }
        assert!(Instant::now() < deadline, "loss line not observed");
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(fs::read_to_string(&loss_path).unwrap(), "loss\n");

    let final_stop = runtime.stop(Duration::from_secs(2));
    assert_eq!(final_stop.phase, Phase::Stopped);
    assert_reaped(good.pid(), ready2.endpoint.as_deref());
}
