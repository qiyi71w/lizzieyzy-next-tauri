//! Main-window OS adaptation. A single actor owns sampling and saves; no native
//! call is made while holding the shared preferences transaction mutex.
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

use app_model::{WindowGeometryDto, WindowGeometryStatusDto};
use app_preferences::window_geometry::{
    fit_default, record_sample, restore_geometry, FrameInsets, WindowSample, WorkArea,
};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

use crate::continuous_analysis::PreferencesState;
use engine_manager::ForegroundEngineManager;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::title_insets;

const DEBOUNCE: Duration = Duration::from_millis(500);
const SETTLE: Duration = Duration::from_millis(75);
const FLUSH_TIMEOUT: Duration = Duration::from_secs(5);
const EVENT: &str = "window-geometry://status";
type Reply = Sender<Result<WindowGeometryStatusDto, String>>;

pub struct WindowGeometryOwner(Sender<Request>);
enum Request {
    Observe,
    Status(Reply),
    Reset(Reply),
    Flush { seal: bool, reply: Reply },
    Retry(Reply),
    Freeze(bool, Reply),
}

struct Session {
    app: AppHandle,
    window: WebviewWindow,
    geometry: Option<WindowGeometryDto>,
    durable: Option<WindowGeometryDto>,
    phase: &'static str,
    error: Option<String>,
    due: Option<Instant>,
    sample_due: Option<Instant>,
    frozen: bool,
    sealed: bool,
    suspended: bool,
}

fn work_area(monitor: &tauri::Monitor) -> WorkArea {
    let area = monitor.work_area();
    WorkArea {
        x: f64::from(area.position.x),
        y: f64::from(area.position.y),
        width: f64::from(area.size.width),
        height: f64::from(area.size.height),
        scale_factor: monitor.scale_factor(),
    }
}

#[cfg(windows)]
fn is_caption(hwnd: windows::Win32::Foundation::HWND, x: i32, y: i32) -> Result<bool, String> {
    use windows::Win32::{
        Foundation::{LPARAM, WPARAM},
        UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HTCAPTION, SMTO_ABORTIFHUNG, SMTO_ERRORONEXIT, WM_NCHITTEST,
        },
    };
    let point = ((y as u32 & 0xffff) << 16) | (x as u32 & 0xffff);
    let mut hit = 0;
    let result = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_NCHITTEST,
            WPARAM(0),
            LPARAM(point as isize),
            SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
            1000,
            Some(&mut hit),
        )
    };
    if result.0 == 0 {
        return Err("Cannot hit-test native title bounds; saved geometry was retained.".into());
    }
    Ok(hit == HTCAPTION as usize)
}

#[cfg(windows)]
fn title_insets(window: &WebviewWindow, scale: f64) -> Result<(f64, f64, f64, f64), String> {
    use windows::Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging::{
            SendMessageTimeoutW, SMTO_ABORTIFHUNG, SMTO_ERRORONEXIT, TITLEBARINFOEX, WM_GETTITLEBARINFOEX,
        },
    };
    let origin = window.outer_position().map_err(|e| e.to_string())?;
    let outer = window.outer_size().map_err(|e| e.to_string())?;
    let hwnd = HWND(window.hwnd().map_err(|e| e.to_string())?.0);
    let mut info = TITLEBARINFOEX {
        cbSize: std::mem::size_of::<TITLEBARINFOEX>() as u32,
        ..Default::default()
    };
    // No preference mutex is held while the UI thread handles this native query.
    let result = unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_GETTITLEBARINFOEX,
            WPARAM(0),
            LPARAM((&mut info as *mut TITLEBARINFOEX) as isize),
            SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
            1000,
            None,
        )
    };
    if result.0 == 0 || info.rcTitleBar.right <= info.rcTitleBar.left {
        return Err("Cannot query native draggable title bounds; saved geometry was retained.".into());
    }
    let mut right = info.rcTitleBar.right;
    for index in [2, 3, 4, 5] {
        let rect = info.rgrect[index];
        if info.rgstate[index] & 0x8000 == 0 && rect.right > rect.left {
            right = right.min(rect.left);
        }
    }
    // TITLEBARINFOEX includes resize pixels and can overlap the system menu.
    // Trim those native hit-test boundaries rather than inventing a caption height.
    let mut left = info.rcTitleBar.left;
    let mut top = info.rcTitleBar.top;
    let mut bottom = info.rcTitleBar.bottom;
    let middle_x = left + (right - left) / 2;
    while top < bottom && !is_caption(hwnd, middle_x, top)? {
        top += 1;
    }
    while bottom > top && !is_caption(hwnd, middle_x, bottom - 1)? {
        bottom -= 1;
    }
    if bottom <= top || right <= left {
        return Err("Native title has no draggable region; saved geometry was retained.".into());
    }
    let middle_y = top + (bottom - top) / 2;
    while left < right
        && !(is_caption(hwnd, left, top)?
            && is_caption(hwnd, left, middle_y)?
            && is_caption(hwnd, left, bottom - 1)?)
    {
        left += 1;
    }
    while right > left
        && !(is_caption(hwnd, right - 1, top)?
            && is_caption(hwnd, right - 1, middle_y)?
            && is_caption(hwnd, right - 1, bottom - 1)?)
    {
        right -= 1;
    }
    if right <= left {
        return Err("Native title has no draggable width; saved geometry was retained.".into());
    }
    Ok((
        f64::from(left - origin.x).max(0.0) / scale,
        (f64::from(origin.x) + f64::from(outer.width) - f64::from(right)).max(0.0) / scale,
        f64::from(top - origin.y).max(0.0) / scale,
        f64::from(bottom - top) / scale,
    ))
}

