//! Windows native per-URL proxy resolution using WinHTTP.
//!
//! Provides per-URL proxy resolution by querying current user WinINet/IE configuration,
//! executing PAC/WPAD scripts asynchronously via WinHTTP, and falling back to static
//! system proxy settings or direct connections.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use super::Route;
use crate::{provider_error, runtime_unavailable, timeout};
use app_model::ProviderErrorKind;

/// Resolves the proxy route for a given target URL using Windows WinHTTP APIs.
pub(crate) fn resolve(url: &str, cancelled: &AtomicBool, deadline: Instant) -> crate::ProviderResult<Route> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(provider_error(
            ProviderErrorKind::Cancelled,
            "proxy resolution cancelled",
        ));
    }
    if Instant::now() >= deadline {
        return Err(timeout("proxy resolution timed out"));
    }

    let parsed_url = parse_target_url(url)?;

    #[cfg(target_os = "windows")]
    {
        winhttp_resolve(&parsed_url, cancelled, deadline)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = parsed_url;
        Err(runtime_unavailable(
            "Windows proxy resolver is only supported on Windows",
        ))
    }
}

// ---------------------------------------------------------------------------
// Pure Rust URL parsing & sanitization (portable across platforms & testable)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedTargetUrl {
    scheme: String,
    host: String,
    port: u16,
    sanitized_url: String,
}

fn parse_target_url(raw: &str) -> crate::ProviderResult<ParsedTargetUrl> {
    let raw = raw.trim();
    let (scheme, rest) = raw
        .split_once("://")
        .ok_or_else(|| crate::invalid_url("invalid request url"))?;

    let scheme = scheme.to_ascii_lowercase();
    if scheme != "http" && scheme != "https" {
        return Err(crate::invalid_url("invalid request url"));
    }

    let path_start = rest
        .find(|c| c == '/' || c == '?' || c == '#')
        .unwrap_or(rest.len());
    let authority = &rest[..path_start];
    let path_and_query = &rest[path_start..];
    let path_and_query = if path_and_query.is_empty() {
        "/"
    } else {
        path_and_query
    };

    if authority.is_empty() {
        return Err(crate::invalid_url("invalid request url"));
    }

    // Strip credentials (user:pass@) if present.
    let host_port_str = match authority.rsplit_once('@') {
        Some((_, hp)) => hp,
        None => authority,
    };

    let (host, port) = if let Some(stripped) = host_port_str.strip_prefix('[') {
        // IPv6 literal
        let end = stripped
            .find(']')
            .ok_or_else(|| crate::invalid_url("invalid request url"))?;
        let ip_str = &stripped[..end];
        let after_bracket = &stripped[end + 1..];
        let port = if let Some(port_str) = after_bracket.strip_prefix(':') {
            port_str
                .parse::<u16>()
                .map_err(|_| crate::invalid_url("invalid request url"))?
        } else if after_bracket.is_empty() {
            if scheme == "https" {
                443
            } else {
                80
            }
        } else {
            return Err(crate::invalid_url("invalid request url"));
        };
        (format!("[{}]", ip_str.to_ascii_lowercase()), port)
    } else if let Some((h, p)) = host_port_str.split_once(':') {
        let port = p
            .parse::<u16>()
            .map_err(|_| crate::invalid_url("invalid request url"))?;
        (h.to_ascii_lowercase(), port)
    } else {
        let port = if scheme == "https" { 443 } else { 80 };
        (host_port_str.to_ascii_lowercase(), port)
    };

    if host.is_empty() {
        return Err(crate::invalid_url("invalid request url"));
    }

    let sanitized_url = format!("{scheme}://{host_port_str}{path_and_query}");

    Ok(ParsedTargetUrl {
        scheme,
        host,
        port,
        sanitized_url,
    })
}

// ---------------------------------------------------------------------------
// Static Proxy Parsing and Bypass Rules (pure Rust, testable on any OS)
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
enum StaticProxyMatch {
    Direct,
    Proxy(String),
    UnsupportedScheme,
}

