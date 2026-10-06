//! macOS native per-URL proxy resolution using CoreFoundation and CFNetwork.
//!
//! Evaluates macOS system proxy settings and executes PAC (Proxy Auto-Config) / WPAD
//! scripts via CFNetwork run loop sources with deadline and cancellation bounds.

use std::ffi::{c_char, c_uchar, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use super::Route;
use crate::{provider_error, runtime_unavailable, timeout};
use app_model::ProviderErrorKind;

/// Exported platform entry point resolving the proxy route for a target URL.
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

    #[cfg(target_os = "macos")]
    {
        macos_resolve(&parsed_url, cancelled, deadline)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = parsed_url;
        Err(runtime_unavailable(
            "macOS proxy resolver is only supported on macOS",
        ))
    }
}

// ---------------------------------------------------------------------------
// Pure Rust URL parsing and endpoint formatting (portable and testable)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedTargetUrl {
    pub(crate) scheme: String,
    pub(crate) host: String,
    pub(crate) port: u16,
    pub(crate) sanitized_url: String,
}

pub(crate) fn parse_target_url(raw: &str) -> crate::ProviderResult<ParsedTargetUrl> {
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

pub(crate) fn format_proxy_endpoint(scheme: &str, host: &str, port: u16) -> String {
    let host = host.trim();
    let formatted_host = if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    format!("{scheme}://{formatted_host}:{port}")
}

// ---------------------------------------------------------------------------
// CoreFoundation and CFNetwork FFI Definitions
// ---------------------------------------------------------------------------

pub type CFTypeRef = *const c_void;
pub type CFStringRef = *const c_void;
pub type CFDictionaryRef = *const c_void;
pub type CFArrayRef = *const c_void;
pub type CFURLRef = *const c_void;
pub type CFNumberRef = *const c_void;
pub type CFErrorRef = *const c_void;
pub type CFRunLoopRef = *const c_void;
pub type CFRunLoopSourceRef = *const c_void;
pub type CFIndex = isize;
pub type CFStringEncoding = u32;
pub type Boolean = c_uchar;
pub type CFTimeInterval = f64;
pub type CFTypeID = usize;
pub type CFNumberType = CFIndex;

pub const K_CFSTRING_ENCODING_UTF8: CFStringEncoding = 0x08000100;
pub const K_CFNUMBER_SINT32_TYPE: CFNumberType = 3;

pub const K_CFRUNLOOP_RUN_FINISHED: i32 = 1;
pub const K_CFRUNLOOP_RUN_STOPPED: i32 = 2;
pub const K_CFRUNLOOP_RUN_TIMED_OUT: i32 = 3;
pub const K_CFRUNLOOP_RUN_HANDLED_SOURCE: i32 = 4;

#[repr(C)]
pub struct CFStreamClientContext {
    pub version: CFIndex,
    pub info: *mut c_void,
    pub retain: Option<unsafe extern "C" fn(info: *mut c_void) -> *mut c_void>,
    pub release: Option<unsafe extern "C" fn(info: *mut c_void)>,
    pub copy_description: Option<unsafe extern "C" fn(info: *mut c_void) -> CFStringRef>,
}

pub type CFProxyAutoConfigurationResultCallback =
    unsafe extern "C" fn(client: *mut c_void, proxy_list: CFArrayRef, error: CFErrorRef);

#[cfg_attr(target_os = "macos", link(name = "CoreFoundation", kind = "framework"))]
#[cfg_attr(target_os = "macos", link(name = "CFNetwork", kind = "framework"))]
#[cfg_attr(target_os = "macos", link(name = "SystemConfiguration", kind = "framework"))]
extern "C" {
    pub static kCFRunLoopDefaultMode: CFStringRef;

    pub static kCFProxyTypeKey: CFStringRef;
    pub static kCFProxyTypeNone: CFStringRef;
    pub static kCFProxyTypeHTTP: CFStringRef;
    pub static kCFProxyTypeHTTPS: CFStringRef;
    pub static kCFProxyTypeSOCKS: CFStringRef;
    pub static kCFProxyTypeFTP: CFStringRef;
    pub static kCFProxyTypeAutoConfigurationURL: CFStringRef;
    pub static kCFProxyTypeAutoConfigurationJavaScript: CFStringRef;
    pub static kCFProxyHostNameKey: CFStringRef;
    pub static kCFProxyPortNumberKey: CFStringRef;
    pub static kCFProxyAutoConfigurationURLKey: CFStringRef;
    pub static kCFProxyAutoConfigurationJavaScriptKey: CFStringRef;
    pub static kSCPropNetProxiesProxyAutoDiscoveryEnable: CFStringRef;
    pub static kSCPropNetProxiesProxyAutoConfigEnable: CFStringRef;

    pub fn CFRelease(cf: CFTypeRef);
    pub fn CFRetain(cf: CFTypeRef) -> CFTypeRef;
    pub fn CFEqual(cf1: CFTypeRef, cf2: CFTypeRef) -> Boolean;
    pub fn CFGetTypeID(cf: CFTypeRef) -> CFTypeID;
    pub fn CFStringGetTypeID() -> CFTypeID;
    pub fn CFDictionaryGetTypeID() -> CFTypeID;
    pub fn CFArrayGetTypeID() -> CFTypeID;
    pub fn CFNumberGetTypeID() -> CFTypeID;
    pub fn CFURLGetTypeID() -> CFTypeID;

    pub fn CFStringGetLength(the_string: CFStringRef) -> CFIndex;
    pub fn CFStringGetMaximumSizeForEncoding(length: CFIndex, encoding: CFStringEncoding) -> CFIndex;
    pub fn CFStringGetCString(
        the_string: CFStringRef,
        buffer: *mut c_char,
        buffer_size: CFIndex,
        encoding: CFStringEncoding,
    ) -> Boolean;

    pub fn CFStringCreateWithBytes(
        allocator: *const c_void,
        bytes: *const u8,
        length: CFIndex,
        encoding: CFStringEncoding,
        external_representation: Boolean,
    ) -> CFStringRef;

    pub fn CFURLCreateWithBytes(
        allocator: *const c_void,
        url_bytes: *const u8,
        length: CFIndex,
        encoding: CFStringEncoding,
        base_url: CFURLRef,
    ) -> CFURLRef;
    pub fn CFURLCreateWithString(
        allocator: *const c_void,
        url_string: CFStringRef,
        base_url: CFURLRef,
    ) -> CFURLRef;

    pub fn CFNumberGetValue(number: CFNumberRef, the_type: CFNumberType, value_ptr: *mut c_void) -> Boolean;

    pub fn CFDictionaryGetValue(the_dict: CFDictionaryRef, key: *const c_void) -> *const c_void;

    pub fn CFArrayGetCount(the_array: CFArrayRef) -> CFIndex;
    pub fn CFArrayGetValueAtIndex(the_array: CFArrayRef, idx: CFIndex) -> *const c_void;

    pub fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    pub fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    pub fn CFRunLoopRemoveSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    pub fn CFRunLoopRunInMode(
        mode: CFStringRef,
        seconds: CFTimeInterval,
        return_after_source_handled: Boolean,
    ) -> i32;
    pub fn CFRunLoopStop(rl: CFRunLoopRef);
    pub fn CFRunLoopSourceInvalidate(source: CFRunLoopSourceRef);

    pub fn CFNetworkCopySystemProxySettings() -> CFDictionaryRef;
    pub fn CFNetworkCopyProxiesForURL(url: CFURLRef, proxy_settings: CFDictionaryRef) -> CFArrayRef;
    pub fn CFNetworkExecuteProxyAutoConfigurationURL(
        proxy_auto_config_url: CFURLRef,
        target_url: CFURLRef,
        cb: CFProxyAutoConfigurationResultCallback,
        client_context: *mut CFStreamClientContext,
    ) -> CFRunLoopSourceRef;
    pub fn CFNetworkExecuteProxyAutoConfigurationScript(
        proxy_auto_configuration_script: CFStringRef,
        target_url: CFURLRef,
        cb: CFProxyAutoConfigurationResultCallback,
        client_context: *mut CFStreamClientContext,
    ) -> CFRunLoopSourceRef;
}

// ---------------------------------------------------------------------------
// RAII Handles for CoreFoundation Objects and RunLoop Sources
// ---------------------------------------------------------------------------

struct CfRef<T>(*const T);

impl<T> CfRef<T> {
    fn new(ptr: *const T) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self(ptr))
        }
    }

    unsafe fn from_raw(ptr: *const T) -> Self {
        Self(ptr)
    }

    fn as_ptr(&self) -> *const T {
        self.0
    }
}

