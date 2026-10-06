use app_model::{
    NetworkModeDto, NetworkSettingsDto, ProviderErrorKind, ProviderFetchMethod, ProviderFetchRequest,
    ProviderKind,
};
use provider_core::{
    network::{NetworkOperation, NetworkState},
    ProviderTransport,
};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static ENV_LOCK: Mutex<()> = Mutex::new(());

const PROXY_ENV_KEYS: &[&str] = &[
    "http_proxy",
    "HTTP_PROXY",
    "https_proxy",
    "HTTPS_PROXY",
    "all_proxy",
    "ALL_PROXY",
    "no_proxy",
    "NO_PROXY",
];

struct EnvGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    original: Vec<(&'static str, Option<String>)>,
}

impl EnvGuard {
    fn new() -> Self {
        let lock = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let original = PROXY_ENV_KEYS
            .iter()
            .map(|&k| (k, std::env::var(k).ok()))
            .collect();
        for &k in PROXY_ENV_KEYS {
            std::env::remove_var(k);
        }
        Self {
            _lock: lock,
            original,
        }
    }

    fn set(&self, key: &str, val: &str) {
        std::env::set_var(key, val);
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (k, val) in &self.original {
            match val {
                Some(v) => std::env::set_var(k, v),
                None => std::env::remove_var(k),
            }
        }
    }
}

fn accept_timeout(listener: &TcpListener, timeout: Duration) -> std::io::Result<std::net::TcpStream> {
    let deadline = Instant::now() + timeout;
    listener.set_nonblocking(true)?;
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_nonblocking(false)?;
                stream.set_read_timeout(Some(Duration::from_secs(3)))?;
                stream.set_write_timeout(Some(Duration::from_secs(3)))?;
                return Ok(stream);
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "accept timed out",
                    ));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(e),
        }
    }
}

struct ControlledRead {
    request: ProviderFetchRequest,
    accepted: std::sync::mpsc::Receiver<()>,
    release: std::sync::mpsc::Sender<()>,
    server: std::thread::JoinHandle<()>,
}

impl ControlledRead {
    fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (accepted_tx, accepted) = std::sync::mpsc::channel();
        let (release, release_rx) = std::sync::mpsc::channel();
        let server = std::thread::spawn(move || {
            let mut socket = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
            let mut first_byte = [0u8; 1];
            socket.read_exact(&mut first_byte).unwrap();
            accepted_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            let _ = socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\n(;SZ[19])");
        });
        Self {
            request: ProviderFetchRequest {
                provider: ProviderKind::Yike,
                url: format!("http://127.0.0.1:{port}/controlled"),
                method: ProviderFetchMethod::Get,
                headers: Default::default(),
                body: None,
                source_url: None,
                source_id: None,
                timeout_ms: None,
            },
            accepted,
            release,
            server,
        }
    }

    fn start(
        &self,
        operation: NetworkOperation,
    ) -> std::thread::JoinHandle<provider_core::ProviderResult<app_model::ProviderFetchResult>> {
        let request = self.request.clone();
        let client = std::thread::spawn(move || operation.fetch(&request));
        self.accepted.recv_timeout(Duration::from_secs(3)).unwrap();
        client
    }

    fn finish(self) {
        self.release.send(()).unwrap();
        self.server.join().unwrap();
    }
}

#[test]
fn manual_policy_is_consumed_by_provider_transport_and_records_safe_route() {
    let _env = EnvGuard::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let mut buffer = [0; 4096];
        let n = socket.read(&mut buffer).unwrap();
        let wire = String::from_utf8_lossy(&buffer[..n]);
        assert!(
            wire.starts_with("GET http://provider.invalid/game?signature=private HTTP/1.1\r\n"),
            "manual route request wire mismatch"
        );
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10\r\nConnection: close\r\n\r\n(;SZ[9];B)")
            .unwrap();
    });
    let state = NetworkState::default();
    let snapshot = state
        .commit(
            NetworkSettingsDto {
                mode: NetworkModeDto::Manual,
                manual_host: "127.0.0.1".into(),
                manual_port: port,
            },
            || Ok(()),
        )
        .unwrap();
    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let operation = state.operation(&id).unwrap();
    let result = operation
        .fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: "http://provider.invalid/game?signature=private".into(),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
        .unwrap();
    assert_eq!(result.payload, "(;SZ[9];B)");
    assert_eq!(operation.routes()[0].target, "provider.invalid:80");
    assert_eq!(
        operation.routes()[0].proxy.as_deref(),
        Some(format!("127.0.0.1:{port}").as_str())
    );
    assert!(!result.url.contains("private"));
    server.join().unwrap();
}

