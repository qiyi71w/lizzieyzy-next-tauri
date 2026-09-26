use app_model::{FileActivationDeliveryDto, FileActivationRejectionDto};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub const FILE_ACTIVATION_AVAILABLE_EVENT: &str = "file-activation://available";
pub const FILE_ACTIVATION_REJECTED_EVENT: &str = "file-activation://rejected";

const BUSY_MESSAGE: &str = "Another file action is active; the external open request was rejected.";
const MULTIPLE_MESSAGE: &str = "Opening multiple files is unavailable; open one SGF or GIB at a time.";
const DIRECTORY_MESSAGE: &str = "Opening a directory is unavailable; choose one SGF or GIB file.";
const UNSUPPORTED_MESSAGE: &str = "This file type is unsupported; choose an SGF, TXT, or GIB file.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WarmAdmission {
    FocusOnly,
    Available,
    Rejected(String),
}

#[derive(Debug)]
enum ActivationIntent {
    FocusOnly,
    Open(PathBuf),
    Rejected(String),
}

#[derive(Debug)]
struct ActivationState {
    next_request_id: u64,
    startup: Option<FileActivationDeliveryDto>,
    startup_taken: bool,
    warm: Option<FileActivationDeliveryDto>,
    ready: bool,
    file_flow_busy: bool,
}

#[derive(Debug)]
pub struct FileActivationOwner {
    state: Mutex<ActivationState>,
}

impl FileActivationOwner {
    pub fn from_process() -> Self {
        let args = std::env::args_os().collect::<Vec<_>>();
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::from_launch_args(args, &cwd)
    }

    pub fn from_launch_args(args: Vec<OsString>, cwd: &Path) -> Self {
        let mut state = ActivationState {
            next_request_id: 1,
            startup: None,
            startup_taken: false,
            warm: None,
            ready: false,
            file_flow_busy: false,
        };
        state.startup = delivery_for_intent(classify_arguments(args, cwd), &mut state.next_request_id);
        Self {
            state: Mutex::new(state),
        }
    }

    pub fn take_startup(&self) -> Option<FileActivationDeliveryDto> {
        let mut state = self.state.lock().expect("file activation state poisoned");
        state.startup_taken = true;
        state.startup.take()
    }

    pub fn take_warm(&self) -> Option<FileActivationDeliveryDto> {
        self.state
            .lock()
            .expect("file activation state poisoned")
            .warm
            .take()
    }

    pub fn mark_ready(&self) {
        self.state.lock().expect("file activation state poisoned").ready = true;
    }

    pub fn set_file_flow_busy(&self, busy: bool) {
        self.state
            .lock()
            .expect("file activation state poisoned")
            .file_flow_busy = busy;
    }

    pub fn admit_warm(&self, args: Vec<OsString>, cwd: &Path) -> WarmAdmission {
        self.admit_intent(classify_arguments(args, cwd))
    }

    pub fn admit_drop(&self, paths: Vec<PathBuf>) -> WarmAdmission {
        self.admit_intent(classify_paths(paths))
    }

    fn admit_intent(&self, intent: ActivationIntent) -> WarmAdmission {
        let mut state = self.state.lock().expect("file activation state poisoned");
        match intent {
            ActivationIntent::FocusOnly => WarmAdmission::FocusOnly,
            ActivationIntent::Rejected(message) => WarmAdmission::Rejected(message),
            ActivationIntent::Open(path) => {
                if !state.ready
                    || state.startup.is_some()
                    || !state.startup_taken
                    || state.file_flow_busy
                    || state.warm.is_some()
                {
                    return WarmAdmission::Rejected(BUSY_MESSAGE.to_string());
                }
                let request_id = state.next_request_id;
                state.next_request_id = state.next_request_id.saturating_add(1);
                state.warm = Some(FileActivationDeliveryDto::Open {
                    request_id,
                    path: display_path(&path),
                });
                state.file_flow_busy = true;
                WarmAdmission::Available
            }
        }
    }
}

fn classify_arguments(args: Vec<OsString>, cwd: &Path) -> ActivationIntent {
    let inputs = args.into_iter().skip(1).collect::<Vec<_>>();
    if inputs.is_empty() {
        return ActivationIntent::FocusOnly;
    }
    classify_paths(
        inputs
            .into_iter()
            .map(PathBuf::from)
            .map(|path| if path.is_absolute() { path } else { cwd.join(path) })
            .collect(),
    )
}

