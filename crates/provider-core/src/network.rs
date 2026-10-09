//! The one outbound provider network owner. A lease fences fetch, preview and document commit.
use crate::{invalid_request, invalid_url, provider_error, ProviderResult, ProviderTransport};
use app_model::{
    NetworkModeDto, NetworkRouteDto, NetworkSettingsDto, NetworkSnapshotDto, ProviderErrorKind,
    ProviderFetchMethod, ProviderFetchRequest, ProviderFetchResult, ProviderGameMetadata, ProviderKind,
    ProviderRequestIdentityDto,
};
use curl::easy::{Auth, Easy2, Handler, List, WriteError};
use curl::multi::Multi;
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::time::{Duration, Instant};
use url::Url;

#[path = "network_download.rs"]
mod download;

#[cfg(windows)]
#[path = "platform/windows.rs"]
mod platform;
#[cfg(target_os = "macos")]
#[path = "platform/macos.rs"]
mod platform;

#[derive(Clone, Debug)]
pub(crate) struct Route {
    pub proxy: Option<String>,
    pub source: &'static str,
}

#[derive(Default)]
pub struct NetworkState(Mutex<PolicyState>, Arc<Activity>);
#[derive(Default)]
struct PolicyState {
    settings: NetworkSettingsDto,
    revision: u64,
    sequence: u64,
    current: Option<NetworkOperation>,
    sync: Option<NetworkOperation>,
    resource: Option<NetworkOperation>,
}

#[derive(Default)]
struct Activity {
    state: Mutex<(bool, usize)>,
    drained: Condvar,
}
struct ActiveRead<'a>(&'a Activity);
impl Drop for ActiveRead<'_> {
    fn drop(&mut self) {
        self.0.state.lock().expect("network activity").1 -= 1;
        self.0.drained.notify_all();
    }
}

#[derive(Clone)]
pub struct NetworkLease(Arc<LeaseState>);
struct LeaseState {
    valid: Mutex<bool>,
    cancelled: AtomicBool,
}
impl NetworkLease {
    fn new() -> Self {
        Self(Arc::new(LeaseState {
            valid: Mutex::new(true),
            cancelled: AtomicBool::new(false),
        }))
    }
    fn cancel(&self) {
        // Commit and cancellation have one linearization point, not a check-then-install race.
        *self.0.valid.lock().expect("network lease") = false;
        self.0.cancelled.store(true, Ordering::Release);
    }
    pub fn with_valid<T>(&self, action: impl FnOnce() -> T) -> ProviderResult<T> {
        let valid = self.0.valid.lock().expect("network lease");
        if !*valid || self.0.cancelled.load(Ordering::Acquire) {
            return Err(cancelled());
        }
        Ok(action())
    }
    pub fn check(&self) -> ProviderResult<()> {
        self.with_valid(|| ())
    }
}

fn lock_before<T>(mutex: &Mutex<T>, deadline: Instant) -> Option<std::sync::MutexGuard<'_, T>> {
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return None;
        }
        match mutex.try_lock() {
            Ok(guard) => return Some(guard),
            Err(std::sync::TryLockError::WouldBlock) => {
                std::thread::sleep(remaining.min(Duration::from_millis(1)));
            }
            Err(std::sync::TryLockError::Poisoned(error)) => panic!("network lock poisoned: {error}"),
        }
    }
}