#[test]
fn redirect_requests_hit_correct_target_and_record_proxy_routes() {
    let _env = EnvGuard::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = std::thread::spawn(move || {
        let mut sock1 = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        let mut buf = [0u8; 2048];
        let n1 = sock1.read(&mut buf).unwrap();
        let wire1 = String::from_utf8_lossy(&buf[..n1]);
        assert!(wire1.starts_with("GET http://origin.invalid/first HTTP/1.1\r\n"));
        sock1
            .write_all(b"HTTP/1.1 302 Found\r\nLocation: http://destination.invalid/second\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
        drop(sock1);

        let mut sock2 = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        let n2 = sock2.read(&mut buf).unwrap();
        let wire2 = String::from_utf8_lossy(&buf[..n2]);
        assert!(wire2.starts_with("GET http://destination.invalid/second HTTP/1.1\r\n"));
        sock2
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nfinished")
            .unwrap();
    });

    let state = NetworkState::default();
    let snapshot = state
        .commit(
            NetworkSettingsDto {
                mode: NetworkModeDto::Manual,
                manual_host: "127.0.0.1".into(),
                manual_port: port,
            },
            || Ok(()),
        )
        .unwrap();

    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let operation = state.operation(&id).unwrap();
    let result = operation
        .fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: "http://origin.invalid/first".into(),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
        .unwrap();

    assert_eq!(result.status_code, 200);
    assert_eq!(result.payload, "finished");
    assert_eq!(result.url, "http://destination.invalid/second");
    let routes = operation.routes();
    assert_eq!(routes.len(), 2);
    assert_eq!(routes[0].target, "origin.invalid:80");
    assert_eq!(
        routes[0].proxy.as_deref(),
        Some(format!("127.0.0.1:{port}").as_str())
    );
    assert_eq!(routes[1].target, "destination.invalid:80");
    assert_eq!(
        routes[1].proxy.as_deref(),
        Some(format!("127.0.0.1:{port}").as_str())
    );
    server.join().unwrap();
}

#[test]
fn direct_mode_ignores_proxy_environment() {
    let env = EnvGuard::new();
    env.set("HTTP_PROXY", "http://127.0.0.1:1");
    env.set("ALL_PROXY", "http://127.0.0.1:1");

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = std::thread::spawn(move || {
        let mut sock = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        let mut buf = [0u8; 1024];
        let n = sock.read(&mut buf).unwrap();
        let wire = String::from_utf8_lossy(&buf[..n]);
        assert!(wire.starts_with("GET /direct-test HTTP/1.1\r\n"));
        sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\ndirect")
            .unwrap();
    });

    let state = NetworkState::default();
    let snapshot = state.snapshot();
    assert_eq!(snapshot.settings.mode, NetworkModeDto::Direct);

    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let operation = state.operation(&id).unwrap();
    let result = operation
        .fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: format!("http://127.0.0.1:{port}/direct-test"),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
        .unwrap();

    assert_eq!(result.payload, "direct");
    let routes = operation.routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].target, format!("127.0.0.1:{port}"));
    assert_eq!(routes[0].proxy, None);
    assert_eq!(routes[0].source, "direct");
    server.join().unwrap();
}

#[test]
fn manual_mode_ignores_other_proxy_endpoints() {
    let env = EnvGuard::new();
    env.set("HTTP_PROXY", "http://env-proxy.invalid:8888");
    env.set("ALL_PROXY", "http://env-proxy.invalid:8888");

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = std::thread::spawn(move || {
        let mut sock = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        let mut buf = [0u8; 1024];
        let n = sock.read(&mut buf).unwrap();
        let wire = String::from_utf8_lossy(&buf[..n]);
        assert!(wire.starts_with("GET http://target.invalid/manual HTTP/1.1\r\n"));
        sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nmanual")
            .unwrap();
    });

    let state = NetworkState::default();
    let snapshot = state
        .commit(
            NetworkSettingsDto {
                mode: NetworkModeDto::Manual,
                manual_host: "127.0.0.1".into(),
                manual_port: port,
            },
            || Ok(()),
        )
        .unwrap();

    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let operation = state.operation(&id).unwrap();
    let result = operation
        .fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: "http://target.invalid/manual".into(),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
        .unwrap();

    assert_eq!(result.payload, "manual");
    let routes = operation.routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].target, "target.invalid:80");
    assert_eq!(
        routes[0].proxy.as_deref(),
        Some(format!("127.0.0.1:{port}").as_str())
    );
    assert_eq!(routes[0].source, "manual");
    server.join().unwrap();
}