fn select_static_proxy(proxy_config_str: &str, target_scheme: &str) -> StaticProxyMatch {
    // Windows proxy strings can be separated by semicolons or whitespace:
    // e.g. "http=httpproxy:8080;https=httpsproxy:8443" or "proxy.example.com:8080"
    let entries: Vec<&str> = proxy_config_str
        .split(|c: char| c == ';' || c.is_ascii_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    if entries.is_empty() {
        return StaticProxyMatch::Direct;
    }

    let mut scheme_entries: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut generic_entries = Vec::new();

    for entry in entries {
        if let Some((proto, proxy_val)) = entry.split_once('=') {
            scheme_entries.insert(proto.trim().to_ascii_lowercase(), proxy_val.trim().to_string());
        } else {
            generic_entries.push(entry.to_string());
        }
    }

    let selected_raw = if !scheme_entries.is_empty() {
        if let Some(val) = scheme_entries.get(&target_scheme.to_ascii_lowercase()) {
            Some(val.as_str())
        } else if !generic_entries.is_empty() {
            Some(generic_entries[0].as_str())
        } else {
            // In Windows, if protocol-specific proxies are configured and the requested
            // protocol is not listed, connections for that protocol are DIRECT.
            return StaticProxyMatch::Direct;
        }
    } else if !generic_entries.is_empty() {
        Some(generic_entries[0].as_str())
    } else {
        None
    };

    match selected_raw {
        Some(raw) => format_proxy_url(raw),
        None => StaticProxyMatch::Direct,
    }
}

fn format_proxy_url(raw: &str) -> StaticProxyMatch {
    let mut s = raw.trim();
    if s.is_empty() {
        return StaticProxyMatch::Direct;
    }

    let mut scheme = "http";
    if let Some((sch, rest)) = s.split_once("://") {
        let sch_lower = sch.to_ascii_lowercase();
        if sch_lower != "http" && sch_lower != "https" {
            return StaticProxyMatch::UnsupportedScheme;
        }
        scheme = if sch_lower == "https" { "https" } else { "http" };
        s = rest;
    }

    // Strip credentials if present.
    if let Some((_, rest)) = s.rsplit_once('@') {
        s = rest;
    }
    let s = s.trim_end_matches('/');
    if s.is_empty() {
        return StaticProxyMatch::Direct;
    }

    StaticProxyMatch::Proxy(format!("{scheme}://{s}"))
}

fn matches_bypass_list(bypass_str: &str, target_host: &str, target_port: u16) -> bool {
    let entries: Vec<&str> = bypass_str
        .split(|c: char| c == ';' || c.is_ascii_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    let has_no_loopback = entries.iter().any(|e| e.eq_ignore_ascii_case("<-loopback>"));
    let is_loopback_target = is_loopback_host(target_host);

    for entry in entries {
        if entry.eq_ignore_ascii_case("<-loopback>") {
            continue;
        }
        if entry.eq_ignore_ascii_case("<local>") {
            if is_loopback_target {
                if !has_no_loopback {
                    return true;
                }
            } else if is_plain_host(target_host) {
                return true;
            }
            continue;
        }

        // Rule may specify host and optional port: e.g. "*.example.com:8443" or "10.0.0.1"
        let (rule_host, rule_port) = parse_bypass_entry(entry);
        if let Some(port) = rule_port {
            if port != target_port {
                continue;
            }
        }

        if host_matches_bypass_rule(rule_host, target_host) {
            return true;
        }
    }

    false
}

fn is_plain_host(host: &str) -> bool {
    let h = host.trim_matches(['[', ']']);
    !h.contains('.') && !h.contains(':')
}

fn is_loopback_host(host: &str) -> bool {
    let h = host.trim_matches(['[', ']']);
    h.eq_ignore_ascii_case("localhost") || h == "127.0.0.1" || h == "::1"
}

fn parse_bypass_entry(entry: &str) -> (&str, Option<u16>) {
    if let Some(stripped) = entry.strip_prefix('[') {
        if let Some(end) = stripped.find(']') {
            let host = &entry[..end + 2]; // include [ and ]
            let after = &stripped[end + 1..];
            if let Some(port_str) = after.strip_prefix(':') {
                if let Ok(port) = port_str.parse::<u16>() {
                    return (host, Some(port));
                }
            }
            return (host, None);
        }
    }

    if let Some((h, p)) = entry.rsplit_once(':') {
        if !h.contains(':') {
            if let Ok(port) = p.parse::<u16>() {
                return (h, Some(port));
            }
        }
    }

    (entry, None)
}

fn host_matches_bypass_rule(rule_host: &str, target_host: &str) -> bool {
    let r = rule_host.to_ascii_lowercase();
    let h = target_host.to_ascii_lowercase();
    let r = r.trim_matches(['[', ']']);
    let h = h.trim_matches(['[', ']']);

    if r == h {
        return true;
    }

    if let Some(stripped) = r.strip_prefix("*.") {
        return h == stripped || h.ends_with(&format!(".{stripped}")) || wildcard_match(&r, h);
    }

    if let Some(stripped) = r.strip_prefix('.') {
        return h == stripped || h.ends_with(&r);
    }

    if r.contains('*') || r.contains('?') {
        return wildcard_match(&r, h);
    }

    // In Windows bypass list, "example.com" also matches subdomains like "api.example.com"
    if h.ends_with(&format!(".{r}")) {
        return true;
    }

    false
}

fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p = pattern.as_bytes();
    let t = text.as_bytes();
    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while t_idx < t.len() {
        if p_idx < p.len()
            && (p[p_idx] == b'?' || p[p_idx].to_ascii_lowercase() == t[t_idx].to_ascii_lowercase())
        {
            p_idx += 1;
            t_idx += 1;
        } else if p_idx < p.len() && p[p_idx] == b'*' {
            star_idx = Some(p_idx);
            p_idx += 1;
            match_idx = t_idx;
        } else if let Some(star) = star_idx {
            p_idx = star + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p.len() && p[p_idx] == b'*' {
        p_idx += 1;
    }

    p_idx == p.len()
}

// ---------------------------------------------------------------------------
// Route selection from PAC entries
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum PacResultEntry {
    Direct,
    Proxy {
        scheme: &'static str,
        host: String,
        port: u16,
    },
    UnsupportedScheme,
}

fn select_route_from_pac_entries(entries: &[PacResultEntry]) -> crate::ProviderResult<Route> {
    if entries.is_empty() {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "PAC resolution returned no routes",
        ));
    }

    let mut seen_unsupported_proxy = false;

    for entry in entries {
        match entry {
            PacResultEntry::Proxy { scheme, host, port } => {
                let proxy_url = format!("{scheme}://{host}:{port}");
                return Ok(Route {
                    proxy: Some(proxy_url),
                    source: "windows-pac",
                });
            }
            PacResultEntry::Direct => {
                if seen_unsupported_proxy {
                    // Contract: never failover to direct after an attempted failed / unsupported proxy.
                    return Err(provider_error(
                        ProviderErrorKind::ProxyFailed,
                        "unsupported proxy scheme",
                    ));
                }
                return Ok(Route {
                    proxy: None,
                    source: "windows-pac",
                });
            }
            PacResultEntry::UnsupportedScheme => {
                seen_unsupported_proxy = true;
            }
        }
    }

    if seen_unsupported_proxy {
        Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "unsupported proxy scheme",
        ))
    } else {
        Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "PAC resolution returned no routes",
        ))
    }
}