impl NetworkState {
    pub fn snapshot(&self) -> NetworkSnapshotDto {
        let state = self.0.lock().expect("network state");
        NetworkSnapshotDto {
            settings: state.settings.clone(),
            policy_revision: state.revision,
        }
    }
    /// Call under the preferences transaction. Failed persistence cannot change policy or leases.
    pub fn commit(
        &self,
        settings: NetworkSettingsDto,
        persist: impl FnOnce() -> Result<(), String>,
    ) -> Result<NetworkSnapshotDto, String> {
        settings.validate()?;
        let mut state = self.0.lock().expect("network state");
        persist()?;
        if let Some(previous) = state.current.take() {
            previous.lease.cancel();
        }
        if let Some(previous) = state.sync.take() {
            previous.cancel();
        }
        if let Some(previous) = state.resource.take() {
            previous.cancel();
        }
        state.settings = settings;
        state.revision += 1;
        Ok(NetworkSnapshotDto {
            settings: state.settings.clone(),
            policy_revision: state.revision,
        })
    }
    pub fn begin(
        &self,
        policy_revision: u64,
        document_identity: u64,
    ) -> ProviderResult<ProviderRequestIdentityDto> {
        let mut state = self.0.lock().expect("network state");
        if self.1.state.lock().expect("network activity").0 {
            return Err(cancelled());
        }
        if state.revision != policy_revision {
            return Err(cancelled());
        }
        if let Some(previous) = state.current.take() {
            previous.lease.cancel();
        }
        state.sequence += 1;
        let identity = ProviderRequestIdentityDto {
            request_id: state.sequence,
            policy_revision,
            document_identity,
        };
        state.current = Some(NetworkOperation::new(
            identity.clone(),
            state.settings.clone(),
            self.1.clone(),
        ));
        Ok(identity)
    }
    pub fn begin_sync(
        &self,
        policy_revision: u64,
        document_identity: u64,
    ) -> ProviderResult<NetworkOperation> {
        let mut state = self.0.lock().expect("network state");
        if self.1.state.lock().expect("network activity").0 || state.revision != policy_revision {
            return Err(cancelled());
        }
        if let Some(previous) = state.sync.take() {
            previous.cancel();
        }
        state.sequence += 1;
        let operation = NetworkOperation::new(
            ProviderRequestIdentityDto {
                request_id: state.sequence,
                policy_revision,
                document_identity,
            },
            state.settings.clone(),
            self.1.clone(),
        );
        state.sync = Some(operation.clone());
        Ok(operation)
    }
    /// Independent managed-download lane; provider/document reads cannot retire it.
    pub fn begin_resource(&self, policy_revision: u64) -> ProviderResult<NetworkOperation> {
        let mut state = self.0.lock().expect("network state");
        if self.1.state.lock().expect("network activity").0 || state.revision != policy_revision {
            return Err(cancelled());
        }
        if let Some(previous) = state.resource.take() {
            previous.cancel();
        }
        state.sequence += 1;
        let operation = NetworkOperation::new(
            ProviderRequestIdentityDto { request_id: state.sequence, policy_revision, document_identity: 0 },
            state.settings.clone(), self.1.clone(),
        );
        state.resource = Some(operation.clone());
        Ok(operation)
    }

    pub fn operation(&self, identity: &ProviderRequestIdentityDto) -> ProviderResult<NetworkOperation> {
        let state = self.0.lock().expect("network state");
        let operation = state
            .current
            .as_ref()
            .filter(|op| &op.identity == identity)
            .ok_or_else(cancelled)?;
        operation.lease.check()?;
        Ok(operation.clone())
    }
    pub fn cancel(&self, identity: &ProviderRequestIdentityDto) {
        let state = self.0.lock().expect("network state");
        if let Some(op) = state.current.as_ref().filter(|op| &op.identity == identity) {
            op.lease.cancel();
        }
    }
    pub fn cancel_all(&self) {
        let mut state = self.0.lock().expect("network state");
        if let Some(op) = state.current.take() {
            op.cancel();
        }
        if let Some(op) = state.sync.take() {
            op.cancel();
        }
        if let Some(op) = state.resource.take() {
            op.cancel();
        }
    }
    /// Seal both lanes and drain all generations within the shared exit budget.
    pub fn shutdown(&self, budget: Duration) -> bool {
        let deadline = Instant::now() + budget;
        let Some(mut active) = lock_before(&self.1.state, deadline) else {
            return false;
        };
        active.0 = true;
        drop(active);
        {
            let Some(mut state) = lock_before(&self.0, deadline) else {
                return false;
            };
            // Seal both reads before waiting for either in-flight commit lease.
            for op in [&state.current, &state.sync, &state.resource].into_iter().flatten() {
                op.lease.0.cancelled.store(true, Ordering::Release);
            }
            let PolicyState { current, sync, resource, .. } = &mut *state;
            for slot in [current, sync, resource] {
                if let Some(op) = slot.as_ref() {
                    let Some(mut valid) = lock_before(&op.lease.0.valid, deadline) else {
                        // Retain this and later slots so a shutdown retry can finish sealing them.
                        return false;
                    };
                    *valid = false;
                }
                *slot = None;
            }
        }
        let Some(mut active) = lock_before(&self.1.state, deadline) else {
            return false;
        };
        while active.1 != 0 {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }
            active = self
                .1
                .drained
                .wait_timeout(active, remaining)
                .expect("network drain")
                .0;
        }
        Instant::now() <= deadline
    }
}