fn classify_paths(mut paths: Vec<PathBuf>) -> ActivationIntent {
    if paths.len() != 1 {
        return ActivationIntent::Rejected(MULTIPLE_MESSAGE.to_string());
    }
    let path = paths.pop().expect("one path checked above");
    if path.is_dir() {
        return ActivationIntent::Rejected(DIRECTORY_MESSAGE.to_string());
    }
    let supported = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "sgf" | "txt" | "gib"));
    if !supported {
        return ActivationIntent::Rejected(UNSUPPORTED_MESSAGE.to_string());
    }
    ActivationIntent::Open(path)
}

fn delivery_for_intent(
    intent: ActivationIntent,
    next_request_id: &mut u64,
) -> Option<FileActivationDeliveryDto> {
    match intent {
        ActivationIntent::FocusOnly => None,
        ActivationIntent::Rejected(message) => Some(FileActivationDeliveryDto::Rejected { message }),
        ActivationIntent::Open(path) => {
            let request_id = *next_request_id;
            *next_request_id = next_request_id.saturating_add(1);
            Some(FileActivationDeliveryDto::Open {
                request_id,
                path: display_path(&path),
            })
        }
    }
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn handle_second_instance(app: &AppHandle, args: Vec<String>, cwd: String) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    let owner = app.state::<FileActivationOwner>();
    emit_admission(
        app,
        owner.admit_warm(args.into_iter().map(OsString::from).collect(), Path::new(&cwd)),
    );
}

pub fn handle_file_drop(app: &AppHandle, paths: Vec<PathBuf>) {
    let owner = app.state::<FileActivationOwner>();
    emit_admission(app, owner.admit_drop(paths));
}

fn emit_admission(app: &AppHandle, admission: WarmAdmission) {
    match admission {
        WarmAdmission::FocusOnly => {}
        WarmAdmission::Available => {
            let _ = app.emit(FILE_ACTIVATION_AVAILABLE_EVENT, ());
        }
        WarmAdmission::Rejected(message) => {
            let _ = app.emit(
                FILE_ACTIVATION_REJECTED_EVENT,
                FileActivationRejectionDto { message },
            );
        }
    }
}

#[tauri::command]
pub fn take_initial_file_activation(
    owner: State<'_, FileActivationOwner>,
) -> Option<FileActivationDeliveryDto> {
    owner.take_startup()
}

#[tauri::command]
pub fn take_pending_file_activation(
    owner: State<'_, FileActivationOwner>,
) -> Option<FileActivationDeliveryDto> {
    owner.take_warm()
}

#[tauri::command]
pub fn mark_file_activation_ready(owner: State<'_, FileActivationOwner>) {
    owner.mark_ready();
}

#[tauri::command]
pub fn set_file_activation_busy(owner: State<'_, FileActivationOwner>, busy: bool) {
    owner.set_file_flow_busy(busy);
}

#[cfg(test)]
mod tests {
    use super::*;
    use app_model::FileActivationDeliveryDto;
    use std::ffi::OsString;
    use std::path::Path;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn launch_retains_one_relative_unicode_file_until_taken_once() {
        let owner = FileActivationOwner::from_launch_args(
            args(&["lizzieyzy-next-desktop.exe", "棋谱 files/game one.gib"]),
            Path::new("C:/requesting cwd"),
        );

        assert!(matches!(
            owner.take_startup(),
            Some(FileActivationDeliveryDto::Open { request_id: 1, path })
                if Path::new(&path) == Path::new("C:/requesting cwd/棋谱 files/game one.gib")
        ));
        assert_eq!(owner.take_startup(), None);
    }

    #[test]
    fn invalid_and_multiple_launch_inputs_are_explanatory() {
        let unsupported =
            FileActivationOwner::from_launch_args(args(&["app", "game.pdf"]), Path::new("C:/games"));
        assert!(matches!(
            unsupported.take_startup(),
            Some(FileActivationDeliveryDto::Rejected { message }) if message.contains("unsupported")
        ));

        let multiple = FileActivationOwner::from_launch_args(
            args(&["app", "one.sgf", "two.gib"]),
            Path::new("C:/games"),
        );
        assert!(matches!(
            multiple.take_startup(),
            Some(FileActivationDeliveryDto::Rejected { message }) if message.contains("multiple")
        ));

        let directory = FileActivationOwner::from_launch_args(
            vec![
                OsString::from("app"),
                std::env::current_dir().unwrap().into_os_string(),
            ],
            Path::new("."),
        );
        assert!(matches!(
            directory.take_startup(),
            Some(FileActivationDeliveryDto::Rejected { message }) if message.contains("directory")
        ));
    }