// ---------------------------------------------------------------------------
// Windows WinHTTP FFI and Native Implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
mod ffi {
    use std::ffi::c_void;

    pub type BOOL = i32;
    pub type DWORD = u32;
    pub type DWORD_PTR = usize;
    pub type LPVOID = *mut c_void;
    pub type LPCWSTR = *const u16;
    pub type LPWSTR = *mut u16;
    pub type PWSTR = *mut u16;
    pub type HINTERNET = *mut c_void;
    pub type HGLOBAL = *mut c_void;
    pub type INTERNET_SCHEME = i32;
    pub type INTERNET_PORT = u16;

    pub const TRUE: BOOL = 1;

    pub const ERROR_SUCCESS: DWORD = 0;
    pub const ERROR_IO_PENDING: DWORD = 997;
    pub const ERROR_OPERATION_ABORTED: DWORD = 995;
    pub const ERROR_OPERATION_CANCELLED: DWORD = 1223;
    pub const ERROR_WINHTTP_LOGIN_FAILURE: DWORD = 12015;
    pub const ERROR_WINHTTP_TIMEOUT: DWORD = 12002;

    pub const WINHTTP_ACCESS_TYPE_NO_PROXY: DWORD = 1;
    pub const WINHTTP_FLAG_ASYNC: DWORD = 0x10000000;

    pub const WINHTTP_AUTOPROXY_AUTO_DETECT: DWORD = 0x00000001;
    pub const WINHTTP_AUTOPROXY_CONFIG_URL: DWORD = 0x00000002;

    pub const WINHTTP_AUTO_DETECT_TYPE_DHCP: DWORD = 0x00000001;
    pub const WINHTTP_AUTO_DETECT_TYPE_DNS_A: DWORD = 0x00000002;

    pub const WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING: DWORD = 0x00000800;
    pub const WINHTTP_CALLBACK_STATUS_REQUEST_ERROR: DWORD = 0x00020000;
    pub const WINHTTP_CALLBACK_STATUS_GETPROXYFORURL_COMPLETE: DWORD = 0x01000000;

    pub const WINHTTP_CALLBACK_FLAG_ALL_NOTIFICATIONS: DWORD = 0xffffffff;

    pub const INTERNET_SCHEME_HTTP: INTERNET_SCHEME = 1;
    pub const INTERNET_SCHEME_HTTPS: INTERNET_SCHEME = 2;

    #[repr(C)]
    pub struct WINHTTP_CURRENT_USER_IE_PROXY_CONFIG {
        pub fAutoDetect: BOOL,
        pub lpszAutoConfigUrl: LPWSTR,
        pub lpszProxy: LPWSTR,
        pub lpszProxyBypass: LPWSTR,
    }

    #[repr(C)]
    pub struct WINHTTP_AUTOPROXY_OPTIONS {
        pub dwFlags: DWORD,
        pub dwAutoDetectFlags: DWORD,
        pub lpszAutoConfigUrl: LPCWSTR,
        pub lpvReserved: LPVOID,
        pub dwReserved: DWORD,
        pub fAutoLogonIfChallenged: BOOL,
    }