#[cfg(not(target_os = "linux"))]
fn frame(window: &WebviewWindow) -> Result<FrameInsets, String> {
    let outer = window.outer_size().map_err(|e| e.to_string())?;
    let inner = window.inner_size().map_err(|e| e.to_string())?;
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let (title_left, title_right, title_top, title_height) = title_insets(window, scale)?;
    Ok(FrameInsets {
        width: f64::from(outer.width.saturating_sub(inner.width)) / scale,
        height: f64::from(outer.height.saturating_sub(inner.height)) / scale,
        title_left,
        title_right,
        title_top,
        title_height,
    })
}

#[cfg(target_os = "linux")]
use linux::frame;

fn usable_area(area: &WorkArea, frame: FrameInsets) -> bool {
    area.scale_factor.is_finite()
        && area.scale_factor > 0.0
        && area.width / area.scale_factor > frame.width + 100.0
        && area.height / area.scale_factor > frame.height + 32.0
}

fn sample(
    window: &WebviewWindow,
    previous: Option<WindowGeometryDto>,
) -> Result<Option<WindowGeometryDto>, String> {
    if window.is_minimized().map_err(|e| e.to_string())? {
        return Ok(record_sample(previous, WindowSample::Minimized));
    }
    if window.is_maximized().map_err(|e| e.to_string())? {
        return Ok(record_sample(previous, WindowSample::Maximized));
    }
    #[cfg(target_os = "linux")]
    let geometry = linux::normal_bounds(window)?;
    #[cfg(not(target_os = "linux"))]
    let geometry = {
        let origin = window.outer_position().map_err(|e| e.to_string())?;
        let client = window.inner_size().map_err(|e| e.to_string())?;
        let scale = window.scale_factor().map_err(|e| e.to_string())?;
        WindowGeometryDto {
            x: Some(f64::from(origin.x)),
            y: Some(f64::from(origin.y)),
            width: f64::from(client.width) / scale,
            height: f64::from(client.height) / scale,
            scale_factor: scale,
            maximized: false,
        }
    };
    // Recheck state after the separate native queries; transient minimize/maximize
    // bounds must never replace normal bounds.
    if window.is_minimized().map_err(|e| e.to_string())?
        || window.is_maximized().map_err(|e| e.to_string())?
    {
        return Ok(previous);
    }
    Ok(record_sample(previous, WindowSample::Normal(geometry)))
}

fn apply(window: &WebviewWindow, geometry: WindowGeometryDto) -> Result<(), String> {
    window.unmaximize().map_err(|e| e.to_string())?;
    // Move first so the client setter uses the destination DPI. Never pass outer
    // dimensions to this setter.
    if let (Some(x), Some(y)) = (geometry.x, geometry.y) {
        #[cfg(target_os = "linux")]
        linux::set_position(window, x, y)?;
        #[cfg(not(target_os = "linux"))]
        window
            .set_position(tauri::PhysicalPosition::new(x as i32, y as i32))
            .map_err(|e| e.to_string())?;
    }
    window
        .set_size(tauri::LogicalSize::new(geometry.width, geometry.height))
        .map_err(|e| e.to_string())?;
    if geometry.maximized {
        window.maximize().map_err(|e| e.to_string())?;
    }
    Ok(())
}

