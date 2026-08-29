use save_as_dialog::{classify_save_as_session, SaveAsDialogOutcome};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use windows::core::{implement, w, Error, Ref, Result as WinResult, HSTRING};
use windows::Win32::Foundation::S_OK;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
use windows::Win32::UI::Shell::{
    FileSaveDialog, IFileDialog, IFileDialogEvents, IFileSaveDialog, IShellItem, FDEOR_DEFAULT,
    FDESVR_DEFAULT, FDE_OVERWRITE_RESPONSE, FDE_SHAREVIOLATION_RESPONSE, FOS_FORCEFILESYSTEM,
    FOS_OVERWRITEPROMPT, SIGDN_FILESYSPATH,
};

pub fn pick_save_as_outcome(default_file_name: &str) -> Result<SaveAsDialogOutcome, String> {
    let _com = ComGuard::new()?;
    unsafe { pick_save_as_outcome_com(default_file_name) }.map_err(com_error)
}

unsafe fn pick_save_as_outcome_com(default_file_name: &str) -> WinResult<SaveAsDialogOutcome> {
    let dialog: IFileSaveDialog = CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)?;
    dialog.SetOptions(FOS_FORCEFILESYSTEM | FOS_OVERWRITEPROMPT)?;
    let filters = [COMDLG_FILTERSPEC {
        pszName: w!("SGF files"),
        pszSpec: w!("*.sgf;*.txt"),
    }];
    dialog.SetFileTypes(&filters)?;
    dialog.SetDefaultExtension(w!("sgf"))?;
    dialog.SetFileName(&HSTRING::from(default_file_name))?;

    let requested_unwritable = Arc::new(Mutex::new(None));
    let events: IFileDialogEvents = SaveAsEvents {
        requested_unwritable: requested_unwritable.clone(),
    }
    .into();
    let cookie = dialog.Advise(&events)?;
    let shown = dialog.Show(None);
    let _ = dialog.Unadvise(cookie);
    let requested = requested_unwritable
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .clone();
    if shown.is_err() {
        return Ok(classify_save_as_session(requested, None));
    }

    let returned = dialog.GetResult().ok().and_then(|item| shell_item_path(&item));
    Ok(classify_save_as_session(requested, returned))
}

#[implement(IFileDialogEvents)]
struct SaveAsEvents {
    requested_unwritable: Arc<Mutex<Option<String>>>,
}

impl SaveAsEvents {
    fn capture(&self, dialog: &IFileDialog) {
        let Some(path) = typed_path(dialog) else {
            return;
        };
        if parent_writable(Path::new(&path)) {
            return;
        }
        if let Ok(mut slot) = self.requested_unwritable.lock() {
            *slot = Some(path);
        }
    }
}

impl IFileDialogEvents_Impl for SaveAsEvents_Impl {
    fn OnFileOk(&self, pfd: Ref<'_, IFileDialog>) -> WinResult<()> {
        if let Ok(dialog) = pfd.ok() {
            self.capture(dialog);
        }
        Ok(())
    }

    fn OnFolderChanging(&self, pfd: Ref<'_, IFileDialog>, _folder: Ref<'_, IShellItem>) -> WinResult<()> {
        if let Ok(dialog) = pfd.ok() {
            self.capture(dialog);
        }
        Ok(())
    }

    fn OnFolderChange(&self, pfd: Ref<'_, IFileDialog>) -> WinResult<()> {
        if let Ok(dialog) = pfd.ok() {
            self.capture(dialog);
        }
        Ok(())
    }

    fn OnSelectionChange(&self, pfd: Ref<'_, IFileDialog>) -> WinResult<()> {
        if let Ok(dialog) = pfd.ok() {
            self.capture(dialog);
        }
        Ok(())
    }

    fn OnShareViolation(
        &self,
        _pfd: Ref<'_, IFileDialog>,
        _psi: Ref<'_, IShellItem>,
    ) -> WinResult<FDE_SHAREVIOLATION_RESPONSE> {
        Ok(FDESVR_DEFAULT)
    }

    fn OnTypeChange(&self, _pfd: Ref<'_, IFileDialog>) -> WinResult<()> {
        Ok(())
    }

    fn OnOverwrite(
        &self,
        _pfd: Ref<'_, IFileDialog>,
        _psi: Ref<'_, IShellItem>,
    ) -> WinResult<FDE_OVERWRITE_RESPONSE> {
        Ok(FDEOR_DEFAULT)
    }
}

fn typed_path(dialog: &IFileDialog) -> Option<String> {
    let name = dialog_file_name(dialog)?;
    if Path::new(&name).is_absolute() {
        return Some(name);
    }
    let folder = dialog_folder_path(dialog)?;
    Some(Path::new(&folder).join(name).to_string_lossy().into_owned())
}

fn dialog_file_name(dialog: &IFileDialog) -> Option<String> {
    unsafe {
        let pwstr = dialog.GetFileName().ok()?;
        let name = pwstr.to_string().ok();
        CoTaskMemFree(Some(pwstr.as_ptr().cast()));
        name
    }
}

fn dialog_folder_path(dialog: &IFileDialog) -> Option<String> {
    unsafe { dialog.GetFolder().ok().and_then(|item| shell_item_path(&item)) }
}

fn shell_item_path(item: &IShellItem) -> Option<String> {
    unsafe {
        let pwstr = item.GetDisplayName(SIGDN_FILESYSPATH).ok()?;
        let path = pwstr.to_string().ok();
        CoTaskMemFree(Some(pwstr.as_ptr().cast()));
        path
    }
}

fn parent_writable(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return false;
    };
    if parent.as_os_str().is_empty() {
        return false;
    }
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let probe = parent.join(format!(".lizzieyzy-write-probe-{}-{unique}", std::process::id()));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(file) => {
            drop(file);
            let _ = std::fs::remove_file(probe);
            true
        }
        Err(_) => false,
    }
}

fn com_error(error: Error) -> String {
    format!("failed to write SGF file: {error}")
}

struct ComGuard {
    initialized: bool,
}

impl ComGuard {
    fn new() -> Result<Self, String> {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if hr.is_err() {
            return Err(com_error(Error::from_hresult(hr)));
        }
        Ok(Self {
            initialized: hr == S_OK,
        })
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}
