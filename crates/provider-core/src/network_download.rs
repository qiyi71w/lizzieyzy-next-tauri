//! Binary downloads share NET-01 route resolution, cancellation and shutdown.
use super::*;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

struct Download {
    file: File,
    bytes: u64,
    limit: u64,
    location: Option<String>,
    activity: Instant,
    failed: bool,
}
impl Handler for Download {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.activity = Instant::now();
        if self.bytes.saturating_add(data.len() as u64) > self.limit || self.file.write_all(data).is_err() {
            self.failed = true;
            return Ok(0);
        }
        self.bytes += data.len() as u64;
        Ok(data.len())
    }
    fn header(&mut self, data: &[u8]) -> bool {
        self.activity = Instant::now();
        if data.starts_with(b"HTTP/") {
            self.location = None;
        }
        if let Ok(line) = std::str::from_utf8(data) {
            if let Some((key, value)) = line.trim_end().split_once(':') {
                if key.eq_ignore_ascii_case("location") {
                    if value.len() > 8192 {
                        return false;
                    }
                    self.location = Some(value.trim().into());
                }
            }
        }
        true
    }
}

impl NetworkOperation {
    /// No automatic retry or endpoint fallback. Every redirect re-resolves the selected policy
    /// and must remain in the caller's trusted origin set. Bytes are streamed, never a UTF-8 payload.
    pub fn download(
        &self,
        source: &str,
        destination: &Path,
        max_bytes: u64,
        allowed_origins: &[&str],
        mut progress: impl FnMut(u64),
    ) -> ProviderResult<()> {
        self.lease.check()?;
        {
            let mut active = self.activity.state.lock().expect("network activity");
            if active.0 {
                return Err(cancelled());
            }
            active.1 += 1;
        }
        let _active = ActiveRead(&self.activity);
        let mut url = http_url(source)?;
        let deadline = Instant::now() + Duration::from_secs(1800);
        for redirect in 0..=10 {
            self.lease.check()?;
            if !allowed_origins.contains(&url.origin().ascii_serialization().as_str()) {
                return Err(invalid_url("Resource download origin is not admitted."));
            }
            let route = resolve_route(
                &self.settings,
                &url,
                &self.lease.0.cancelled,
                Instant::now() + Duration::from_secs(20),
            )?;
            let observation = NetworkRouteDto {
                mode: self.settings.mode,
                source: route.source.into(),
                target: authority(&url),
                proxy: route
                    .proxy
                    .as_deref()
                    .and_then(|value| Url::parse(value).ok())
                    .map(|value| authority(&value)),
            };
            // At most 11 redirects per file; retain bounded route observations across the operation.
            let mut routes = self.routes.lock().expect("network routes");
            if routes.len() == 32 {
                routes.remove(0);
            }
            routes.push(observation);
            drop(routes);
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(destination)
                .map_err(|_| invalid_request("Resource staging file cannot be written."))?;
            let mut easy = Easy2::new(Download {
                file,
                bytes: 0,
                limit: max_bytes,
                location: None,
                activity: Instant::now(),
                failed: false,
            });
            let configure = |_| invalid_request("Resource transport configuration failed.");
            easy.url(url.as_str()).map_err(configure)?;
            easy.follow_location(false).map_err(configure)?;
            easy.proxy(route.proxy.as_deref().unwrap_or(""))
                .map_err(configure)?;
            easy.noproxy("").map_err(configure)?;
            easy.connect_timeout(Duration::from_secs(20)).map_err(configure)?;
            easy.timeout(deadline.saturating_duration_since(Instant::now()))
                .map_err(configure)?;
            if route.proxy.is_some() {
                let mut auth = Auth::new();
                auth.gssnegotiate(true);
                auth.ntlm(true);
                easy.proxy_auth(&auth).map_err(configure)?;
                easy.proxy_username("").map_err(configure)?;
                easy.proxy_password("").map_err(configure)?;
            }
            let multi = Multi::new();
            let runtime_error = |_| {
                provider_error(
                    ProviderErrorKind::TransportFailed,
                    "Resource transfer runtime failed.",
                )
            };
            let handle = multi.add2(easy).map_err(runtime_error)?;
            loop {
                self.lease.check()?;
                if Instant::now() >= deadline
                    || handle.get_ref().activity.elapsed() >= Duration::from_secs(25)
                {
                    return Err(crate::timeout(
                        "Resource download timed out; check connectivity and selected policy.",
                    ));
                }
                let running = multi.perform().map_err(runtime_error)?;
                progress(handle.get_ref().bytes);
                let mut completed = None;
                multi.messages(|message| {
                    if let Some(result) = message.result_for2(&handle) {
                        completed = Some(result);
                    }
                });
                if let Some(result) = completed {
                    if handle.get_ref().failed {
                        return Err(invalid_request(
                            "Resource exceeds admitted size or staging write failed.",
                        ));
                    }
                    result.map_err(|error| provider_error(
                        if route.proxy.is_some() { ProviderErrorKind::ProxyFailed } else { ProviderErrorKind::TransportFailed },
                        format!("Resource unavailable or offline (transport code {}); selected policy was not changed.", error.code()),
                    ))?;
                    break;
                }
                if running == 0 {
                    return Err(provider_error(
                        ProviderErrorKind::TransportFailed,
                        "Resource transfer ended without response.",
                    ));
                }
                multi
                    .wait(&mut [], Duration::from_millis(50))
                    .map_err(runtime_error)?;
            }
            let mut easy = multi.remove2(handle).map_err(runtime_error)?;
            let status = easy.response_code().map_err(configure)?;
            if matches!(status, 301 | 302 | 303 | 307 | 308) {
                if redirect == 10 {
                    return Err(invalid_url("Resource redirect limit exceeded."));
                }
                let next = url
                    .join(
                        easy.get_ref()
                            .location
                            .as_deref()
                            .ok_or_else(|| invalid_url("Resource redirect has no location."))?,
                    )
                    .map_err(|_| invalid_url("Invalid resource redirect."))?;
                if url.scheme() == "https" && next.scheme() != "https" {
                    return Err(invalid_url("Resource HTTPS downgrade refused."));
                }
                url = http_url(next.as_str())?;
                continue;
            }
            if status != 200 {
                return Err(provider_error(
                    if matches!(status, 404 | 410) {
                        ProviderErrorKind::NotFound
                    } else {
                        ProviderErrorKind::TransportFailed
                    },
                    format!("Resource unavailable (HTTP {status})."),
                ));
            }
            self.lease.check()?;
            easy.get_mut()
                .file
                .sync_all()
                .map_err(|_| invalid_request("Resource staging sync failed."))?;
            return Ok(());
        }
        unreachable!()
    }
}