impl<T> Drop for CfRef<T> {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CFRelease(self.0 as CFTypeRef) };
        }
    }
}

struct RunLoopSourceGuard {
    run_loop: CFRunLoopRef,
    source: CFRunLoopSourceRef,
    mode: CFStringRef,
}

impl Drop for RunLoopSourceGuard {
    fn drop(&mut self) {
        unsafe {
            CFRunLoopRemoveSource(self.run_loop, self.source, self.mode);
            CFRunLoopSourceInvalidate(self.source);
            CFRelease(self.source as CFTypeRef);
        }
    }
}

// CFNetwork internally has concurrency hazards when multiple threads execute PAC scripts
// concurrently. Serializing PAC run loop execution guarantees stability.
static PAC_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Native macOS Resolution Implementation
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
fn macos_resolve(
    target: &ParsedTargetUrl,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> crate::ProviderResult<Route> {
    let url_bytes = target.sanitized_url.as_bytes();
    let target_cfurl = unsafe {
        CFURLCreateWithBytes(
            std::ptr::null(),
            url_bytes.as_ptr(),
            url_bytes.len() as CFIndex,
            K_CFSTRING_ENCODING_UTF8,
            std::ptr::null(),
        )
    };
    let target_cfurl = CfRef::new(target_cfurl).ok_or_else(|| crate::invalid_url("invalid request url"))?;

    let system_settings = unsafe { CFNetworkCopySystemProxySettings() };
    let system_settings = CfRef::new(system_settings)
        .ok_or_else(|| runtime_unavailable("unable to read macOS system proxy settings"))?;

    let settings_type_id = unsafe { CFGetTypeID(system_settings.as_ptr()) };
    if settings_type_id != unsafe { CFDictionaryGetTypeID() } {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "invalid macOS system proxy settings",
        ));
    }

    let proxies = unsafe { CFNetworkCopyProxiesForURL(target_cfurl.as_ptr(), system_settings.as_ptr()) };
    let proxies = CfRef::new(proxies).ok_or_else(|| {
        provider_error(
            ProviderErrorKind::ProxyFailed,
            "failed to resolve proxies for url",
        )
    })?;

    let array_type_id = unsafe { CFGetTypeID(proxies.as_ptr()) };
    if array_type_id != unsafe { CFArrayGetTypeID() } {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "invalid proxy list from system",
        ));
    }

    // An empty PAC URL asks CFNetwork to perform WPAD. CopyProxiesForURL above
    // also initializes the native PAC machinery; it does not execute discovery.
    if unsafe {
        setting_enabled(
            system_settings.as_ptr(),
            kSCPropNetProxiesProxyAutoDiscoveryEnable,
        )
    } && !unsafe { setting_enabled(system_settings.as_ptr(), kSCPropNetProxiesProxyAutoConfigEnable) }
    {
        let empty = CfRef::new(unsafe {
            CFStringCreateWithBytes(std::ptr::null(), b"".as_ptr(), 0, K_CFSTRING_ENCODING_UTF8, 0)
        })
        .ok_or_else(|| runtime_unavailable("unable to initialize WPAD"))?;
        let discovery_url =
            CfRef::new(unsafe { CFURLCreateWithString(std::ptr::null(), empty.as_ptr(), std::ptr::null()) })
                .ok_or_else(|| runtime_unavailable("unable to initialize WPAD URL"))?;
        return execute_pac_url(discovery_url.as_ptr(), target_cfurl.as_ptr(), cancelled, deadline);
    }

    evaluate_proxy_array(
        proxies.as_ptr(),
        target_cfurl.as_ptr(),
        "macos-system",
        cancelled,
        deadline,
    )
}