#[derive(Clone)]
pub struct NetworkOperation {
    identity: ProviderRequestIdentityDto,
    settings: NetworkSettingsDto,
    lease: NetworkLease,
    retries: Arc<Mutex<u8>>,
    routes: Arc<Mutex<Vec<NetworkRouteDto>>>,
    terminal: Arc<Mutex<Option<app_model::ProviderError>>>,
    activity: Arc<Activity>,
}
impl NetworkOperation {
    fn new(
        identity: ProviderRequestIdentityDto,
        settings: NetworkSettingsDto,
        activity: Arc<Activity>,
    ) -> Self {
        Self {
            identity,
            settings,
            lease: NetworkLease::new(),
            retries: Arc::new(Mutex::new(3)),
            routes: Arc::default(),
            terminal: Arc::default(),
            activity,
        }
    }
    pub fn identity(&self) -> ProviderRequestIdentityDto {
        self.identity.clone()
    }
    pub fn cancel(&self) {
        self.lease.cancel();
    }
    pub fn retry_count(&self) -> u8 {
        3 - *self.retries.lock().expect("network retry budget")
    }
    pub fn lease(&self) -> NetworkLease {
        self.lease.clone()
    }
    pub fn routes(&self) -> Vec<NetworkRouteDto> {
        self.routes.lock().expect("network routes").clone()
    }
    fn retry(&self) -> bool {
        let mut remaining = self.retries.lock().expect("network retry budget");
        if *remaining == 0 {
            false
        } else {
            *remaining -= 1;
            true
        }
    }
    fn fetch_reads(&self, requests: &[ProviderFetchRequest]) -> ProviderResult<ProviderFetchResult> {
        self.lease.check()?;
        {
            let mut active = self.activity.state.lock().expect("network activity");
            if active.0 {
                return Err(cancelled());
            }
            active.1 += 1;
        }
        let _active = ActiveRead(&self.activity);
        if requests.is_empty() {
            return Err(invalid_request("No provider read endpoint was supplied."));
        }
        if let Some(error) = self.terminal.lock().expect("network terminal state").as_ref() {
            return Err(error.clone());
        }
        let mut endpoint = 0;
        loop {
            let request = &requests[endpoint];
            let idempotent = request.method == ProviderFetchMethod::Get
                || (request.provider == ProviderKind::Fox
                    && [
                        "http://happyapp.huanle.qq.com/cgi-bin/CommonMobileCGI/TXWQFetchChess",
                        "http://cgi.foxwq.com/cgi-bin/CommonMobileCGI/TXWQFetchChess",
                    ]
                    .contains(&request.url.as_str()));
            let result = self.fetch_once(request);
            match &result {
                Err(error)
                    if idempotent
                        && matches!(
                            error.kind,
                            ProviderErrorKind::TransportFailed | ProviderErrorKind::Timeout
                        )
                        && self.retry() =>
                {
                    endpoint = (endpoint + 1).min(requests.len() - 1);
                    for _ in 0..5 {
                        self.lease.check()?;
                        std::thread::sleep(Duration::from_millis(20));
                    }
                }
                _ => {
                    if let Err(error) = &result {
                        *self.terminal.lock().expect("network terminal state") = Some(error.clone());
                    }
                    return result;
                }
            }
        }
    }