#[test]
fn no_proxy_bypass_respects_port_and_domain_boundaries() {
    let env = EnvGuard::new();
    let direct_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let direct_port = direct_listener.local_addr().unwrap().port();

    let proxy_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let proxy_port = proxy_listener.local_addr().unwrap().port();

    env.set("NO_PROXY", &format!("localhost:{direct_port},.corp.invalid"));

    let direct_server = std::thread::spawn(move || {
        let mut sock = accept_timeout(&direct_listener, Duration::from_secs(3)).unwrap();
        let mut buf = [0u8; 1024];
        let n = sock.read(&mut buf).unwrap();
        let wire = String::from_utf8_lossy(&buf[..n]);
        assert!(wire.starts_with("GET /port-match HTTP/1.1\r\n"));
        sock.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\ndirect")
            .unwrap();
    });

    let proxy_server = std::thread::spawn(move || {
        let mut sock1 = accept_timeout(&proxy_listener, Duration::from_secs(3)).unwrap();
        let mut buf = [0u8; 1024];
        let n1 = sock1.read(&mut buf).unwrap();
        let wire1 = String::from_utf8_lossy(&buf[..n1]);
        assert!(wire1.starts_with("GET http://localhost:9999/port-mismatch HTTP/1.1\r\n"));
        sock1
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: close\r\n\r\nproxy-port-ok")
            .unwrap();
        drop(sock1);

        let mut sock2 = accept_timeout(&proxy_listener, Duration::from_secs(3)).unwrap();
        let n2 = sock2.read(&mut buf).unwrap();
        let wire2 = String::from_utf8_lossy(&buf[..n2]);
        assert!(wire2.starts_with("GET http://notcorp.invalid/domain-mismatch HTTP/1.1\r\n"));
        sock2
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\nproxy-domain-ok")
            .unwrap();
        drop(sock2);
    });

    let state = NetworkState::default();
    let snapshot = state
        .commit(
            NetworkSettingsDto {
                mode: NetworkModeDto::Manual,
                manual_host: "127.0.0.1".into(),
                manual_port: proxy_port,
            },
            || Ok(()),
        )
        .unwrap();

    // 1. Port match: localhost:{direct_port} bypasses proxy
    let id1 = state.begin(snapshot.policy_revision, 0).unwrap();
    let op1 = state.operation(&id1).unwrap();
    let res1 = op1
        .fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: format!("http://localhost:{direct_port}/port-match"),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
        .unwrap();
    assert_eq!(res1.payload, "direct");
    let routes1 = op1.routes();
    assert_eq!(routes1.len(), 1);
    assert_eq!(routes1[0].source, "no-proxy");
    assert_eq!(routes1[0].proxy, None);

    // 2. Port mismatch: localhost:9999 does not match localhost:{direct_port}, routed to proxy
    let id2 = state.begin(snapshot.policy_revision, 0).unwrap();
    let op2 = state.operation(&id2).unwrap();
    let res2 = op2
        .fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: "http://localhost:9999/port-mismatch".into(),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
        .unwrap();
    assert_eq!(res2.payload, "proxy-port-ok");
    let routes2 = op2.routes();
    assert_eq!(routes2.len(), 1);
    assert_eq!(routes2[0].source, "manual");
    assert_eq!(
        routes2[0].proxy.as_deref(),
        Some(format!("127.0.0.1:{proxy_port}").as_str())
    );

    // 3. Domain mismatch: notcorp.invalid does not match .corp.invalid, routed to proxy
    let id3 = state.begin(snapshot.policy_revision, 0).unwrap();
    let op3 = state.operation(&id3).unwrap();
    let res3 = op3
        .fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: "http://notcorp.invalid/domain-mismatch".into(),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
        .unwrap();
    assert_eq!(res3.payload, "proxy-domain-ok");
    let routes3 = op3.routes();
    assert_eq!(routes3.len(), 1);
    assert_eq!(routes3[0].source, "manual");
    assert_eq!(
        routes3[0].proxy.as_deref(),
        Some(format!("127.0.0.1:{proxy_port}").as_str())
    );

    direct_server.join().unwrap();
    proxy_server.join().unwrap();
}

#[test]
fn failed_proxy_has_no_direct_fallback() {
    let _env = EnvGuard::new();
    let direct_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let direct_port = direct_listener.local_addr().unwrap().port();

    let direct_server = std::thread::spawn(move || {
        let conn = accept_timeout(&direct_listener, Duration::from_millis(500));
        assert!(
            conn.is_err(),
            "direct server must not receive connection when proxy fails"
        );
    });

    let closed_port = {
        let temp = TcpListener::bind("127.0.0.1:0").unwrap();
        temp.local_addr().unwrap().port()
    };

    let state = NetworkState::default();
    let snapshot = state
        .commit(
            NetworkSettingsDto {
                mode: NetworkModeDto::Manual,
                manual_host: "127.0.0.1".into(),
                manual_port: closed_port,
            },
            || Ok(()),
        )
        .unwrap();

    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let op = state.operation(&id).unwrap();

    let result = op.fetch(&ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url: format!("http://127.0.0.1:{direct_port}/resource"),
        method: ProviderFetchMethod::Get,
        headers: Default::default(),
        body: None,
        source_url: None,
        source_id: None,
        timeout_ms: None,
    });

    let err = result.unwrap_err();
    assert_eq!(err.kind, ProviderErrorKind::ProxyFailed);
    let routes = op.routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].target, format!("127.0.0.1:{direct_port}"));
    assert_eq!(
        routes[0].proxy.as_deref(),
        Some(format!("127.0.0.1:{closed_port}").as_str())
    );
    assert_eq!(routes[0].source, "manual");

    direct_server.join().unwrap();
}

#[test]
fn transient_statuses_use_at_most_four_attempts_per_operation() {
    let _env = EnvGuard::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = std::thread::spawn(move || {
        let mut count = 0;
        while let Ok(mut sock) = accept_timeout(&listener, Duration::from_secs(2)) {
            let mut buf = [0u8; 1024];
            let _ = sock.read(&mut buf);
            sock.write_all(
                b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
            count += 1;
            if count == 4 {
                break;
            }
        }
        count
    });

    let state = NetworkState::default();
    let snapshot = state.snapshot();
    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let op = state.operation(&id).unwrap();

    let result = op.fetch(&ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url: format!("http://127.0.0.1:{port}/transient"),
        method: ProviderFetchMethod::Get,
        headers: Default::default(),
        body: None,
        source_url: None,
        source_id: None,
        timeout_ms: None,
    });

    let err = result.unwrap_err();
    assert_eq!(err.kind, ProviderErrorKind::TransportFailed);
    assert_eq!(op.routes().len(), 4);
    let attempts = server.join().unwrap();
    assert_eq!(attempts, 4);
}