#[cfg(target_os = "macos")]
fn evaluate_proxy_array(
    the_array: CFArrayRef,
    target_url: CFURLRef,
    default_source: &'static str,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> crate::ProviderResult<Route> {
    let count = unsafe { CFArrayGetCount(the_array) };
    if count <= 0 {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "no proxy routes returned",
        ));
    }

    let mut seen_unsupported_proxy = false;

    for idx in 0..count {
        if cancelled.load(Ordering::Relaxed) {
            return Err(provider_error(
                ProviderErrorKind::Cancelled,
                "proxy resolution cancelled",
            ));
        }
        if Instant::now() >= deadline {
            return Err(timeout("proxy resolution timed out"));
        }

        let item = unsafe { CFArrayGetValueAtIndex(the_array, idx) };
        if item.is_null() || unsafe { CFGetTypeID(item) != CFDictionaryGetTypeID() } {
            return Err(provider_error(
                ProviderErrorKind::ProxyFailed,
                "invalid proxy dictionary",
            ));
        }

        let proxy_dict = item as CFDictionaryRef;
        let type_val = unsafe { CFDictionaryGetValue(proxy_dict, kCFProxyTypeKey as *const c_void) };
        if type_val.is_null() || unsafe { CFGetTypeID(type_val) != CFStringGetTypeID() } {
            return Err(provider_error(
                ProviderErrorKind::ProxyFailed,
                "invalid proxy type key",
            ));
        }

        let type_str = type_val as CFStringRef;

        // DIRECT route (including OS static exception / bypass matches)
        if unsafe { cfstring_equals(type_str, kCFProxyTypeNone) } {
            if seen_unsupported_proxy {
                return Err(provider_error(
                    ProviderErrorKind::ProxyFailed,
                    "unsupported proxy scheme",
                ));
            }
            return Ok(Route {
                proxy: None,
                source: if default_source == "macos-pac" {
                    "macos-pac"
                } else {
                    "macos-direct"
                },
            });
        }

        // CFNetwork HTTPS means an HTTP proxy for HTTPS targets, not TLS to the proxy.
        if unsafe {
            cfstring_equals(type_str, kCFProxyTypeHTTP) || cfstring_equals(type_str, kCFProxyTypeHTTPS)
        } {
            let host = unsafe { extract_host(proxy_dict) }
                .ok_or_else(|| provider_error(ProviderErrorKind::ProxyFailed, "missing proxy host"))?;
            let port = unsafe { extract_port_from_dict(proxy_dict) }.unwrap_or(80);
            let endpoint = format_proxy_endpoint("http", &host, port);
            return Ok(Route {
                proxy: Some(endpoint),
                source: default_source,
            });
        }

        // PAC Auto-Configuration URL (including WPAD auto-discovery results)
        if unsafe { cfstring_equals(type_str, kCFProxyTypeAutoConfigurationURL) } {
            if default_source == "macos-pac" {
                return Err(provider_error(
                    ProviderErrorKind::ProxyFailed,
                    "nested PAC configuration not supported",
                ));
            }

            let pac_url_val =
                unsafe { CFDictionaryGetValue(proxy_dict, kCFProxyAutoConfigurationURLKey as *const c_void) };
            if pac_url_val.is_null() {
                return Err(provider_error(
                    ProviderErrorKind::ProxyFailed,
                    "missing PAC URL in proxy configuration",
                ));
            }

            let pac_url_ref = if unsafe { CFGetTypeID(pac_url_val) == CFURLGetTypeID() } {
                unsafe {
                    CFRetain(pac_url_val);
                    CfRef::from_raw(pac_url_val as CFURLRef)
                }
            } else if unsafe { CFGetTypeID(pac_url_val) == CFStringGetTypeID() } {
                let created = unsafe {
                    CFURLCreateWithString(std::ptr::null(), pac_url_val as CFStringRef, std::ptr::null())
                };
                CfRef::new(created)
                    .ok_or_else(|| provider_error(ProviderErrorKind::ProxyFailed, "invalid PAC URL string"))?
            } else {
                return Err(provider_error(
                    ProviderErrorKind::ProxyFailed,
                    "invalid PAC URL type",
                ));
            };

            return execute_pac_url(pac_url_ref.as_ptr(), target_url, cancelled, deadline);
        }

        // PAC Inlined JavaScript
        if unsafe { cfstring_equals(type_str, kCFProxyTypeAutoConfigurationJavaScript) } {
            if default_source == "macos-pac" {
                return Err(provider_error(
                    ProviderErrorKind::ProxyFailed,
                    "nested PAC configuration not supported",
                ));
            }

            let script_val = unsafe {
                CFDictionaryGetValue(
                    proxy_dict,
                    kCFProxyAutoConfigurationJavaScriptKey as *const c_void,
                )
            };
            if script_val.is_null() || unsafe { CFGetTypeID(script_val) != CFStringGetTypeID() } {
                return Err(provider_error(
                    ProviderErrorKind::ProxyFailed,
                    "missing PAC script in proxy configuration",
                ));
            }

            return execute_pac_script(script_val as CFStringRef, target_url, cancelled, deadline);
        }

        // Unsupported scheme (SOCKS, FTP, etc.)
        seen_unsupported_proxy = true;
    }

    if seen_unsupported_proxy {
        Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "unsupported proxy scheme",
        ))
    } else {
        Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "no supported proxy route found",
        ))
    }
}

