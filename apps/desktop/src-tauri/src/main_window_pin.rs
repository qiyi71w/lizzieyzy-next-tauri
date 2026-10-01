use crate::continuous_analysis::PreferencesState;
use app_model::MainWindowPinStatusDto;
use std::{path::Path, sync::Mutex};
use tauri::{AppHandle, Manager};

#[cfg(windows)]
mod windows;

pub trait PinWindow {
    fn set_pin(&self, value: bool) -> Result<(), String>;
    fn read_pin(&self) -> Result<bool, String>;
}

impl PinWindow for tauri::WebviewWindow {
    fn set_pin(&self, value: bool) -> Result<(), String> {
        #[cfg(windows)]
        {
            windows::set_pin(self, value)
        }
        #[cfg(not(windows))]
        {
            self.set_always_on_top(value).map_err(|error| error.to_string())
        }
    }
    fn read_pin(&self) -> Result<bool, String> {
        #[cfg(windows)]
        {
            windows::read_pin(self)
        }
        #[cfg(not(windows))]
        {
            self.is_always_on_top().map_err(|error| error.to_string())
        }
    }
}

#[derive(Default)]
pub struct MainWindowPin(Mutex<Option<MainWindowPinStatusDto>>);

impl MainWindowPin {
    // The pin lock serializes native operations, never the preferences mutex.
    pub fn apply(
        &self,
        window: &impl PinWindow,
        preferences: &PreferencesState,
        path: &Path,
        target: Option<bool>,
    ) -> Result<MainWindowPinStatusDto, String> {
        let mut status = self.0.lock().expect("main window pin transaction");
        let durable = preferences.pin_intent()?;
        let requested = target.unwrap_or(durable);
        let previous = match window.read_pin() {
            Ok(value) => value,
            Err(failure) if target.is_some() => {
                let result = MainWindowPinStatusDto {
                    durable,
                    actual: None,
                    error: Some(format!("Cannot read pre-transaction native state: {failure}")),
                };
                *status = Some(result.clone());
                return Ok(result);
            }
            Err(_) => durable,
        };
        let mut error = window.set_pin(requested).err();
        let mut actual = window
            .read_pin()
            .map_err(|failure| {
                error = Some(format!(
                    "{}Native state unknown: {failure}",
                    error
                        .as_ref()
                        .map(|value| format!("{value}; "))
                        .unwrap_or_default()
                ));
            })
            .ok();
        if error.is_none() && actual != Some(requested) {
            error = Some("Native pin state did not match the requested value.".into());
        }
        let mut committed = durable;
        if error.is_none() && target.is_some() {
            match preferences.save_pin(path, requested) {
                Ok(()) => committed = requested,
                Err(failure) => {
                    let rollback = window.set_pin(previous).err();
                    error = Some(format!(
                        "Preferences write failed: {failure}{}",
                        rollback
                            .map(|value| format!("; rollback failed: {value}"))
                            .unwrap_or_default()
                    ));
                    actual = match window.read_pin() {
                        Ok(value) => Some(value),
                        Err(failure) => {
                            error = Some(format!("{}; native state unknown: {failure}", error.unwrap()));
                            None
                        }
                    };
                }
            }
        }
        let result = MainWindowPinStatusDto {
            durable: committed,
            actual,
            error,
        };
        *status = Some(result.clone());
        Ok(result)
    }

    pub fn status(&self) -> Result<MainWindowPinStatusDto, String> {
        self.0
            .lock()
            .expect("main window pin transaction")
            .clone()
            .ok_or_else(|| "Main window pin has not initialized.".into())
    }
}

#[tauri::command]
pub async fn main_window_pin_status(app: AppHandle) -> Result<MainWindowPinStatusDto, String> {
    tauri::async_runtime::spawn_blocking(move || app.state::<MainWindowPin>().status())
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_main_window_pin(
    app: AppHandle,
    value: Option<bool>,
) -> Result<MainWindowPinStatusDto, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let window = app.get_webview_window("main").ok_or("Main window unavailable.")?;
        app.state::<MainWindowPin>().apply(
            &window,
            &app.state::<PreferencesState>(),
            &crate::app_preferences_path(&app)?,
            value,
        )
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
#[path = "main_window_pin_tests.rs"]
mod tests;