#[test]
fn auth_and_not_found_statuses_do_not_retry() {
    let _env = EnvGuard::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = std::thread::spawn(move || {
        let mut sock1 = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        let mut buf = [0u8; 1024];
        let _ = sock1.read(&mut buf);
        sock1
            .write_all(b"HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
        drop(sock1);

        let mut sock2 = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        let _ = sock2.read(&mut buf);
        sock2
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
        drop(sock2);
    });

    let state = NetworkState::default();
    let snapshot = state.snapshot();

    // 1. 401 Unauthorized -> AuthenticationFailed, no retry
    let id1 = state.begin(snapshot.policy_revision, 0).unwrap();
    let op1 = state.operation(&id1).unwrap();
    let res1 = op1.fetch(&ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url: format!("http://127.0.0.1:{port}/auth-fail"),
        method: ProviderFetchMethod::Get,
        headers: Default::default(),
        body: None,
        source_url: None,
        source_id: None,
        timeout_ms: None,
    });
    assert_eq!(res1.unwrap_err().kind, ProviderErrorKind::AuthenticationFailed);
    assert_eq!(op1.routes().len(), 1);

    // 2. 404 Not Found -> NotFound, no retry
    let id2 = state.begin(snapshot.policy_revision, 0).unwrap();
    let op2 = state.operation(&id2).unwrap();
    let res2 = op2.fetch(&ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url: format!("http://127.0.0.1:{port}/not-found"),
        method: ProviderFetchMethod::Get,
        headers: Default::default(),
        body: None,
        source_url: None,
        source_id: None,
        timeout_ms: None,
    });
    assert_eq!(res2.unwrap_err().kind, ProviderErrorKind::NotFound);
    assert_eq!(op2.routes().len(), 1);

    server.join().unwrap();
}

#[test]
fn cancellation_bounds_slow_response() {
    let _env = EnvGuard::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (accepted_tx, accepted_rx) = std::sync::mpsc::channel();

    let server = std::thread::spawn(move || {
        let mut socket = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        let _ = accepted_tx.send(());
        let mut buf = [0u8; 1024];
        let _ = socket.read(&mut buf);
        socket.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
        let mut discard = [0u8; 64];
        while let Ok(n) = socket.read(&mut discard) {
            if n == 0 {
                break;
            }
        }
    });

    let state = std::sync::Arc::new(NetworkState::default());
    let snapshot = state.snapshot();
    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let op = state.operation(&id).unwrap();

    let client = std::thread::spawn(move || {
        op.fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: format!("http://127.0.0.1:{port}/slow"),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
    });

    accepted_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    std::thread::sleep(Duration::from_millis(30));

    let start_cancel = Instant::now();
    state.cancel(&id);

    let result = client.join().unwrap();
    let elapsed = start_cancel.elapsed();

    assert!(elapsed < Duration::from_secs(2), "cancellation took {elapsed:?}");
    assert_eq!(result.unwrap_err().kind, ProviderErrorKind::Cancelled);
    server.join().unwrap();
}