// ---------------------------------------------------------------------------
// Native PAC Execution and RunLoop Pump
// ---------------------------------------------------------------------------

struct PacRunLoopState {
    result: Option<Result<CfRef<c_void>, &'static str>>,
    run_loop: CFRunLoopRef,
}

unsafe extern "C" fn pac_result_callback(client: *mut c_void, proxy_list: CFArrayRef, error: CFErrorRef) {
    if client.is_null() {
        return;
    }
    let state = &mut *(client as *mut PacRunLoopState);
    if !error.is_null() {
        state.result = Some(Err("PAC execution failed"));
    } else if !proxy_list.is_null() {
        CFRetain(proxy_list as CFTypeRef);
        state.result = Some(Ok(CfRef::from_raw(proxy_list as *const c_void)));
    } else {
        state.result = Some(Err("PAC execution returned null proxy list"));
    }
    CFRunLoopStop(state.run_loop);
}

#[cfg(target_os = "macos")]
fn execute_pac_url(
    pac_url: CFURLRef,
    target_url: CFURLRef,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> crate::ProviderResult<Route> {
    execute_pac_runner(
        |cb, ctx| unsafe { CFNetworkExecuteProxyAutoConfigurationURL(pac_url, target_url, cb, ctx) },
        target_url,
        cancelled,
        deadline,
    )
}

#[cfg(target_os = "macos")]
fn execute_pac_script(
    script: CFStringRef,
    target_url: CFURLRef,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> crate::ProviderResult<Route> {
    execute_pac_runner(
        |cb, ctx| unsafe { CFNetworkExecuteProxyAutoConfigurationScript(script, target_url, cb, ctx) },
        target_url,
        cancelled,
        deadline,
    )
}

#[cfg(target_os = "macos")]
fn execute_pac_runner<F>(
    create_source: F,
    target_url: CFURLRef,
    cancelled: &AtomicBool,
    deadline: Instant,
) -> crate::ProviderResult<Route>
where
    F: FnOnce(CFProxyAutoConfigurationResultCallback, *mut CFStreamClientContext) -> CFRunLoopSourceRef,
{
    if cancelled.load(Ordering::Relaxed) {
        return Err(provider_error(
            ProviderErrorKind::Cancelled,
            "proxy resolution cancelled",
        ));
    }
    if Instant::now() >= deadline {
        return Err(timeout("PAC execution timed out"));
    }

    // Acquire lock to serialize PAC execution across threads
    let _lock = loop {
        if cancelled.load(Ordering::Relaxed) {
            return Err(provider_error(
                ProviderErrorKind::Cancelled,
                "proxy resolution cancelled",
            ));
        }
        if Instant::now() >= deadline {
            return Err(timeout("PAC execution timed out waiting for resolver lock"));
        }
        match PAC_LOCK.try_lock() {
            Ok(guard) => break guard,
            Err(_) => {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
        }
    };

    let run_loop = unsafe { CFRunLoopGetCurrent() };
    if run_loop.is_null() {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "unable to obtain current run loop",
        ));
    }

    let mut state = PacRunLoopState {
        result: None,
        run_loop,
    };

    let mut client_context = CFStreamClientContext {
        version: 0,
        info: &mut state as *mut PacRunLoopState as *mut c_void,
        retain: None,
        release: None,
        copy_description: None,
    };

    let source = create_source(pac_result_callback, &mut client_context);
    if source.is_null() {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "failed to create PAC run loop source",
        ));
    }

    let mode = unsafe { kCFRunLoopDefaultMode };
    unsafe {
        CFRunLoopAddSource(run_loop, source, mode);
    }
    // RAII guard guarantees source is removed, invalidated, and released upon scope exit.
    let _guard = RunLoopSourceGuard {
        run_loop,
        source,
        mode,
    };

    while state.result.is_none() {
        if cancelled.load(Ordering::Relaxed) {
            return Err(provider_error(
                ProviderErrorKind::Cancelled,
                "proxy resolution cancelled",
            ));
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(timeout("PAC execution timed out"));
        }

        let remaining = deadline.saturating_duration_since(now);
        // Bound each run loop iteration to at most 50ms for responsive cancellation.
        let slice = remaining.min(std::time::Duration::from_millis(50));
        let seconds = slice.as_secs_f64();

        let status = unsafe { CFRunLoopRunInMode(mode, seconds, 1) };
        if status == K_CFRUNLOOP_RUN_FINISHED {
            break;
        }
    }

    let pac_proxies = match state.result {
        Some(Ok(proxies_ref)) => proxies_ref,
        Some(Err(err_msg)) => {
            return Err(provider_error(ProviderErrorKind::ProxyFailed, err_msg));
        }
        None => {
            if cancelled.load(Ordering::Relaxed) {
                return Err(provider_error(
                    ProviderErrorKind::Cancelled,
                    "proxy resolution cancelled",
                ));
            }
            return Err(timeout("PAC execution timed out"));
        }
    };

    let pac_array = pac_proxies.as_ptr() as CFArrayRef;
    if unsafe { CFGetTypeID(pac_array as CFTypeRef) != CFArrayGetTypeID() } {
        return Err(provider_error(
            ProviderErrorKind::ProxyFailed,
            "invalid PAC proxy result list",
        ));
    }

    evaluate_proxy_array(pac_array, target_url, "macos-pac", cancelled, deadline)
}

