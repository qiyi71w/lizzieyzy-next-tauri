use app_model::RenderedImageExportOptionsDto;
use std::path::{Path, PathBuf};
use windows::core::{w, HSTRING};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
    COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
use windows::Win32::UI::Shell::{
    FileSaveDialog, IFileSaveDialog, IShellItem, SHCreateItemFromParsingName, FOS_FORCEFILESYSTEM,
    SIGDN_FILESYSPATH,
};

pub fn pick(
    image: bool,
    directory: Option<&Path>,
    options: &RenderedImageExportOptionsDto,
) -> Result<Option<PathBuf>, String> {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED)
            .ok()
            .map_err(|error| error.to_string())?;
        let result = pick_com(image, directory, options);
        CoUninitialize();
        result.map_err(|error| format!("failed to select export target: {error}"))
    }
}

unsafe fn pick_com(
    image: bool,
    directory: Option<&Path>,
    options: &RenderedImageExportOptionsDto,
) -> windows::core::Result<Option<PathBuf>> {
    let dialog: IFileSaveDialog = CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)?;
    // Overwrite confirmation is performed only after the actual final target is normalized.
    dialog.SetOptions(FOS_FORCEFILESYSTEM)?;
    let formats = if image && options.png_only {
        vec![COMDLG_FILTERSPEC {
            pszName: w!("PNG"),
            pszSpec: w!("*.png"),
        }]
    } else if image {
        vec![
            COMDLG_FILTERSPEC {
                pszName: w!("PNG"),
                pszSpec: w!("*.png"),
            },
            COMDLG_FILTERSPEC {
                pszName: w!("JPG / JPEG"),
                pszSpec: w!("*.jpg;*.jpeg"),
            },
            COMDLG_FILTERSPEC {
                pszName: w!("GIF"),
                pszSpec: w!("*.gif"),
            },
            COMDLG_FILTERSPEC {
                pszName: w!("BMP"),
                pszSpec: w!("*.bmp"),
            },
        ]
    } else {
        vec![COMDLG_FILTERSPEC {
            pszName: w!("SGF"),
            pszSpec: w!("*.sgf"),
        }]
    };
    dialog.SetFileTypes(&formats)?;
    dialog.SetFileTypeIndex(1)?;
    if let Some(name) = &options.default_file_name {
        dialog.SetFileName(&HSTRING::from(name))?;
    }
    if let Some(directory) = directory {
        let item = folder_item(directory)?;
        dialog.SetFolder(&item)?;
    }
    if let Err(error) = dialog.Show(None) {
        if error.code().0 as u32 == 0x800704c7 {
            return Ok(None);
        }
        return Err(error);
    }
    let item = dialog.GetResult()?;
    let raw = item.GetDisplayName(SIGDN_FILESYSPATH)?;
    let path = raw.to_string();
    CoTaskMemFree(Some(raw.as_ptr().cast()));
    let mut path = PathBuf::from(path?);
    let index = dialog.GetFileTypeIndex()?;
    let extension = if image {
        match index {
            2 => "jpg",
            3 => "gif",
            4 => "bmp",
            _ => "png",
        }
    } else {
        "sgf"
    };
    let actual = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if actual != extension && !(extension == "jpg" && actual == "jpeg") {
        let mut name = path.into_os_string();
        name.push(format!(".{extension}"));
        path = PathBuf::from(name);
    }
    Ok(Some(path))
}

unsafe fn folder_item(directory: &Path) -> windows::core::Result<IShellItem> {
    // Canonical filesystem paths remain authoritative; the Shell needs a compatible parsing name.
    SHCreateItemFromParsingName(&HSTRING::from(dunce::simplified(directory).as_os_str()), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_recent_image_directory_is_usable_by_the_shell() {
        let directory = std::env::temp_dir().join(format!(
            "r11-export-folder-中文-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let canonical = directory.canonicalize().unwrap();
        assert!(canonical.as_os_str().as_encoded_bytes().starts_with(b"\\\\?\\"));
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
            {
                let dialog: IFileSaveDialog =
                    CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER).unwrap();
                dialog.SetOptions(FOS_FORCEFILESYSTEM).unwrap();
                for remembered in [&canonical, &directory] {
                    let item = folder_item(remembered).unwrap();
                    dialog.SetFolder(&item).unwrap();
                    let raw = item.GetDisplayName(SIGDN_FILESYSPATH).unwrap();
                    let display = raw.to_string();
                    CoTaskMemFree(Some(raw.as_ptr().cast()));
                    assert_eq!(PathBuf::from(display.unwrap()).canonicalize().unwrap(), canonical);
                }
            }
            CoUninitialize();
        }
        std::fs::remove_dir(directory).unwrap();
    }
}