    fn fetch_once(&self, request: &ProviderFetchRequest) -> ProviderResult<ProviderFetchResult> {
        let mut url = http_url(&request.url)?;
        let mut method = request.method;
        let mut headers = request.headers.clone();
        let started = Instant::now();
        let deadline = (request.provider == ProviderKind::Yike).then(|| started + Duration::from_secs(10));
        for redirect in 0..=10 {
            self.lease.check()?;
            let resolve_deadline = deadline.unwrap_or_else(|| Instant::now() + Duration::from_secs(20));
            let route = resolve_route(&self.settings, &url, &self.lease.0.cancelled, resolve_deadline)?;
            let observation = NetworkRouteDto {
                mode: self.settings.mode,
                source: route.source.into(),
                target: authority(&url),
                proxy: route
                    .proxy
                    .as_deref()
                    .and_then(|p| Url::parse(p).ok())
                    .map(|p| authority(&p)),
            };
            self.routes
                .lock()
                .expect("network routes")
                .push(observation.clone());
            let response = transfer(
                &url,
                method,
                &headers,
                request.body.as_deref(),
                &route,
                &self.lease,
                deadline,
            )
            .map_err(|mut error| {
                error.message = format!(
                    "{} [{}; {}; {}]",
                    error.message,
                    route.source,
                    observation.target,
                    observation.proxy.as_deref().unwrap_or("DIRECT")
                );
                error
            })?;
            if matches!(response.status, 301 | 302 | 303 | 307 | 308) {
                if redirect == 10 {
                    return Err(invalid_url("Provider redirect limit exceeded."));
                }
                let location = response
                    .headers
                    .get("location")
                    .ok_or_else(|| invalid_url("Redirect has no location."))?;
                let next = url
                    .join(location)
                    .map_err(|_| invalid_url("Invalid redirect destination."))?;
                let next = http_url(next.as_str())?;
                if url.scheme() == "https" && next.scheme() != "https" {
                    return Err(provider_error(
                        ProviderErrorKind::TlsFailed,
                        "HTTPS downgrade redirect refused.",
                    ));
                }
                if next.origin() != url.origin() {
                    // Signed provider headers belong to the original origin only.
                    headers.retain(|name, _| {
                        matches!(
                            name.to_ascii_lowercase().as_str(),
                            "accept" | "user-agent" | "content-type"
                        )
                    });
                }
                if response.status == 303
                    || (matches!(response.status, 301 | 302) && method == ProviderFetchMethod::Post)
                {
                    method = ProviderFetchMethod::Get;
                }
                url = next;
                continue;
            }
            let kind = match response.status {
                401 | 403 | 407 => Some(ProviderErrorKind::AuthenticationFailed),
                404 | 410 => Some(ProviderErrorKind::NotFound),
                408 | 425 | 429 | 500 | 502 | 503 | 504 => Some(ProviderErrorKind::TransportFailed),
                400..=599 => Some(ProviderErrorKind::InvalidRequest),
                _ => None,
            };
            if let Some(kind) = kind {
                return Err(provider_error(
                    kind,
                    format!(
                        "Provider returned HTTP {} [{}; {}; {}].",
                        response.status,
                        route.source,
                        observation.target,
                        observation.proxy.as_deref().unwrap_or("DIRECT")
                    ),
                ));
            }
            self.lease.check()?;
            // Only safe response metadata crosses IPC; cookies/auth and redirect query strings do not.
            let content_type = response.headers.get("content-type").cloned();
            return Ok(ProviderFetchResult {
                provider: request.provider,
                url: sanitized_url(&url),
                status_code: response.status,
                payload: String::from_utf8(response.payload)
                    .map_err(|_| crate::invalid_payload("Provider response is not UTF-8."))?,
                headers: content_type
                    .as_ref()
                    .map(|v| [("content-type".into(), v.clone())].into())
                    .unwrap_or_default(),
                content_type,
                metadata: ProviderGameMetadata {
                    request_url: Some(sanitized_url(&http_url(&request.url)?)),
                    source_id: request.source_id.clone(),
                    ..Default::default()
                },
                warnings: Vec::new(),
            });
        }
        unreachable!()
    }
}
impl ProviderTransport for NetworkOperation {
    fn fetch(&self, request: &ProviderFetchRequest) -> ProviderResult<ProviderFetchResult> {
        self.fetch_reads(std::slice::from_ref(request))
    }
    fn fetch_alternatives(&self, requests: &[ProviderFetchRequest]) -> ProviderResult<ProviderFetchResult> {
        self.fetch_reads(requests)
    }
}