// ---------------------------------------------------------------------------
// Native Helper Functions
// ---------------------------------------------------------------------------

#[cfg(target_os = "macos")]
unsafe fn setting_enabled(settings: CFDictionaryRef, key: CFStringRef) -> bool {
    let value = CFDictionaryGetValue(settings, key);
    if value.is_null() || CFGetTypeID(value) != CFNumberGetTypeID() {
        return false;
    }
    let mut enabled: i32 = 0;
    CFNumberGetValue(
        value,
        K_CFNUMBER_SINT32_TYPE,
        &mut enabled as *mut i32 as *mut c_void,
    ) != 0
        && enabled != 0
}

#[allow(dead_code)]
unsafe fn cfstring_equals(str1: CFStringRef, str2: CFStringRef) -> bool {
    if str1.is_null() || str2.is_null() {
        return false;
    }
    if str1 == str2 {
        return true;
    }
    CFEqual(str1 as CFTypeRef, str2 as CFTypeRef) != 0
}

#[allow(dead_code)]
unsafe fn cfstring_to_string(cf_str: CFStringRef) -> Option<String> {
    if cf_str.is_null() {
        return None;
    }
    let length = CFStringGetLength(cf_str);
    let max_size = CFStringGetMaximumSizeForEncoding(length, K_CFSTRING_ENCODING_UTF8);
    if max_size < 0 {
        return None;
    }
    let mut buf = vec![0u8; (max_size as usize) + 1];
    let ok = CFStringGetCString(
        cf_str,
        buf.as_mut_ptr() as *mut c_char,
        buf.len() as CFIndex,
        K_CFSTRING_ENCODING_UTF8,
    );
    if ok != 0 {
        let null_pos = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8(buf[..null_pos].to_vec()).ok()
    } else {
        None
    }
}

