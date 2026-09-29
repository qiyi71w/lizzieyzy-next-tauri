use std::sync::mpsc;
use std::time::Duration;

use objc2_app_kit::{NSWindow, NSWindowButton, NSWindowStyleMask};
use tauri::WebviewWindow;

const QUERY_TIMEOUT: Duration = Duration::from_secs(2);
const QUERY_ERROR: &str = "Cannot query native draggable title bounds; saved geometry was retained.";

pub(super) fn title_insets(window: &WebviewWindow, scale: f64) -> Result<(f64, f64, f64, f64), String> {
    // AppKit frame and view coordinates are already logical points; scale is
    // validated here but must not be applied a second time.
    if !scale.is_finite() || scale <= 0.0 {
        return Err(QUERY_ERROR.into());
    }

    let (tx, rx) = mpsc::sync_channel(1);
    let for_ui = window.clone();
    window
        .run_on_main_thread(move || {
            let result = measure_title(&for_ui);
            let _ = tx.send(result);
        })
        .map_err(|e| e.to_string())?;
    rx.recv_timeout(QUERY_TIMEOUT)
        .map_err(|_| QUERY_ERROR.to_string())?
}

fn measure_title(window: &WebviewWindow) -> Result<(f64, f64, f64, f64), String> {
    let raw = window.ns_window().map_err(|e| e.to_string())?;
    if raw.is_null() {
        return Err(QUERY_ERROR.into());
    }
    // Tauri's ns_window() returns the live NSWindow; the dispatched closure
    // keeps its WebviewWindow alive and confines all AppKit access to the UI thread.
    let native: &NSWindow = unsafe { &*raw.cast() };
    let style = native.styleMask();
    if !style.contains(NSWindowStyleMask::Titled)
        || style.contains(NSWindowStyleMask::FullSizeContentView)
        || style.contains(NSWindowStyleMask::FullScreen)
        || !native.isMovable()
    {
        return Err(QUERY_ERROR.into());
    }

    let frame = native.frame();
    let content = native.contentRectForFrameRect(frame);
    let width = frame.size.width;
    let height = frame.size.height;
    let content_top = content.origin.y + content.size.height - frame.origin.y;
    let title_height = height - content_top;
    if ![
        frame.origin.x,
        frame.origin.y,
        width,
        height,
        content.origin.x,
        content.origin.y,
        content.size.width,
        content.size.height,
        content_top,
        title_height,
    ]
    .into_iter()
    .all(f64::is_finite)
        || width <= 0.0
        || height <= 0.0
        || content.size.width <= 0.0
        || content.size.height <= 0.0
        || content.origin.x < frame.origin.x
        || content.origin.x + content.size.width > frame.origin.x + width
        || content.origin.y < frame.origin.y
        || content_top <= 0.0
        || title_height <= 0.0
        || title_height >= height
    {
        return Err(QUERY_ERROR.into());
    }

    let mut left: f64 = 0.0;
    for kind in [
        NSWindowButton::CloseButton,
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ] {
        let Some(button) = native.standardWindowButton(kind) else {
            continue;
        };
        if button.isHiddenOrHasHiddenAncestor() {
            continue;
        }
        // A nil destination converts the button's local bounds into window-base
        // points; these share the outer frame's width, but not its screen origin.
        let rect = button.convertRect_toView(button.bounds(), None);
        let right = rect.origin.x + rect.size.width;
        let top = height - (rect.origin.y + rect.size.height);
        if ![
            right,
            top,
            rect.origin.x,
            rect.origin.y,
            rect.size.width,
            rect.size.height,
        ]
        .into_iter()
        .all(f64::is_finite)
            || rect.size.width <= 0.0
            || rect.size.height <= 0.0
        {
            return Err(QUERY_ERROR.into());
        }
        if top < title_height && top + rect.size.height > 0.0 {
            left = left.max(right);
        }
    }
    if left < 0.0 || left >= width {
        return Err(QUERY_ERROR.into());
    }

    // AppKit exposes title and button geometry, not an equivalent of Win32's
    // WM_NCHITTEST; this is the contiguous space left after the visible controls.
    Ok((left, 0.0, 0.0, title_height))
}