    #[repr(C)]
    pub struct WINHTTP_ASYNC_RESULT {
        pub dwResult: DWORD_PTR,
        pub dwError: DWORD,
    }

    #[repr(C)]
    pub struct WINHTTP_PROXY_RESULT_ENTRY {
        pub fProxy: BOOL,
        pub fBypass: BOOL,
        pub ProxyScheme: INTERNET_SCHEME,
        pub pwszProxy: PWSTR,
        pub ProxyPort: INTERNET_PORT,
    }

    #[repr(C)]
    pub struct WINHTTP_PROXY_RESULT {
        pub cEntries: DWORD,
        pub pEntries: *mut WINHTTP_PROXY_RESULT_ENTRY,
    }

    pub type WINHTTP_STATUS_CALLBACK = Option<
        unsafe extern "system" fn(
            hInternet: HINTERNET,
            dwContext: DWORD_PTR,
            dwInternetStatus: DWORD,
            lpvStatusInformation: LPVOID,
            dwStatusInformationLength: DWORD,
        ),
    >;

    #[link(name = "winhttp")]
    #[link(name = "kernel32")]
    extern "system" {
        pub fn WinHttpOpen(
            pszAgentW: LPCWSTR,
            dwAccessType: DWORD,
            pszProxyW: LPCWSTR,
            pszProxyBypassW: LPCWSTR,
            dwFlags: DWORD,
        ) -> HINTERNET;

        pub fn WinHttpCloseHandle(hInternet: HINTERNET) -> BOOL;

        pub fn WinHttpSetStatusCallback(
            hInternet: HINTERNET,
            lpfnInternetCallback: WINHTTP_STATUS_CALLBACK,
            dwNotificationFlags: DWORD,
            dwReserved: DWORD_PTR,
        ) -> *const c_void;

        pub fn WinHttpSetOption(hInternet: HINTERNET, option: DWORD, buffer: LPVOID, length: DWORD) -> BOOL;

        pub fn WinHttpGetIEProxyConfigForCurrentUser(
            pProxyConfig: *mut WINHTTP_CURRENT_USER_IE_PROXY_CONFIG,
        ) -> BOOL;

        pub fn WinHttpCreateProxyResolver(hSession: HINTERNET, phResolver: *mut HINTERNET) -> DWORD;

        pub fn WinHttpGetProxyForUrlEx(
            hResolver: HINTERNET,
            pcwszUrl: LPCWSTR,
            pAutoProxyOptions: *const WINHTTP_AUTOPROXY_OPTIONS,
            pContext: DWORD_PTR,
        ) -> DWORD;

        pub fn WinHttpGetProxyResult(hResolver: HINTERNET, pProxyResult: *mut WINHTTP_PROXY_RESULT) -> DWORD;

        pub fn WinHttpFreeProxyResult(pProxyResult: *mut WINHTTP_PROXY_RESULT);

        pub fn GlobalFree(hMem: HGLOBAL) -> HGLOBAL;
    }
}

#[cfg(target_os = "windows")]
use ffi::*;

#[cfg(target_os = "windows")]
struct SafeHInternet(HINTERNET);

#[cfg(target_os = "windows")]
impl SafeHInternet {
    fn new(handle: HINTERNET) -> Option<Self> {
        if handle.is_null() {
            None
        } else {
            Some(Self(handle))
        }
    }

    fn as_raw(&self) -> HINTERNET {
        self.0
    }
}

#[cfg(target_os = "windows")]
impl Drop for SafeHInternet {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                WinHttpCloseHandle(self.0);
            }
            self.0 = std::ptr::null_mut();
        }
    }
}

#[cfg(target_os = "windows")]
struct IeProxyConfigGuard(WINHTTP_CURRENT_USER_IE_PROXY_CONFIG);

#[cfg(target_os = "windows")]
impl Drop for IeProxyConfigGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.0.lpszAutoConfigUrl.is_null() {
                GlobalFree(self.0.lpszAutoConfigUrl as HGLOBAL);
                self.0.lpszAutoConfigUrl = std::ptr::null_mut();
            }
            if !self.0.lpszProxy.is_null() {
                GlobalFree(self.0.lpszProxy as HGLOBAL);
                self.0.lpszProxy = std::ptr::null_mut();
            }
            if !self.0.lpszProxyBypass.is_null() {
                GlobalFree(self.0.lpszProxyBypass as HGLOBAL);
                self.0.lpszProxyBypass = std::ptr::null_mut();
            }
        }
    }
}

#[cfg(target_os = "windows")]
unsafe fn wide_ptr_to_string(ptr: *const u16) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let mut len = 0;
    while *ptr.add(len) != 0 {
        len += 1;
    }
    let slice = std::slice::from_raw_parts(ptr, len);
    String::from_utf16(slice).ok()
}

#[cfg(target_os = "windows")]
enum ResolutionOutcome {
    Pending,
    Complete(Vec<PacResultEntry>),
    Error(u32),
}