fn cancelled() -> app_model::ProviderError {
    provider_error(
        ProviderErrorKind::Cancelled,
        "Provider request cancelled or superseded; preview again.",
    )
}
fn http_url(value: &str) -> ProviderResult<Url> {
    let url = Url::parse(value).map_err(|_| invalid_url("Provider URL is invalid."))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(invalid_url("Provider URL must use HTTP(S) without credentials."));
    }
    Ok(url)
}
fn authority(url: &Url) -> String {
    format!(
        "{}:{}",
        url.host_str().unwrap_or("unknown"),
        url.port_or_known_default().unwrap_or(0)
    )
}
fn sanitized_url(url: &Url) -> String {
    let mut safe = url.clone();
    safe.set_query(None);
    safe.set_fragment(None);
    safe.to_string()
}
fn env_value(lower: &str, upper: &str) -> Option<String> {
    std::env::var(lower)
        .ok()
        .or_else(|| std::env::var(upper).ok())
        .filter(|v| !v.is_empty())
}
fn resolve_route(
    settings: &NetworkSettingsDto,
    url: &Url,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> ProviderResult<Route> {
    if settings.mode == NetworkModeDto::Direct {
        return Ok(Route {
            proxy: None,
            source: "direct",
        });
    }
    if env_value("no_proxy", "NO_PROXY").is_some_and(|list| bypass(url, &list)) {
        return Ok(Route {
            proxy: None,
            source: "no-proxy",
        });
    }
    if settings.mode == NetworkModeDto::Manual {
        let host = if settings.manual_host.contains(':') && !settings.manual_host.starts_with('[') {
            format!("[{}]", settings.manual_host)
        } else {
            settings.manual_host.clone()
        };
        return Ok(Route {
            proxy: Some(format!("http://{host}:{}", settings.manual_port)),
            source: "manual",
        });
    }
    let configured = if url.scheme() == "https" {
        env_value("https_proxy", "HTTPS_PROXY")
    } else {
        env_value("http_proxy", "HTTP_PROXY")
    }
    .or_else(|| env_value("all_proxy", "ALL_PROXY"));
    if let Some(proxy) = configured {
        let endpoint = if proxy.contains("://") {
            proxy
        } else {
            format!("http://{proxy}")
        };
        let parsed = http_url(&endpoint).map_err(|_| {
            provider_error(
                ProviderErrorKind::ProxyFailed,
                "System proxy environment must name an HTTP(S) endpoint without credentials.",
            )
        })?;
        if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
            return Err(provider_error(
                ProviderErrorKind::ProxyFailed,
                "Invalid system proxy endpoint.",
            ));
        }
        return Ok(Route {
            proxy: Some(parsed.to_string()),
            source: "environment",
        });
    }
    #[cfg(any(windows, target_os = "macos"))]
    {
        platform::resolve(url.as_str(), cancelled, deadline)
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = (cancelled, deadline);
        Ok(Route {
            proxy: None,
            source: "linux-environment-direct",
        })
    }
}
fn bypass(url: &Url, list: &str) -> bool {
    let host = url
        .host_str()
        .unwrap_or_default()
        .trim_matches(['[', ']'])
        .to_ascii_lowercase();
    list.split(',')
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .any(|entry| {
            if entry == "*" {
                return true;
            }
            let (name, port) = if entry.starts_with('[') {
                match entry.find(']') {
                    Some(end) => (&entry[1..end], entry[end + 1..].strip_prefix(':')),
                    None => (entry, None),
                }
            } else if entry.matches(':').count() == 1 {
                let (name, port) = entry.rsplit_once(':').unwrap();
                (name, Some(port))
            } else {
                (entry, None)
            };
            if port.is_some_and(|p| p.parse::<u16>().ok() != url.port_or_known_default()) {
                return false;
            }
            if let Some((network, bits)) = name.split_once('/') {
                if let (Ok(ip), Ok(net), Ok(bits)) = (
                    host.parse::<std::net::IpAddr>(),
                    network.parse::<std::net::IpAddr>(),
                    bits.parse::<u32>(),
                ) {
                    return match (ip, net) {
                        (std::net::IpAddr::V4(ip), std::net::IpAddr::V4(net)) if bits <= 32 => {
                            bits == 0 || (u32::from(ip) >> (32 - bits)) == (u32::from(net) >> (32 - bits))
                        }
                        (std::net::IpAddr::V6(ip), std::net::IpAddr::V6(net)) if bits <= 128 => {
                            bits == 0 || (u128::from(ip) >> (128 - bits)) == (u128::from(net) >> (128 - bits))
                        }
                        _ => false,
                    };
                }
            }
            let name = name.trim_start_matches('.').to_ascii_lowercase();
            host == name || host.ends_with(&format!(".{name}"))
        })
}