#[allow(dead_code)]
unsafe fn cfnumber_to_i32(cf_num: CFNumberRef) -> Option<i32> {
    if cf_num.is_null() {
        return None;
    }
    let mut value: i32 = 0;
    let ok = CFNumberGetValue(
        cf_num,
        K_CFNUMBER_SINT32_TYPE,
        &mut value as *mut i32 as *mut c_void,
    );
    if ok != 0 {
        Some(value)
    } else {
        None
    }
}

#[allow(dead_code)]
unsafe fn extract_port(val: CFTypeRef) -> Option<u16> {
    if val.is_null() {
        return None;
    }
    let type_id = CFGetTypeID(val);
    if type_id == CFNumberGetTypeID() {
        cfnumber_to_i32(val as CFNumberRef).and_then(|p| u16::try_from(p).ok())
    } else if type_id == CFStringGetTypeID() {
        cfstring_to_string(val as CFStringRef).and_then(|s| s.trim().parse::<u16>().ok())
    } else {
        None
    }
}

#[allow(dead_code)]
unsafe fn extract_host(dict: CFDictionaryRef) -> Option<String> {
    let host_val = CFDictionaryGetValue(dict, kCFProxyHostNameKey as *const c_void);
    if host_val.is_null() || CFGetTypeID(host_val) != CFStringGetTypeID() {
        return None;
    }
    cfstring_to_string(host_val as CFStringRef).map(|s| s.trim().to_ascii_lowercase())
}