#[cfg(target_os = "windows")]
struct ResolverState {
    mutex: std::sync::Mutex<ResolutionInner>,
    condvar: std::sync::Condvar,
    closed_condvar: std::sync::Condvar,
}

#[cfg(target_os = "windows")]
impl ResolverState {
    fn lock(&self) -> std::sync::MutexGuard<'_, ResolutionInner> {
        match self.mutex.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    fn wait_outcome<'a>(
        &self,
        guard: std::sync::MutexGuard<'a, ResolutionInner>,
        duration: std::time::Duration,
    ) -> std::sync::MutexGuard<'a, ResolutionInner> {
        match self.condvar.wait_timeout(guard, duration) {
            Ok((g, _)) => g,
            Err(poisoned) => poisoned.into_inner().0,
        }
    }

    fn wait_closed<'a>(
        &self,
        guard: std::sync::MutexGuard<'a, ResolutionInner>,
        duration: std::time::Duration,
    ) -> std::sync::MutexGuard<'a, ResolutionInner> {
        match self.closed_condvar.wait_timeout(guard, duration) {
            Ok((g, _)) => g,
            Err(poisoned) => poisoned.into_inner().0,
        }
    }
}

#[cfg(target_os = "windows")]
struct ResolutionInner {
    outcome: ResolutionOutcome,
    handle_closed: bool,
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn winhttp_status_callback(
    h_internet: HINTERNET,
    dw_context: DWORD_PTR,
    dw_internet_status: DWORD,
    lpv_status_information: LPVOID,
    _dw_status_information_length: DWORD,
) {
    if dw_context == 0 {
        return;
    }

    let state_ref = &*(dw_context as *const ResolverState);

    match dw_internet_status {
        WINHTTP_CALLBACK_STATUS_GETPROXYFORURL_COMPLETE => {
            let mut result = WINHTTP_PROXY_RESULT {
                cEntries: 0,
                pEntries: std::ptr::null_mut(),
            };
            let ret = WinHttpGetProxyResult(h_internet, &mut result);
            let outcome = if ret == ERROR_SUCCESS {
                let entries = parse_winhttp_proxy_result(&result);
                WinHttpFreeProxyResult(&mut result);
                ResolutionOutcome::Complete(entries)
            } else {
                ResolutionOutcome::Error(ret)
            };

            let mut guard = state_ref.lock();
            if let ResolutionOutcome::Pending = guard.outcome {
                guard.outcome = outcome;
                state_ref.condvar.notify_all();
            }
        }
        WINHTTP_CALLBACK_STATUS_REQUEST_ERROR => {
            let error_code = if !lpv_status_information.is_null() {
                let async_res = &*(lpv_status_information as *const WINHTTP_ASYNC_RESULT);
                async_res.dwError
            } else {
                1
            };

            let mut guard = state_ref.lock();
            if let ResolutionOutcome::Pending = guard.outcome {
                guard.outcome = ResolutionOutcome::Error(error_code);
                state_ref.condvar.notify_all();
            }
        }
        WINHTTP_CALLBACK_STATUS_HANDLE_CLOSING => {
            let mut guard = state_ref.lock();
            guard.handle_closed = true;
            state_ref.closed_condvar.notify_all();
            drop(guard);

            // HANDLE_CLOSING is guaranteed to be the final callback for this handle.
            // Release the Arc reference passed as dw_context.
            let _ = std::sync::Arc::from_raw(dw_context as *const ResolverState);
        }
        _ => {}
    }
}

#[cfg(target_os = "windows")]
unsafe fn parse_winhttp_proxy_result(result: &WINHTTP_PROXY_RESULT) -> Vec<PacResultEntry> {
    let mut out = Vec::new();
    if result.pEntries.is_null() || result.cEntries == 0 {
        return out;
    }

    let slice = std::slice::from_raw_parts(result.pEntries, result.cEntries as usize);
    for entry in slice {
        if entry.fProxy == 0 || entry.fBypass != 0 {
            out.push(PacResultEntry::Direct);
            continue;
        }

        let scheme = match entry.ProxyScheme {
            INTERNET_SCHEME_HTTP => "http",
            INTERNET_SCHEME_HTTPS => "https",
            _ => {
                out.push(PacResultEntry::UnsupportedScheme);
                continue;
            }
        };

        if let Some(host_str) = wide_ptr_to_string(entry.pwszProxy) {
            let mut host = host_str.trim().to_string();
            if let Some((_, rest)) = host.rsplit_once('@') {
                host = rest.to_string();
            }
            let host = host.trim_end_matches('/').to_string();
            if host.is_empty() {
                out.push(PacResultEntry::UnsupportedScheme);
                continue;
            }

            let port = if entry.ProxyPort != 0 {
                entry.ProxyPort
            } else if scheme == "https" {
                443
            } else {
                80
            };

            out.push(PacResultEntry::Proxy { scheme, host, port });
        } else {
            out.push(PacResultEntry::UnsupportedScheme);
        }
    }

    out
}

#[cfg(target_os = "windows")]
fn winhttp_resolve(
    target: &ParsedTargetUrl,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> crate::ProviderResult<Route> {
    let mut ie_config = WINHTTP_CURRENT_USER_IE_PROXY_CONFIG {
        fAutoDetect: 0,
        lpszAutoConfigUrl: std::ptr::null_mut(),
        lpszProxy: std::ptr::null_mut(),
        lpszProxyBypass: std::ptr::null_mut(),
    };

    let got_config = unsafe { WinHttpGetIEProxyConfigForCurrentUser(&mut ie_config) };
    let guard = IeProxyConfigGuard(ie_config);

    if got_config == 0 {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "Unable to read Windows proxy configuration.",
        ));
    }

