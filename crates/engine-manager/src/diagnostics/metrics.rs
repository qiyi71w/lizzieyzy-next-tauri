use super::now_ms;
use app_model::EngineDiagnosticMetricDto;

use parking_lot::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, LazyLock,
};
use std::time::{Duration, Instant};

struct Samples {
    values: Mutex<Option<(Instant, Vec<EngineDiagnosticMetricDto>)>>,
    requested: AtomicBool,
}
struct Sampler {
    samples: Arc<Samples>,
    sender: mpsc::SyncSender<()>,
}

/// One process-local worker, one admitted sample, no queued retries. Slow OS
/// calls cannot block UI/Stop/shutdown or accumulate workers. A sample older
/// than two seconds is explicitly missing, never presented as current data.
pub(super) fn collect() -> Vec<EngineDiagnosticMetricDto> {
    static SAMPLER: LazyLock<Sampler> = LazyLock::new(|| {
        let samples = Arc::new(Samples {
            values: Mutex::new(None),
            requested: AtomicBool::new(false),
        });
        let worker = Arc::clone(&samples);
        let (sender, receiver) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            while receiver.recv().is_ok() {
                let started = Instant::now();
                let values = sample();
                if started.elapsed() <= Duration::from_millis(250) {
                    *worker.values.lock() = Some((Instant::now(), values));
                } else {
                    *worker.values.lock() = None;
                }
                worker.requested.store(false, Ordering::Release);
            }
        });
        Sampler { samples, sender }
    });
    let sampler = &*SAMPLER;
    if !sampler.samples.requested.swap(true, Ordering::AcqRel) && sampler.sender.try_send(()).is_err() {
        sampler.samples.requested.store(false, Ordering::Release);
    }
    sampler
        .samples
        .values
        .lock()
        .as_ref()
        .filter(|(at, _)| at.elapsed() <= Duration::from_secs(2))
        .map(|(_, values)| values.clone())
        .unwrap_or_else(|| {
            metrics(
                None,
                None,
                None,
                None,
                "sample pending, unavailable or exceeded 250 ms collection deadline",
            )
        })
}

fn metrics(
    memory: Option<f64>,
    threads: Option<f64>,
    app_disk: Option<f64>,
    temp_disk: Option<f64>,
    missing: &str,
) -> Vec<EngineDiagnosticMetricDto> {
    let at_ms = now_ms();
    [
        ("application-process", "resident-memory", "bytes", memory),
        ("application-process", "threads", "count", threads),
        ("application-directory", "available-space", "bytes", app_disk),
        ("temporary-directory", "available-space", "bytes", temp_disk),
    ]
    .into_iter()
    .map(|(role, name, unit, value)| EngineDiagnosticMetricDto {
        role: role.into(),
        name: name.into(),
        unit: unit.into(),
        at_ms,
        value,
        missing: value.is_none().then(|| missing.into()),
    })
    .collect()
}
fn sample() -> Vec<EngineDiagnosticMetricDto> {
    let (memory, threads) = process_sample();
    let app_disk = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().and_then(disk_space));
    let temp_disk = disk_space(&std::env::temp_dir());
    metrics(memory, threads, app_disk, temp_disk, "OS metric unavailable")
}

#[cfg(target_os = "linux")]
fn process_sample() -> (Option<f64>, Option<f64>) {
    use std::io::Read;
    let mut text = String::new();
    let result = std::fs::File::open("/proc/self/status")
        .and_then(|file| file.take(64 * 1024).read_to_string(&mut text));
    if result.is_err() {
        return (None, None);
    }
    let number = |key: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(key))
            .and_then(|value| value.split_whitespace().next())
            .and_then(|value| value.parse::<u64>().ok())
    };
    (
        number("VmRSS:").map(|value| value as f64 * 1024.0),
        number("Threads:").map(|value| value as f64),
    )
}

#[cfg(windows)]
fn process_sample() -> (Option<f64>, Option<f64>) {
    use windows_sys::Win32::System::{
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
        Threading::GetCurrentProcess,
    };
    let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { std::mem::zeroed() };
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    // A current-process pseudo-handle is always valid and must not be closed.
    let success = unsafe {
        GetProcessMemoryInfo(
            GetCurrentProcess(),
            &mut counters,
            std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };
    ((success != 0).then_some(counters.WorkingSetSize as f64), None)
}

#[cfg(not(any(target_os = "linux", windows)))]
fn process_sample() -> (Option<f64>, Option<f64>) {
    (None, None)
}

#[cfg(unix)]
fn disk_space(path: &std::path::Path) -> Option<f64> {
    use std::os::unix::ffi::OsStrExt;
    let path = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut result = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // Both pointers remain valid for the OS call; output is read only on success.
    if unsafe { libc::statvfs(path.as_ptr(), result.as_mut_ptr()) } != 0 {
        return None;
    }
    let result = unsafe { result.assume_init() };
    Some(result.f_bavail as f64 * result.f_frsize as f64)
}
#[cfg(windows)]
fn disk_space(path: &std::path::Path) -> Option<f64> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut available = 0u64;
    // NUL-terminated path and output storage live through this synchronous call.
    let ok = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut available,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    (ok != 0).then_some(available as f64)
}
#[cfg(not(any(unix, windows)))]
fn disk_space(_: &std::path::Path) -> Option<f64> {
    None
}