#[allow(dead_code)]
unsafe fn extract_port_from_dict(dict: CFDictionaryRef) -> Option<u16> {
    let port_val = CFDictionaryGetValue(dict, kCFProxyPortNumberKey as *const c_void);
    if port_val.is_null() {
        return None;
    }
    extract_port(port_val)
}

// ---------------------------------------------------------------------------
// Unit Tests (Portable across platforms)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn parses_plain_http_url() {
        let parsed = parse_target_url("http://example.com/path?foo=bar").unwrap();
        assert_eq!(parsed.scheme, "http");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 80);
        assert_eq!(parsed.sanitized_url, "http://example.com/path?foo=bar");
    }

    #[test]
    fn parses_plain_https_url() {
        let parsed = parse_target_url("https://example.com/secure").unwrap();
        assert_eq!(parsed.scheme, "https");
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 443);
        assert_eq!(parsed.sanitized_url, "https://example.com/secure");
    }

    #[test]
    fn parses_url_with_explicit_port() {
        let parsed = parse_target_url("http://proxy.local:8080/").unwrap();
        assert_eq!(parsed.host, "proxy.local");
        assert_eq!(parsed.port, 8080);
        assert_eq!(parsed.sanitized_url, "http://proxy.local:8080/");
    }

    #[test]
    fn parses_ipv6_literal_with_brackets() {
        let parsed = parse_target_url("https://[2001:db8::1]:8443/api").unwrap();
        assert_eq!(parsed.host, "[2001:db8::1]");
        assert_eq!(parsed.port, 8443);
        assert_eq!(parsed.sanitized_url, "https://[2001:db8::1]:8443/api");
    }

    #[test]
    fn strips_credentials_from_target_url() {
        let parsed = parse_target_url("http://user:secret@example.com/info").unwrap();
        assert_eq!(parsed.host, "example.com");
        assert_eq!(parsed.port, 80);
        assert_eq!(parsed.sanitized_url, "http://example.com/info");
        assert!(!parsed.sanitized_url.contains("user"));
        assert!(!parsed.sanitized_url.contains("secret"));
    }

    #[test]
    fn rejects_unsupported_schemes() {
        assert!(parse_target_url("ftp://example.com").is_err());
        assert!(parse_target_url("file:///etc/hosts").is_err());
        assert!(parse_target_url("not_a_url").is_err());
    }

    #[test]
    fn formats_proxy_endpoint_correctly() {
        assert_eq!(
            format_proxy_endpoint("http", "proxy.com", 8080),
            "http://proxy.com:8080"
        );
        assert_eq!(
            format_proxy_endpoint("https", "2001:db8::1", 443),
            "https://[2001:db8::1]:443"
        );
        assert_eq!(format_proxy_endpoint("http", "[::1]", 3128), "http://[::1]:3128");
    }

    #[test]
    fn cancellation_flag_terminates_resolution() {
        let cancelled = AtomicBool::new(true);
        let deadline = Instant::now() + Duration::from_secs(10);
        let err = resolve("http://example.com", &cancelled, deadline).unwrap_err();
        assert_eq!(err.kind, ProviderErrorKind::Cancelled);
    }

    #[test]
    fn expired_deadline_terminates_resolution() {
        let cancelled = AtomicBool::new(false);
        let deadline = Instant::now() - Duration::from_secs(1);
        let err = resolve("http://example.com", &cancelled, deadline).unwrap_err();
        assert_eq!(err.kind, ProviderErrorKind::Timeout);
    }
}