    let has_wpad = guard.0.fAutoDetect != 0;
    let pac_url = unsafe { wide_ptr_to_string(guard.0.lpszAutoConfigUrl) };
    let has_pac_url = pac_url.as_deref().map(|s| !s.trim().is_empty()).unwrap_or(false);

    if has_wpad || has_pac_url {
        // Dynamic PAC/WPAD resolution
        return winhttp_resolve_pac(target, &guard, has_wpad, has_pac_url, cancelled, deadline);
    }

    // Static proxy evaluation
    let proxy_str = unsafe { wide_ptr_to_string(guard.0.lpszProxy) };
    let bypass_str = unsafe { wide_ptr_to_string(guard.0.lpszProxyBypass) };

    let proxy_str = match proxy_str {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            return Ok(Route {
                proxy: None,
                source: "windows-direct",
            });
        }
    };

    // Check static bypass list
    if let Some(bypass) = bypass_str {
        if !bypass.trim().is_empty() && matches_bypass_list(&bypass, &target.host, target.port) {
            return Ok(Route {
                proxy: None,
                source: "windows-direct",
            });
        }
    }

    // Select proxy based on target scheme (HTTP vs HTTPS)
    match select_static_proxy(&proxy_str, &target.scheme) {
        StaticProxyMatch::Proxy(proxy_url) => Ok(Route {
            proxy: Some(proxy_url),
            source: "windows-system",
        }),
        StaticProxyMatch::Direct => Ok(Route {
            proxy: None,
            source: "windows-direct",
        }),
        StaticProxyMatch::UnsupportedScheme => Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "unsupported proxy scheme",
        )),
    }
}