    #[test]
    fn warm_activation_is_admitted_once_only_when_ready_and_idle() {
        let owner = FileActivationOwner::from_launch_args(args(&["app"]), Path::new("C:/games"));
        assert!(matches!(
            owner.admit_warm(args(&["app", "one.sgf"]), Path::new("D:/source")),
            WarmAdmission::Rejected(_)
        ));

        assert_eq!(owner.take_startup(), None);
        owner.mark_ready();
        assert_eq!(
            owner.admit_warm(args(&["app", "one.sgf"]), Path::new("D:/source")),
            WarmAdmission::Available
        );
        assert!(matches!(
            owner.admit_warm(args(&["app", "two.sgf"]), Path::new("D:/source")),
            WarmAdmission::Rejected(_)
        ));
        assert!(matches!(
            owner.take_warm(),
            Some(FileActivationDeliveryDto::Open { request_id: 1, path })
                if Path::new(&path) == Path::new("D:/source/one.sgf")
        ));
        assert_eq!(owner.take_warm(), None);
    }

    #[test]
    fn analysis_does_not_make_admission_busy_but_file_flow_does() {
        let owner = FileActivationOwner::from_launch_args(args(&["app"]), Path::new("C:/games"));
        assert_eq!(owner.take_startup(), None);
        owner.mark_ready();
        owner.set_file_flow_busy(true);
        assert!(matches!(
            owner.admit_warm(args(&["app", "one.sgf"]), Path::new("C:/games")),
            WarmAdmission::Rejected(_)
        ));
        owner.set_file_flow_busy(false);
        assert_eq!(
            owner.admit_warm(args(&["app", "one.sgf"]), Path::new("C:/games")),
            WarmAdmission::Available
        );
    }

    #[test]
    fn one_supported_drop_uses_the_pending_delivery_slot() {
        let owner = FileActivationOwner::from_launch_args(args(&["app"]), Path::new("C:/games"));
        assert_eq!(owner.take_startup(), None);
        owner.mark_ready();

        assert_eq!(
            owner.admit_drop(vec![PathBuf::from("D:/棋谱 files/game one.gib")]),
            WarmAdmission::Available
        );
        assert_eq!(
            owner.take_warm(),
            Some(FileActivationDeliveryDto::Open {
                request_id: 1,
                path: "D:/棋谱 files/game one.gib".to_string(),
            })
        );
    }

    #[test]
    fn rejected_busy_drop_is_not_replayed_after_busy_clears() {
        let owner = FileActivationOwner::from_launch_args(args(&["app"]), Path::new("C:/games"));
        assert_eq!(owner.take_startup(), None);
        owner.mark_ready();
        owner.set_file_flow_busy(true);

        assert!(matches!(
            owner.admit_drop(vec![PathBuf::from("D:/games/rejected.sgf")]),
            WarmAdmission::Rejected(message) if message.contains("active")
        ));
        owner.set_file_flow_busy(false);
        assert_eq!(owner.take_warm(), None);
        assert_eq!(
            owner.admit_drop(vec![PathBuf::from("D:/games/later.sgf")]),
            WarmAdmission::Available
        );
        assert!(matches!(
            owner.take_warm(),
            Some(FileActivationDeliveryDto::Open { path, .. }) if path.ends_with("later.sgf")
        ));
    }

    #[test]
    fn unsupported_directory_and_multiple_drops_are_non_mutating_rejections() {
        let owner = FileActivationOwner::from_launch_args(args(&["app"]), Path::new("C:/games"));
        assert_eq!(owner.take_startup(), None);
        owner.mark_ready();

        assert!(matches!(
            owner.admit_drop(vec![PathBuf::from("D:/games/game.pdf")]),
            WarmAdmission::Rejected(message) if message.contains("unsupported")
        ));
        assert!(matches!(
            owner.admit_drop(vec![std::env::current_dir().unwrap()]),
            WarmAdmission::Rejected(message) if message.contains("directory")
        ));
        assert!(matches!(
            owner.admit_drop(vec![PathBuf::from("one.sgf"), PathBuf::from("two.gib")]),
            WarmAdmission::Rejected(message) if message.contains("multiple")
        ));
        assert_eq!(owner.take_warm(), None);
    }
}
