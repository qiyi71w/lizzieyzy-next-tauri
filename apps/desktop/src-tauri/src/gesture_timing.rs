use app_model::BoardGestureTimingDto;
use tauri::WebviewWindow;

#[tauri::command]
pub async fn board_gesture_timing(window: WebviewWindow) -> Result<BoardGestureTimingDto, String> {
    tauri::async_runtime::spawn_blocking(move || native_timing(&window))
        .await
        .map_err(|error| error.to_string())?
}

fn timing(milliseconds: f64) -> Result<BoardGestureTimingDto, String> {
    let delay = milliseconds.ceil();
    if !delay.is_finite() || delay < 0.0 || delay > i32::MAX as f64 {
        return Err("System double-click interval cannot be represented by the browser scheduler.".into());
    }
    Ok(BoardGestureTimingDto {
        double_click_interval_ms: delay as u32,
    })
}

#[cfg(windows)]
fn native_timing(_: &WebviewWindow) -> Result<BoardGestureTimingDto, String> {
    // Read the current setting; there is no app-owned fallback or preference.
    timing(unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetDoubleClickTime() } as f64)
}

#[cfg(target_os = "macos")]
fn native_timing(_: &WebviewWindow) -> Result<BoardGestureTimingDto, String> {
    timing(objc2_app_kit::NSEvent::doubleClickInterval() * 1000.0)
}

#[cfg(target_os = "linux")]
fn native_timing(window: &WebviewWindow) -> Result<BoardGestureTimingDto, String> {
    use gtk::prelude::*;
    use std::{sync::mpsc, time::Duration};
    let (tx, rx) = mpsc::sync_channel(1);
    let handle = window.clone();
    window
        .run_on_main_thread(move || {
            let result = handle
                .gtk_window()
                .map_err(|error| error.to_string())
                .and_then(|native| {
                    let settings = native
                        .settings()
                        .ok_or("Native GTK window settings unavailable.")?;
                    timing(settings.gtk_double_click_time() as f64)
                });
            let _ = tx.send(result);
        })
        .map_err(|error| error.to_string())?;
    // Same bounded UI-thread query bridge as window_geometry/linux.rs.
    rx.recv_timeout(Duration::from_secs(2))
        .map_err(|_| "Timed out reading the native GTK double-click interval.".to_string())?
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn native_timing(_: &WebviewWindow) -> Result<BoardGestureTimingDto, String> {
    Err("System double-click timing is unavailable on this platform.".into())
}

#[cfg(test)]
mod tests {
    use super::timing;

    #[test]
    fn gesture_timing_preserves_slow_system_values_and_never_rounds_down() {
        assert_eq!(timing(1250.0).unwrap().double_click_interval_ms, 1250);
        assert_eq!(timing(5000.0).unwrap().double_click_interval_ms, 5000);
        assert_eq!(timing(400.25).unwrap().double_click_interval_ms, 401);
        assert_eq!(timing(0.0).unwrap().double_click_interval_ms, 0);
        for invalid in [f64::NAN, f64::INFINITY, -1.0, i32::MAX as f64 + 1.0] {
            assert!(timing(invalid).is_err());
        }
    }
}