#[test]
fn policy_commit_lease_invalidation_and_preservation() {
    let _env = EnvGuard::new();
    let state = NetworkState::default();
    let snap0 = state.snapshot();
    assert_eq!(snap0.policy_revision, 0);

    let id0 = state.begin(snap0.policy_revision, 0).unwrap();
    let op0 = state.operation(&id0).unwrap();
    assert!(op0.lease().check().is_ok());

    // Failed commit: validation fails
    let err_settings = NetworkSettingsDto {
        mode: NetworkModeDto::Manual,
        manual_host: "".into(),
        manual_port: 8080,
    };
    let fail_res1 = state.commit(err_settings, || Ok(()));
    assert!(fail_res1.is_err());
    assert_eq!(state.snapshot().policy_revision, 0);
    assert!(op0.lease().check().is_ok());
    assert!(state.operation(&id0).is_ok());

    // Failed commit: persistence fails
    let valid_settings = NetworkSettingsDto {
        mode: NetworkModeDto::Manual,
        manual_host: "127.0.0.1".into(),
        manual_port: 8080,
    };
    let fail_res2 = state.commit(valid_settings.clone(), || Err("disk write failed".into()));
    assert!(fail_res2.is_err());
    assert_eq!(state.snapshot().policy_revision, 0);
    assert!(op0.lease().check().is_ok());
    assert!(state.operation(&id0).is_ok());

    // Successful commit: persistence succeeds
    let success_snap = state.commit(valid_settings, || Ok(())).unwrap();
    assert_eq!(success_snap.policy_revision, 1);
    assert_eq!(state.snapshot().policy_revision, 1);

    // Old lease is now invalidated
    assert_eq!(
        op0.lease().check().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    assert_eq!(
        state.operation(&id0).err().unwrap().kind,
        ProviderErrorKind::Cancelled
    );

    // Old policy revision cannot begin a new operation
    assert_eq!(
        state.begin(snap0.policy_revision, 0).unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );

    // Invalidation prevents fetch on old operation
    let old_fetch_err = op0
        .fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: "http://127.0.0.1:80/cancelled".into(),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
        .unwrap_err();
    assert_eq!(old_fetch_err.kind, ProviderErrorKind::Cancelled);

    // New policy revision can begin and execute
    let id1 = state.begin(success_snap.policy_revision, 0).unwrap();
    let op1 = state.operation(&id1).unwrap();
    assert!(op1.lease().check().is_ok());
}

#[test]
fn yike_non_idempotent_post_times_out_at_ten_seconds_without_retry() {
    let _env = EnvGuard::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = std::thread::spawn(move || {
        let mut sock = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(18))).unwrap();
        let mut buf = [0u8; 1024];
        let _ = sock.read(&mut buf);
        let mut discard = [0u8; 64];
        while let Ok(n) = sock.read(&mut discard) {
            if n == 0 {
                break;
            }
        }
        drop(sock);
        let retry = accept_timeout(&listener, Duration::from_millis(500));
        assert!(retry.is_err(), "Yike non-idempotent POST must not retry");
        1
    });

    let state = NetworkState::default();
    let snapshot = state.snapshot();
    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let op = state.operation(&id).unwrap();

    let start = Instant::now();
    let result = op.fetch(&ProviderFetchRequest {
        provider: ProviderKind::Yike,
        url: format!("http://127.0.0.1:{port}/yike-post"),
        method: ProviderFetchMethod::Post,
        headers: Default::default(),
        body: Some("test-payload".into()),
        source_url: None,
        source_id: None,
        timeout_ms: None,
    });
    let elapsed = start.elapsed();

    let err = result.unwrap_err();
    assert_eq!(err.kind, ProviderErrorKind::Timeout);
    assert_eq!(op.routes().len(), 1);
    assert!(
        elapsed >= Duration::from_secs(9) && elapsed <= Duration::from_secs(15),
        "Yike POST timeout should be around 10s, was {elapsed:?}"
    );
    let attempts = server.join().unwrap();
    assert_eq!(attempts, 1);
}

#[test]
fn fox_post_slow_server_returns_read_timeout_without_retry() {
    let _env = EnvGuard::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = std::thread::spawn(move || {
        let mut sock = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        sock.set_read_timeout(Some(Duration::from_secs(33))).unwrap();
        let mut buf = [0u8; 1024];
        let _ = sock.read(&mut buf);
        let mut discard = [0u8; 64];
        while let Ok(n) = sock.read(&mut discard) {
            if n == 0 {
                break;
            }
        }
        drop(sock);
        let retry = accept_timeout(&listener, Duration::from_millis(500));
        assert!(retry.is_err(), "Fox non-CGI POST must not retry");
        1
    });

    let state = NetworkState::default();
    let snapshot = state.snapshot();
    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let op = state.operation(&id).unwrap();

    let start = Instant::now();
    let result = op.fetch(&ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url: format!("http://127.0.0.1:{port}/fox-read-slow"),
        method: ProviderFetchMethod::Post,
        headers: Default::default(),
        body: Some("fox-body".into()),
        source_url: None,
        source_id: None,
        timeout_ms: None,
    });
    let elapsed = start.elapsed();

    let err = result.unwrap_err();
    assert_eq!(err.kind, ProviderErrorKind::Timeout);
    assert_eq!(op.routes().len(), 1);
    assert!(
        elapsed >= Duration::from_secs(24) && elapsed <= Duration::from_secs(30),
        "Fox read timeout should be around 25s, was {elapsed:?}"
    );
    let attempts = server.join().unwrap();
    assert_eq!(attempts, 1);
}