#[cfg(target_os = "windows")]
fn winhttp_resolve_pac(
    target: &ParsedTargetUrl,
    config: &IeProxyConfigGuard,
    has_wpad: bool,
    has_pac_url: bool,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> crate::ProviderResult<Route> {
    if cancelled.load(Ordering::Relaxed) {
        return Err(provider_error(
            ProviderErrorKind::Cancelled,
            "proxy resolution cancelled",
        ));
    }
    if Instant::now() >= deadline {
        return Err(timeout("proxy resolution timed out"));
    }

    let session = unsafe {
        WinHttpOpen(
            std::ptr::null(),
            WINHTTP_ACCESS_TYPE_NO_PROXY,
            std::ptr::null(),
            std::ptr::null(),
            WINHTTP_FLAG_ASYNC,
        )
    };

    let session = SafeHInternet::new(session)
        .ok_or_else(|| runtime_unavailable("failed to initialize WinHTTP session"))?;

    let previous_callback = unsafe {
        WinHttpSetStatusCallback(
            session.as_raw(),
            Some(winhttp_status_callback),
            WINHTTP_CALLBACK_FLAG_ALL_NOTIFICATIONS,
            0,
        )
    };
    if previous_callback as usize == usize::MAX {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "Unable to register Windows proxy callback.",
        ));
    }

    let mut h_resolver: HINTERNET = std::ptr::null_mut();
    let create_res = unsafe { WinHttpCreateProxyResolver(session.as_raw(), &mut h_resolver) };
    if create_res != ERROR_SUCCESS || h_resolver.is_null() {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "failed to create WinHTTP proxy resolver",
        ));
    }

    let mut options = WINHTTP_AUTOPROXY_OPTIONS {
        dwFlags: 0,
        dwAutoDetectFlags: 0,
        lpszAutoConfigUrl: std::ptr::null(),
        lpvReserved: std::ptr::null_mut(),
        dwReserved: 0,
        fAutoLogonIfChallenged: TRUE,
    };

    if has_wpad {
        options.dwFlags |= WINHTTP_AUTOPROXY_AUTO_DETECT;
        options.dwAutoDetectFlags = WINHTTP_AUTO_DETECT_TYPE_DHCP | WINHTTP_AUTO_DETECT_TYPE_DNS_A;
    }
    if has_pac_url {
        options.dwFlags |= WINHTTP_AUTOPROXY_CONFIG_URL;
        options.lpszAutoConfigUrl = config.0.lpszAutoConfigUrl;
    }

    let url_wide: Vec<u16> = target
        .sanitized_url
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    let state = std::sync::Arc::new(ResolverState {
        mutex: std::sync::Mutex::new(ResolutionInner {
            outcome: ResolutionOutcome::Pending,
            handle_closed: false,
        }),
        condvar: std::sync::Condvar::new(),
        closed_condvar: std::sync::Condvar::new(),
    });

    let raw_context = std::sync::Arc::into_raw(state.clone());
    // The handle context also accompanies HANDLE_CLOSING, including immediate start failure.
    let mut context = raw_context as DWORD_PTR;
    if unsafe {
        WinHttpSetOption(
            h_resolver,
            45,
            (&mut context as *mut DWORD_PTR).cast(),
            std::mem::size_of::<DWORD_PTR>() as DWORD,
        )
    } == 0
    {
        unsafe {
            drop(std::sync::Arc::from_raw(raw_context));
            WinHttpCloseHandle(h_resolver);
        }
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "Unable to set Windows proxy context.",
        ));
    }

    let dw_ret =
        unsafe { WinHttpGetProxyForUrlEx(h_resolver, url_wide.as_ptr(), &options, raw_context as DWORD_PTR) };

    if dw_ret != ERROR_IO_PENDING && dw_ret != ERROR_SUCCESS {
        // HANDLE_CLOSING owns the raw Arc once the handle context is installed.
        unsafe {
            WinHttpCloseHandle(h_resolver);
        }
        return map_winhttp_error(dw_ret);
    }

    // Wait for async completion or cancellation / deadline.
    let step = std::time::Duration::from_millis(25);
    let mut guard = state.lock();

    loop {
        if cancelled.load(Ordering::Relaxed) {
            break;
        }
        let now = Instant::now();
        if now >= deadline {
            break;
        }
        if !matches!(guard.outcome, ResolutionOutcome::Pending) {
            break;
        }

        let remaining = deadline - now;
        let wait_dur = remaining.min(step);
        guard = state.wait_outcome(guard, wait_dur);
    }

    let was_cancelled = cancelled.load(Ordering::Relaxed);
    let was_timed_out = Instant::now() >= deadline;

    drop(guard);
    // Close the resolver handle to cancel any in-flight native PAC worker.
    unsafe {
        WinHttpCloseHandle(h_resolver);
    }
    let mut guard = state.lock();

    // Wait with bounded deadline for HANDLE_CLOSING to ensure native resources are released.
    let close_deadline = Instant::now() + std::time::Duration::from_millis(250);
    while !guard.handle_closed {
        let now = Instant::now();
        if now >= close_deadline {
            break;
        }
        guard = state.wait_closed(guard, close_deadline - now);
    }

    if was_cancelled {
        return Err(provider_error(
            ProviderErrorKind::Cancelled,
            "proxy resolution cancelled",
        ));
    }
    if was_timed_out {
        return Err(timeout("proxy resolution timed out"));
    }

    match &guard.outcome {
        ResolutionOutcome::Complete(entries) => select_route_from_pac_entries(entries),
        ResolutionOutcome::Error(err_code) => map_winhttp_error(*err_code),
        ResolutionOutcome::Pending => Err(timeout("proxy resolution timed out")),
    }
}