impl Session {
    fn status(&self) -> WindowGeometryStatusDto {
        WindowGeometryStatusDto {
            phase: self.phase.into(),
            geometry: self.geometry,
            error: self.error.clone(),
        }
    }
    fn publish(&self) {
        let _ = self.app.emit(EVENT, self.status());
    }
    fn fail(&mut self, error: String) {
        self.phase = "unsaved";
        self.error = Some(error);
        self.due = None;
        self.publish();
    }
    fn initialize(&mut self) -> Result<(), String> {
        let path = crate::app_preferences_path(&self.app)?;
        let loaded = self
            .app
            .state::<PreferencesState>()
            .load(&path, &self.app.state::<ForegroundEngineManager>())?;
        self.durable = loaded.preferences.window_geometry;
        #[cfg(target_os = "linux")]
        if linux::position_is_managed(&self.window)? {
            let current = linux::normal_bounds(&self.window)?;
            let target = app_preferences::window_geometry::restore_managed_geometry(self.durable, current);
            if current != target {
                apply(&self.window, target)?;
            }
            self.geometry = Some(target);
            self.suspended = false;
            self.error = None;
            self.phase = if self.geometry == self.durable {
                "saved"
            } else {
                "pending"
            };
            self.due = (self.geometry != self.durable).then(|| Instant::now() + DEBOUNCE);
            return Ok(());
        }
        let frame = frame(&self.window)?;
        let areas = self
            .window
            .available_monitors()
            .map(|monitors| monitors.iter().map(work_area).collect::<Vec<_>>())
            .map_err(|e| e.to_string());
        let current = sample(&self.window, None)?;
        let target = restore_geometry(
            self.durable.or(current),
            areas.as_deref().map_err(Clone::clone),
            frame,
            || {
                self.window
                    .primary_monitor()
                    .map_err(|e| e.to_string())?
                    .map(|monitor| work_area(&monitor))
                    .ok_or_else(|| "Cannot query primary monitor; saved geometry was retained.".into())
            },
        )?;
        if current != Some(target.geometry) {
            apply(&self.window, target.geometry)?;
        }
        self.geometry = Some(target.geometry);
        if let Some(warning) = target.warning {
            self.suspended = true;
            return Err(warning);
        }
        self.suspended = false;
        self.error = None;
        self.phase = if self.geometry == self.durable {
            "saved"
        } else {
            "pending"
        };
        self.due = (self.geometry != self.durable).then(|| Instant::now() + DEBOUNCE);
        Ok(())
    }
    fn observe(&mut self) -> Result<(), String> {
        if self.sealed || self.suspended {
            return Ok(());
        }
        let next = sample(&self.window, self.geometry)?;
        if next != self.geometry {
            self.geometry = next;
            self.error = None;
            self.phase = "pending";
            self.due = Some(Instant::now() + DEBOUNCE);
            self.publish();
        }
        Ok(())
    }
    fn persist(&mut self) -> Result<(), String> {
        if self.suspended {
            return Err(self
                .error
                .clone()
                .unwrap_or_else(|| "Window geometry is unavailable.".into()));
        }
        self.due = None;
        if self.geometry == self.durable {
            self.phase = "saved";
            self.error = None;
            self.publish();
            return Ok(());
        }
        let geometry = self.geometry.ok_or("No valid normal window bounds available.")?;
        self.phase = "saving";
        self.publish();
        self.app
            .state::<PreferencesState>()
            .update_window_geometry(&crate::app_preferences_path(&self.app)?, geometry)?;
        self.durable = Some(geometry);
        self.phase = "saved";
        self.error = None;
        self.publish();
        Ok(())
    }
    fn reset(&mut self) -> Result<(), String> {
        if self.frozen || self.sealed {
            return Err("Window changes are frozen during document departure.".into());
        }
        #[cfg(target_os = "linux")]
        if linux::position_is_managed(&self.window)? {
            self.window.unmaximize().map_err(|e| e.to_string())?;
            let target = linux::managed_reset_bounds(&self.window)?;
            apply(&self.window, target)?;
            self.geometry = Some(target);
            self.suspended = false;
            self.error = None;
            self.phase = "pending";
            self.due = Some(Instant::now() + DEBOUNCE);
            self.publish();
            return Ok(());
        }
        let monitor = match self.window.current_monitor().map_err(|e| e.to_string())? {
            Some(monitor) => monitor,
            None => self
                .window
                .primary_monitor()
                .map_err(|e| e.to_string())?
                .ok_or("Cannot query a monitor for window reset.")?,
        };
        // Measure non-client dimensions in normal state, not maximized frame metrics.
        self.window.unmaximize().map_err(|e| e.to_string())?;
        let frame = frame(&self.window)?;
        let area = work_area(&monitor);
        if !usable_area(&area, frame) {
            return Err("Current monitor work area is unusable.".into());
        }
        let target = fit_default(&area, frame);
        apply(&self.window, target)?;
        self.geometry = Some(target);
        self.suspended = false;
        self.error = None;
        self.phase = "pending";
        self.due = Some(Instant::now() + DEBOUNCE);
        self.publish();
        Ok(())
    }
    fn respond(&mut self, result: Result<(), String>, reply: Reply) {
        match result {
            Ok(()) => {
                let _ = reply.send(Ok(self.status()));
            }
            Err(error) => {
                self.fail(error.clone());
                let _ = reply.send(Err(error));
            }
        }
    }
    fn run(mut self, requests: Receiver<Request>, startup: Result<(), String>) {
        if let Err(error) = startup.and_then(|_| self.initialize()) {
            self.suspended = true;
            self.fail(error);
        }
        // Even preference/monitor/native failures must not strand a hidden main window.
        if let Err(error) = self.window.show() {
            self.fail(error.to_string());
        }
        self.publish();
        loop {
            let timeout = [self.sample_due, if self.frozen { None } else { self.due }]
                .into_iter()
                .flatten()
                .min()
                .map(|due| due.saturating_duration_since(Instant::now()));
            let request = match timeout {
                Some(timeout) => requests.recv_timeout(timeout),
                None => requests.recv().map_err(|_| mpsc::RecvTimeoutError::Disconnected),
            };
            match request {
                Ok(Request::Observe) if !self.sealed => self.sample_due = Some(Instant::now() + SETTLE),
                Ok(Request::Observe) => {}
                Ok(Request::Status(reply)) => {
                    let _ = reply.send(Ok(self.status()));
                }
                Ok(Request::Freeze(frozen, reply)) => {
                    self.frozen = frozen;
                    self.respond(Ok(()), reply);
                }
                Ok(Request::Reset(reply)) => {
                    let result = self.reset();
                    self.respond(result, reply);
                }
                Ok(Request::Retry(reply)) => {
                    let result = if self.suspended {
                        self.initialize()
                    } else {
                        self.observe()
                    }
                    .and_then(|_| self.persist());
                    self.respond(result, reply);
                }
                Ok(Request::Flush { seal, reply }) => {
                    match self.observe().and_then(|_| self.persist()) {
                        Ok(()) => {
                            // A caller that timed out must remain able to cancel and move again.
                            if reply.send(Ok(self.status())).is_ok() && seal {
                                self.sealed = true;
                                self.sample_due = None;
                            }
                        }
                        Err(error) => self.respond(Err(error), reply),
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
            let now = Instant::now();
            if self.sample_due.is_some_and(|due| due <= now) {
                self.sample_due = None;
                if let Err(error) = self.observe() {
                    self.fail(error);
                }
            }
            if !self.frozen && self.due.is_some_and(|due| due <= now) {
                if let Err(error) = self.persist() {
                    self.fail(error);
                }
            }
        }
    }
}

pub fn start(app: &AppHandle) {
    let (sender, receiver) = mpsc::channel();
    app.manage(WindowGeometryOwner(sender));
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    #[cfg(target_os = "linux")]
    let startup = linux::configure(&window);
    #[cfg(not(target_os = "linux"))]
    let startup = Ok(());
    let session = Session {
        app: app.clone(),
        window,
        geometry: None,
        durable: None,
        phase: "loading",
        error: None,
        due: None,
        sample_due: None,
        frozen: false,
        sealed: false,
        suspended: true,
    };
    std::thread::spawn(move || session.run(receiver, startup));
}

pub fn observe(app: &AppHandle) {
    if let Some(owner) = app.try_state::<WindowGeometryOwner>() {
        let _ = owner.0.send(Request::Observe);
    }
}

async fn request(
    app: AppHandle,
    make: impl FnOnce(Reply) -> Request + Send + 'static,
) -> Result<WindowGeometryStatusDto, String> {
    let sender = app.state::<WindowGeometryOwner>().0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (reply, result) = mpsc::channel();
        sender
            .send(make(reply))
            .map_err(|_| "Window geometry owner stopped.".to_string())?;
        result.recv_timeout(FLUSH_TIMEOUT).map_err(|_| {
            "Window geometry save did not finish within five seconds. Retry or cancel close.".to_string()
        })?
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn window_geometry_status(app: AppHandle) -> Result<WindowGeometryStatusDto, String> {
    request(app, Request::Status).await
}
#[tauri::command]
pub async fn reset_window_geometry(app: AppHandle) -> Result<WindowGeometryStatusDto, String> {
    request(app, Request::Reset).await
}
#[tauri::command]
pub async fn retry_window_geometry(app: AppHandle) -> Result<WindowGeometryStatusDto, String> {
    request(app, Request::Retry).await
}
#[tauri::command]
pub async fn flush_window_geometry(app: AppHandle) -> Result<(), String> {
    request(app, |reply| Request::Flush { seal: false, reply })
        .await
        .map(|_| ())
}
#[tauri::command]
pub async fn freeze_window_geometry(app: AppHandle, frozen: bool) -> Result<(), String> {
    request(app, move |reply| Request::Freeze(frozen, reply))
        .await
        .map(|_| ())
}
pub async fn seal_for_exit(app: AppHandle) -> Result<(), String> {
    request(app, |reply| Request::Flush { seal: true, reply })
        .await
        .map(|_| ())
}