#[test]
fn fetch_alternatives_shares_retry_budget_across_endpoints_and_terminal_blocks_restart() {
    let _env = EnvGuard::new();
    let listener_a = TcpListener::bind("127.0.0.1:0").unwrap();
    let port_a = listener_a.local_addr().unwrap().port();
    let listener_b = TcpListener::bind("127.0.0.1:0").unwrap();
    let port_b = listener_b.local_addr().unwrap().port();
    let (last_attempt_tx, last_attempt_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();

    let server_a = std::thread::spawn(move || {
        let mut sock = accept_timeout(&listener_a, Duration::from_secs(3)).unwrap();
        let mut buf = [0u8; 1024];
        let _ = sock.read(&mut buf);
        sock.write_all(
            b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
        )
        .unwrap();
        drop(sock);
        let retry = accept_timeout(&listener_a, Duration::from_millis(800));
        assert!(retry.is_err(), "Server A must not receive extra attempts");
        1
    });

    let server_b = std::thread::spawn(move || {
        let mut count = 0;
        while let Ok(mut sock) = accept_timeout(&listener_b, Duration::from_secs(2)) {
            let mut buf = [0u8; 1024];
            let _ = sock.read(&mut buf);
            if count == 2 {
                last_attempt_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            }
            sock.write_all(
                b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
            count += 1;
            if count == 3 {
                break;
            }
        }
        let retry = accept_timeout(&listener_b, Duration::from_millis(500));
        assert!(retry.is_err(), "Server B must not receive extra attempts");
        count
    });

    let state = NetworkState::default();
    let snapshot = state.snapshot();
    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let op = state.operation(&id).unwrap();

    let req_a = ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url: format!("http://127.0.0.1:{port_a}/endpoint-a"),
        method: ProviderFetchMethod::Get,
        headers: Default::default(),
        body: None,
        source_url: None,
        source_id: None,
        timeout_ms: None,
    };
    let req_b = ProviderFetchRequest {
        provider: ProviderKind::Fox,
        url: format!("http://127.0.0.1:{port_b}/endpoint-b"),
        method: ProviderFetchMethod::Get,
        headers: Default::default(),
        body: None,
        source_url: None,
        source_id: None,
        timeout_ms: None,
    };

    // First call: switches from endpoint A to endpoint B on retryable failure.
    // Shared retry budget is 3 retries (1 initial attempt on A + 3 retries on B = 4 total attempts).
    assert_eq!(op.retry_count(), 0);
    let fetching = op.clone();
    let requests = [req_a.clone(), req_b.clone()];
    let client = std::thread::spawn(move || fetching.fetch_alternatives(&requests));
    last_attempt_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    assert_eq!(
        op.retry_count(),
        3,
        "third retry is observable before terminal failure"
    );
    release_tx.send(()).unwrap();
    let first_err = client.join().unwrap().unwrap_err();
    assert_eq!(first_err.kind, ProviderErrorKind::TransportFailed);
    assert_eq!(op.routes().len(), 4);

    // Second call on the same operation: operation reached terminal error state.
    // Terminal state must immediately return error without restarting any network attempts.
    let second_result = op.fetch_alternatives(&[req_a, req_b]);
    let second_err = second_result.unwrap_err();
    assert_eq!(second_err.kind, ProviderErrorKind::TransportFailed);
    assert_eq!(
        op.retry_count(),
        3,
        "terminal error must not reset the consumed budget"
    );
    assert_eq!(
        op.routes().len(),
        4,
        "Routes count must remain 4 with no new requests"
    );

    let attempts_a = server_a.join().unwrap();
    let attempts_b = server_b.join().unwrap();
    assert_eq!(attempts_a, 1, "First attempt hits endpoint A");
    assert_eq!(
        attempts_b, 3,
        "Remaining 3 retries hit endpoint B and exhaust operation budget"
    );
}

#[test]
fn shutdown_cancels_active_transfer_drains_within_budget_and_rejects_begin() {
    let _env = EnvGuard::new();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let (accepted_tx, accepted_rx) = std::sync::mpsc::channel();

    let server = std::thread::spawn(move || {
        let mut socket = accept_timeout(&listener, Duration::from_secs(3)).unwrap();
        let _ = accepted_tx.send(());
        let mut buf = [0u8; 1024];
        let _ = socket.read(&mut buf);
        socket.set_read_timeout(Some(Duration::from_millis(500))).unwrap();
        let mut discard = [0u8; 64];
        while let Ok(n) = socket.read(&mut discard) {
            if n == 0 {
                break;
            }
        }
    });

    let state = std::sync::Arc::new(NetworkState::default());
    let snapshot = state.snapshot();
    let id = state.begin(snapshot.policy_revision, 0).unwrap();
    let op = state.operation(&id).unwrap();

    let client = std::thread::spawn(move || {
        op.fetch(&ProviderFetchRequest {
            provider: ProviderKind::Fox,
            url: format!("http://127.0.0.1:{port}/stalled"),
            method: ProviderFetchMethod::Get,
            headers: Default::default(),
            body: None,
            source_url: None,
            source_id: None,
            timeout_ms: None,
        })
    });

    accepted_rx.recv_timeout(Duration::from_secs(3)).unwrap();
    std::thread::sleep(Duration::from_millis(30));

    let shutdown_start = Instant::now();
    let drained = state.shutdown(Duration::from_secs(3));
    let shutdown_elapsed = shutdown_start.elapsed();

    assert!(drained, "shutdown must drain within budget");
    assert!(
        shutdown_elapsed < Duration::from_secs(2),
        "shutdown took {shutdown_elapsed:?}, expected under 2s"
    );

    let client_result = client.join().unwrap();
    let client_err = client_result.unwrap_err();
    assert_eq!(client_err.kind, ProviderErrorKind::Cancelled);

    let new_begin_res = state.begin(snapshot.policy_revision, 0);
    assert_eq!(new_begin_res.unwrap_err().kind, ProviderErrorKind::Cancelled);

    server.join().unwrap();
}

#[test]
fn shutdown_budget_bounds_policy_persistence_and_allows_retry() {
    let state = std::sync::Arc::new(NetworkState::default());
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let saving = state.clone();
    let save = std::thread::spawn(move || {
        saving.commit(NetworkSettingsDto::default(), || {
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok(())
        })
    });
    entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let stopping = state.clone();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let stop = std::thread::spawn(move || {
        done_tx
            .send(stopping.shutdown(Duration::from_millis(40)))
            .unwrap();
    });
    let bounded = done_rx.recv_timeout(Duration::from_millis(500));
    release_tx.send(()).unwrap();
    let saved = save.join().unwrap().unwrap();
    stop.join().unwrap();
    assert_eq!(bounded, Ok(false), "held persistence must yield an exit timeout");
    assert!(
        state.shutdown(Duration::from_secs(1)),
        "Retry must drain after save finishes"
    );
    assert_eq!(
        state.begin(saved.policy_revision, 0).unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
}

#[test]
fn shutdown_budget_bounds_document_install_lease_and_allows_retry() {
    let state = std::sync::Arc::new(NetworkState::default());
    let revision = state.snapshot().policy_revision;
    let identity = state.begin(revision, 0).unwrap();
    let lease = state.operation(&identity).unwrap().lease();
    let held = lease.clone();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let install = std::thread::spawn(move || {
        held.with_valid(|| {
            entered_tx.send(()).unwrap();
            release_rx.recv().unwrap();
        })
    });
    entered_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    let stopping = state.clone();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let stop = std::thread::spawn(move || {
        done_tx
            .send(stopping.shutdown(Duration::from_millis(40)))
            .unwrap();
    });
    let bounded = done_rx.recv_timeout(Duration::from_millis(500));
    release_tx.send(()).unwrap();
    install.join().unwrap().unwrap();
    stop.join().unwrap();
    assert_eq!(bounded, Ok(false), "held install must yield an exit timeout");
    assert!(state.shutdown(Duration::from_secs(1)));
    assert_eq!(lease.check().unwrap_err().kind, ProviderErrorKind::Cancelled);
    assert_eq!(
        state.begin(revision, 0).unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
}

#[test]
fn beginning_preview_preserves_active_sync_read_and_commit_lease() {
    let _env = EnvGuard::new();
    let state = NetworkState::default();
    let revision = state.snapshot().policy_revision;
    let sync = state.begin_sync(revision, 7).unwrap();
    let sync_identity = sync.identity();
    let read = ControlledRead::new();
    let client = read.start(sync.clone());

    let preview_identity = state.begin(revision, 7).unwrap();
    assert!(preview_identity.request_id > sync_identity.request_id);
    assert_eq!(sync.lease().with_valid(|| "sync commit").unwrap(), "sync commit");
    assert!(state.operation(&preview_identity).is_ok());
    assert_eq!(
        state.operation(&sync_identity).err().unwrap().kind,
        ProviderErrorKind::Cancelled
    );

    read.finish();
    assert_eq!(client.join().unwrap().unwrap().payload, "(;SZ[19])");
}

#[test]
fn beginning_sync_preserves_active_preview_read_and_preview_only_cancel() {
    let _env = EnvGuard::new();
    let state = NetworkState::default();
    let revision = state.snapshot().policy_revision;
    let identity = state.begin(revision, 9).unwrap();
    let preview = state.operation(&identity).unwrap();
    let read = ControlledRead::new();
    let client = read.start(preview.clone());

    let sync = state.begin_sync(revision, 9).unwrap();
    assert!(sync.identity().request_id > identity.request_id);
    assert_eq!(
        preview.lease().with_valid(|| "preview commit").unwrap(),
        "preview commit"
    );
    state.cancel(&sync.identity());
    assert_eq!(sync.lease().with_valid(|| "sync commit").unwrap(), "sync commit");

    read.finish();
    assert_eq!(client.join().unwrap().unwrap().payload, "(;SZ[19])");
    state.cancel(&identity);
    assert_eq!(
        preview.lease().check().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    assert!(sync.lease().check().is_ok());
}

#[test]
fn replacing_sync_cancels_old_read_and_lease_without_sealing_preview() {
    let _env = EnvGuard::new();
    let state = NetworkState::default();
    let revision = state.snapshot().policy_revision;
    let preview_identity = state.begin(revision, 11).unwrap();
    let preview = state.operation(&preview_identity).unwrap();
    let old = state.begin_sync(revision, 11).unwrap();
    let old_read = ControlledRead::new();
    let old_client = old_read.start(old.clone());

    let replacement = state.begin_sync(revision, 11).unwrap();
    assert!(replacement.identity().request_id > old.identity().request_id);
    assert_eq!(
        old.lease()
            .with_valid(|| panic!("stale sync committed"))
            .unwrap_err()
            .kind,
        ProviderErrorKind::Cancelled
    );
    assert!(state.operation(&preview_identity).is_ok());
    assert!(preview.lease().with_valid(|| ()).is_ok());
    assert_eq!(
        old_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    old_read.finish();

    let new_read = ControlledRead::new();
    let new_client = new_read.start(replacement.clone());
    new_read.finish();
    assert_eq!(new_client.join().unwrap().unwrap().payload, "(;SZ[19])");
    replacement.cancel();
    assert_eq!(
        replacement
            .lease()
            .with_valid(|| panic!("cancelled sync committed"))
            .unwrap_err()
            .kind,
        ProviderErrorKind::Cancelled
    );
    assert!(preview.lease().check().is_ok());
}

#[test]
fn policy_commit_cancels_both_active_reads_and_rejects_late_commits() {
    let _env = EnvGuard::new();
    let state = NetworkState::default();
    let revision = state.snapshot().policy_revision;
    let identity = state.begin(revision, 13).unwrap();
    let preview = state.operation(&identity).unwrap();
    let sync = state.begin_sync(revision, 13).unwrap();
    let preview_read = ControlledRead::new();
    let sync_read = ControlledRead::new();
    let preview_client = preview_read.start(preview.clone());
    let sync_client = sync_read.start(sync.clone());

    assert!(state
        .commit(NetworkSettingsDto::default(), || Err("disk full".into()))
        .is_err());
    assert!(preview.lease().with_valid(|| ()).is_ok());
    assert!(sync.lease().with_valid(|| ()).is_ok());
    let snapshot = state.commit(NetworkSettingsDto::default(), || Ok(())).unwrap();
    assert!(snapshot.policy_revision > revision);
    for operation in [&preview, &sync] {
        assert_eq!(
            operation
                .lease()
                .with_valid(|| panic!("old policy committed"))
                .unwrap_err()
                .kind,
            ProviderErrorKind::Cancelled
        );
    }
    assert_eq!(
        preview_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    assert_eq!(
        sync_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    preview_read.finish();
    sync_read.finish();
    assert_eq!(
        state.begin_sync(revision, 13).err().unwrap().kind,
        ProviderErrorKind::Cancelled
    );
    assert!(state.begin_sync(snapshot.policy_revision, 13).is_ok());
}

#[test]
fn cancel_all_seals_preview_and_sync_reads_and_late_commit_leases() {
    let _env = EnvGuard::new();
    let state = NetworkState::default();
    let revision = state.snapshot().policy_revision;
    let identity = state.begin(revision, 15).unwrap();
    let preview = state.operation(&identity).unwrap();
    let sync = state.begin_sync(revision, 15).unwrap();
    let preview_read = ControlledRead::new();
    let sync_read = ControlledRead::new();
    let preview_client = preview_read.start(preview.clone());
    let sync_client = sync_read.start(sync.clone());

    state.cancel_all();
    for operation in [&preview, &sync] {
        assert_eq!(
            operation
                .lease()
                .with_valid(|| panic!("cancelled read committed"))
                .unwrap_err()
                .kind,
            ProviderErrorKind::Cancelled
        );
    }
    assert_eq!(
        preview_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    assert_eq!(
        sync_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    preview_read.finish();
    sync_read.finish();
    assert!(state.shutdown(Duration::from_secs(1)));
}

#[test]
fn shutdown_drains_active_preview_and_sync_with_one_bounded_budget() {
    let _env = EnvGuard::new();
    let state = NetworkState::default();
    let revision = state.snapshot().policy_revision;
    let identity = state.begin(revision, 17).unwrap();
    let preview = state.operation(&identity).unwrap();
    let sync = state.begin_sync(revision, 17).unwrap();
    let preview_read = ControlledRead::new();
    let sync_read = ControlledRead::new();
    let preview_client = preview_read.start(preview.clone());
    let sync_client = sync_read.start(sync.clone());

    let start = Instant::now();
    assert!(state.shutdown(Duration::from_secs(1)), "both reads must drain");
    assert!(start.elapsed() < Duration::from_secs(1));
    for operation in [&preview, &sync] {
        assert_eq!(
            operation
                .lease()
                .with_valid(|| panic!("shutdown read committed"))
                .unwrap_err()
                .kind,
            ProviderErrorKind::Cancelled
        );
    }
    assert_eq!(
        preview_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    assert_eq!(
        sync_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    preview_read.finish();
    sync_read.finish();
    assert_eq!(
        state.begin(revision, 17).unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    assert_eq!(
        state.begin_sync(revision, 17).err().unwrap().kind,
        ProviderErrorKind::Cancelled
    );
}

#[test]
fn shutdown_lease_timeout_seals_other_lane_and_retry_drains_retained_read() {
    let _env = EnvGuard::new();
    let state = std::sync::Arc::new(NetworkState::default());
    let revision = state.snapshot().policy_revision;
    let identity = state.begin(revision, 19).unwrap();
    let preview = state.operation(&identity).unwrap();
    let sync = state.begin_sync(revision, 19).unwrap();
    let preview_read = ControlledRead::new();
    let sync_read = ControlledRead::new();
    let preview_client = preview_read.start(preview.clone());
    let sync_client = sync_read.start(sync.clone());
    let held = preview.lease();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let install = std::thread::spawn(move || {
        held.with_valid(|| {
            entered_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
        })
    });
    entered_rx.recv_timeout(Duration::from_secs(3)).unwrap();

    let stopping = state.clone();
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    let stop = std::thread::spawn(move || {
        done_tx
            .send(stopping.shutdown(Duration::from_millis(40)))
            .unwrap();
    });
    let bounded = done_rx.recv_timeout(Duration::from_millis(500));
    let other_lane = sync.lease().with_valid(|| "late commit");
    release_tx.send(()).unwrap();
    install.join().unwrap().unwrap();
    stop.join().unwrap();
    assert_eq!(bounded, Ok(false), "held lease must use the same shutdown budget");
    assert_eq!(other_lane.unwrap_err().kind, ProviderErrorKind::Cancelled);
    assert!(
        state.shutdown(Duration::from_secs(1)),
        "retry must drain retained activity"
    );
    assert_eq!(
        preview_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    assert_eq!(
        sync_client.join().unwrap().unwrap_err().kind,
        ProviderErrorKind::Cancelled
    );
    preview_read.finish();
    sync_read.finish();
    assert_eq!(
        preview
            .lease()
            .with_valid(|| panic!("late preview committed"))
            .unwrap_err()
            .kind,
        ProviderErrorKind::Cancelled
    );
}