#[cfg(target_os = "windows")]
fn map_winhttp_error<T>(code: u32) -> crate::ProviderResult<T> {
    match code {
        ERROR_OPERATION_CANCELLED | ERROR_OPERATION_ABORTED => Err(provider_error(
            ProviderErrorKind::Cancelled,
            "proxy resolution cancelled",
        )),
        ERROR_WINHTTP_LOGIN_FAILURE => Err(provider_error(
            ProviderErrorKind::AuthenticationFailed,
            "PAC authentication failed",
        )),
        ERROR_WINHTTP_TIMEOUT => Err(timeout("proxy resolution timed out")),
        _ => Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "PAC resolution failed",
        )),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_target_url_basic() {
        let parsed = parse_target_url("http://example.com/foo?bar=1").unwrap();
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 80);
        assert_eq!(parsed.sanitized_url, "http://example.com/foo?bar=1");
        assert_eq!(
            parse_target_url("https://example.com/redirected?record=1")
                .unwrap()
                .sanitized_url,
            "https://example.com/redirected?record=1"
        );

        let parsed_https = parse_target_url("https://secure.example.com:8443").unwrap();
        assert_eq!(parsed_https.scheme, "https");
        assert_eq!(parsed_https.host, "secure.example.com");
        assert_eq!(parsed_https.port, 8443);
        assert_eq!(parsed_https.sanitized_url, "https://secure.example.com:8443/");
    }

    #[test]
    fn test_parse_target_url_strips_credentials() {
        let parsed = parse_target_url("http://user:secret@example.com/api").unwrap();
        assert_eq!(parsed.host, "example.com");
        assert!(!parsed.sanitized_url.contains("secret"));
        assert!(!parsed.sanitized_url.contains("user"));
        assert_eq!(parsed.sanitized_url, "http://example.com/api");
    }

    #[test]
    fn test_parse_target_url_ipv6() {
        let parsed = parse_target_url("https://[::1]:9000/path").unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "[::1]");
        assert_eq!(parsed.port, 9000);
        assert_eq!(parsed.sanitized_url, "https://[::1]:9000/path");
    }

    #[test]
    fn test_parse_target_url_invalid() {
        assert!(parse_target_url("not_a_url").is_err());
        assert!(parse_target_url("ftp://example.com").is_err());
        assert!(parse_target_url("http://").is_err());
    }

    #[test]
    fn test_select_static_proxy_http_vs_https() {
        let config = "http=httpproxy.example:8080;https=httpsproxy.example:8443";
        assert_eq!(
            select_static_proxy(config, "http"),
            StaticProxyMatch::Proxy("http://httpproxy.example:8080".into())
        );
        assert_eq!(
            select_static_proxy(config, "https"),
            StaticProxyMatch::Proxy("http://httpsproxy.example:8443".into())
        );
    }

    #[test]
    fn test_select_static_proxy_unlisted_scheme_is_direct() {
        let config = "http=httpproxy.example:8080";
        assert_eq!(select_static_proxy(config, "https"), StaticProxyMatch::Direct);
    }

    #[test]
    fn test_select_static_proxy_single_generic() {
        let config = "proxy.example:8080";
        assert_eq!(
            select_static_proxy(config, "http"),
            StaticProxyMatch::Proxy("http://proxy.example:8080".into())
        );
        assert_eq!(
            select_static_proxy(config, "https"),
            StaticProxyMatch::Proxy("http://proxy.example:8080".into())
        );
    }

    #[test]
    fn test_select_static_proxy_explicit_https_and_unsupported_socks() {
        let config = "https=https://secureproxy.example:8443";
        assert_eq!(
            select_static_proxy(config, "https"),
            StaticProxyMatch::Proxy("https://secureproxy.example:8443".into())
        );

        let socks_config = "http=socks://127.0.0.1:1080";
        assert_eq!(
            select_static_proxy(socks_config, "http"),
            StaticProxyMatch::UnsupportedScheme
        );
    }

    #[test]
    fn test_matches_bypass_local() {
        assert!(matches_bypass_list("<local>", "intranet", 80));
        assert!(matches_bypass_list("<local>", "127.0.0.1", 80));
        assert!(matches_bypass_list("<local>", "localhost", 80));
        assert!(!matches_bypass_list("<local>", "example.com", 80));
    }

    #[test]
    fn test_matches_bypass_no_loopback() {
        assert!(!matches_bypass_list("<local>;<-loopback>", "127.0.0.1", 80));
        assert!(matches_bypass_list("<local>;<-loopback>", "intranet", 80));
    }

    #[test]
    fn test_matches_bypass_wildcards_and_ports() {
        assert!(matches_bypass_list("*.example.com", "api.example.com", 80));
        assert!(matches_bypass_list("*.example.com:8443", "api.example.com", 8443));
        assert!(!matches_bypass_list("*.example.com:8443", "api.example.com", 80));
        assert!(matches_bypass_list("192.168.*", "192.168.1.100", 80));
        assert!(!matches_bypass_list("192.168.*", "10.0.0.1", 80));
    }

    #[test]
    fn test_select_route_from_pac_entries_preserves_order_and_fails_closed() {
        // First entry is supported proxy
        let entries = vec![
            PacResultEntry::Proxy {
                scheme: "http",
                host: "proxy1.example".into(),
                port: 8080,
            },
            PacResultEntry::Direct,
        ];
        let route = select_route_from_pac_entries(&entries).unwrap();
        assert_eq!(route.proxy.as_deref(), Some("http://proxy1.example:8080"));
        assert_eq!(route.source, "windows-pac");

        // First entry is Direct
        let entries_direct = vec![
            PacResultEntry::Direct,
            PacResultEntry::Proxy {
                scheme: "http",
                host: "proxy1.example".into(),
                port: 8080,
            },
        ];
        let route_direct = select_route_from_pac_entries(&entries_direct).unwrap();
        assert_eq!(route_direct.proxy, None);
        assert_eq!(route_direct.source, "windows-pac");

        // Unsupported scheme followed by Direct must NOT failover to Direct!
        let entries_unsupported = vec![PacResultEntry::UnsupportedScheme, PacResultEntry::Direct];
        let res = select_route_from_pac_entries(&entries_unsupported);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind, ProviderErrorKind::ProxyFailed);

        // Empty entries must fail
        assert!(select_route_from_pac_entries(&[]).is_err());
    }
}