struct Response {
    status: u16,
    headers: BTreeMap<String, String>,
    payload: Vec<u8>,
    activity: Instant,
}
impl Handler for Response {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.activity = Instant::now();
        self.payload.extend_from_slice(data);
        Ok(data.len())
    }
    fn header(&mut self, data: &[u8]) -> bool {
        self.activity = Instant::now();
        if data.starts_with(b"HTTP/") {
            self.headers.clear();
        }
        if let Ok(line) = std::str::from_utf8(data) {
            if let Some((key, value)) = line.trim_end().split_once(':') {
                if matches!(key.to_ascii_lowercase().as_str(), "location" | "content-type") {
                    self.headers.insert(key.to_ascii_lowercase(), value.trim().into());
                }
            }
        }
        true
    }
}
fn transfer(
    url: &Url,
    method: ProviderFetchMethod,
    headers: &BTreeMap<String, String>,
    body: Option<&str>,
    route: &Route,
    lease: &NetworkLease,
    deadline: Option<Instant>,
) -> ProviderResult<Response> {
    let started = Instant::now();
    let mut easy = Easy2::new(Response {
        status: 0,
        headers: BTreeMap::new(),
        payload: Vec::new(),
        activity: started,
    });
    let configure = |error: curl::Error| {
        invalid_request(format!(
            "Provider request configuration failed (code {}).",
            error.code()
        ))
    };
    easy.url(url.as_str()).map_err(configure)?;
    easy.follow_location(false).map_err(configure)?;
    easy.proxy(route.proxy.as_deref().unwrap_or(""))
        .map_err(configure)?;
    easy.noproxy("").map_err(configure)?; // Resolver is authoritative, never libcurl's environment.
    easy.connect_timeout(Duration::from_secs(20)).map_err(configure)?;
    if let Some(deadline) = deadline {
        let remaining = deadline.saturating_duration_since(started);
        if remaining.is_zero() {
            return Err(crate::timeout("Yike request deadline exceeded."));
        }
        easy.timeout(remaining).map_err(configure)?;
    }
    if route.proxy.is_some() {
        // Empty credentials select OS SSPI/GSS credentials, never a stored password.
        let mut auth = Auth::new();
        auth.gssnegotiate(true);
        auth.ntlm(true);
        easy.proxy_auth(&auth).map_err(configure)?;
        easy.proxy_username("").map_err(configure)?;
        easy.proxy_password("").map_err(configure)?;
    }
    let mut list = List::new();
    for (key, value) in headers {
        if key.contains(['\r', '\n', ':']) || value.contains(['\r', '\n']) {
            return Err(invalid_request("Invalid provider header."));
        }
        if matches!(
            key.to_ascii_lowercase().as_str(),
            "proxy-authorization" | "proxy-connection"
        ) {
            return Err(invalid_request(
                "Proxy authentication is managed by the platform.",
            ));
        }
        list.append(&format!("{key}: {value}")).map_err(configure)?;
    }
    easy.http_headers(list).map_err(configure)?;
    if method == ProviderFetchMethod::Post {
        easy.post(true).map_err(configure)?;
        easy.post_fields_copy(body.unwrap_or_default().as_bytes())
            .map_err(configure)?;
    }
    let multi = Multi::new();
    let multi_error = |_| {
        provider_error(
            ProviderErrorKind::TransportFailed,
            "Provider transfer runtime failed.",
        )
    };
    let handle = multi.add2(easy).map_err(multi_error)?;
    let mut connected = None;
    loop {
        lease.check()?;
        if deadline.is_some_and(|d| Instant::now() >= d) {
            return Err(crate::timeout("Yike request deadline exceeded."));
        }
        let running = multi.perform().map_err(multi_error)?;
        let mut completed = None;
        multi.messages(|message| {
            if let Some(result) = message.result_for2(&handle) {
                completed = Some(result);
            }
        });
        if let Some(result) = completed {
            if let Err(error) = result {
                let connect_code = handle.http_connectcode().unwrap_or_default();
                let kind = if connect_code == 407 {
                    ProviderErrorKind::AuthenticationFailed
                } else {
                    match error.code() {
                        28 => ProviderErrorKind::Timeout,
                        35 | 51 | 58 | 60 | 64 | 66 | 77 | 80 | 82 | 83 | 90 | 91 => {
                            ProviderErrorKind::TlsFailed
                        }
                        5 | 7 if route.proxy.is_some() => ProviderErrorKind::ProxyFailed,
                        5 | 6 | 7 | 18 | 52 | 55 | 56 => ProviderErrorKind::TransportFailed,
                        _ => ProviderErrorKind::InvalidRequest,
                    }
                };
                return Err(provider_error(kind, format!("Provider transport failed (code {}); check network, proxy authentication or system trust.", error.code())));
            }
            break;
        }
        if handle.pretransfer_time().unwrap_or_default() > Duration::ZERO {
            let connected_at = *connected.get_or_insert_with(Instant::now);
            if Instant::now().duration_since(handle.get_ref().activity.max(connected_at))
                >= Duration::from_secs(25)
            {
                return Err(crate::timeout("Provider read timed out after 25 seconds."));
            }
        }
        if running == 0 {
            return Err(provider_error(
                ProviderErrorKind::TransportFailed,
                "Provider transfer ended without a response.",
            ));
        }
        multi
            .wait(&mut [], Duration::from_millis(50))
            .map_err(multi_error)?;
    }
    let mut easy = multi.remove2(handle).map_err(multi_error)?;
    let status = easy.response_code().map_err(configure)? as u16;
    let capture = easy.get_mut();
    Ok(Response {
        status,
        headers: std::mem::take(&mut capture.headers),
        payload: std::mem::take(&mut capture.payload),
        activity: capture.activity,
    })
}
