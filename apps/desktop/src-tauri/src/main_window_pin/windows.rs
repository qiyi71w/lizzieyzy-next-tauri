use ::windows::Win32::{
    Foundation::{GetLastError, SetLastError, ERROR_SUCCESS, HWND},
    UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, HWND_NOTOPMOST, HWND_TOPMOST, SWP_NOACTIVATE,
        SWP_NOMOVE, SWP_NOSIZE, WS_EX_TOPMOST,
    },
};
use tauri::WebviewWindow;

pub(super) fn set_pin(window: &WebviewWindow, value: bool) -> Result<(), String> {
    let hwnd = HWND(window.hwnd().map_err(|error| error.to_string())?.0);
    // Check the OS operation even when Tao's cached flag already matches the intent.
    unsafe {
        SetWindowPos(
            hwnd,
            Some(if value { HWND_TOPMOST } else { HWND_NOTOPMOST }),
            0,
            0,
            0,
            0,
            SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE,
        )
    }
    .map_err(|error| format!("Native pin apply failed: {error}"))?;
    // Keep Tao's window flags aligned; later window updates use that cache.
    window.set_always_on_top(value).map_err(|error| error.to_string())
}

pub(super) fn read_pin(window: &WebviewWindow) -> Result<bool, String> {
    // This getter fences queued UI-thread writes, but its cached boolean is not OS readback.
    window.is_always_on_top().map_err(|error| error.to_string())?;
    let hwnd = HWND(window.hwnd().map_err(|error| error.to_string())?.0);
    read_native(hwnd)
}

fn read_native(hwnd: HWND) -> Result<bool, String> {
    unsafe { SetLastError(ERROR_SUCCESS) };
    let flags = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
    let error = unsafe { GetLastError() };
    if flags == 0 && error != ERROR_SUCCESS {
        return Err(format!("Cannot read native pin state (Win32 error {}).", error.0));
    }
    Ok(flags & WS_EX_TOPMOST.0 as isize != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::windows::{
        core::w,
        Win32::{
            Foundation::ERROR_ACCESS_DENIED,
            UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, WS_OVERLAPPED},
        },
    };

    struct TestWindow(HWND);
    impl Drop for TestWindow {
        fn drop(&mut self) {
            let _ = unsafe { DestroyWindow(self.0) };
        }
    }

    #[test]
    fn keeps_unreadable_distinct_from_unpinned() {
        let window = TestWindow(
            unsafe {
                CreateWindowExW(
                    Default::default(),
                    w!("STATIC"),
                    w!("R7 pin readback regression"),
                    WS_OVERLAPPED,
                    0,
                    0,
                    10,
                    10,
                    None,
                    None,
                    None,
                    None,
                )
            }
            .expect("private hidden native window"),
        );
        // Zero extended styles are valid, regardless of the caller's previous last-error value.
        unsafe { SetLastError(ERROR_ACCESS_DENIED) };
        assert_eq!(read_native(window.0), Ok(false));
        let hwnd = window.0;
        drop(window);
        assert!(
            read_native(hwnd).is_err(),
            "destroyed native state must not become false"
        );
    }
}
