use crate::{continuous_analysis::PreferencesState, current_game_state::CurrentGameState};
use app_model::{ExportConfirmationDto, NodePath, RenderedImageExportOptionsDto};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

#[cfg(windows)]
mod windows_picker;

fn sgf_target(path: &Path) -> PathBuf {
    if path
        .extension()
        .is_some_and(|value| value.eq_ignore_ascii_case("sgf"))
    {
        path.to_path_buf()
    } else {
        let mut name = path.as_os_str().to_os_string();
        name.push(".sgf");
        PathBuf::from(name)
    }
}

fn png_target(path: &Path) -> PathBuf {
    if path
        .extension()
        .is_some_and(|value| value.eq_ignore_ascii_case("png"))
    {
        return path.to_path_buf();
    }
    let mut name = path.as_os_str().to_os_string();
    name.push(".png");
    PathBuf::from(name)
}

fn confirm_target(app: &AppHandle, target: &Path, confirmation: &ExportConfirmationDto) -> bool {
    !target.exists()
        || app
            .dialog()
            .message(format!("{}\n{}", confirmation.message, target.display()))
            .title(&confirmation.title)
            .buttons(MessageDialogButtons::OkCancel)
            .blocking_show()
}

fn pick_target(
    app: &AppHandle,
    image: bool,
    directory: Option<&Path>,
    options: &RenderedImageExportOptionsDto,
) -> Result<Option<PathBuf>, String> {
    #[cfg(windows)]
    {
        let _ = app;
        windows_picker::pick(image, directory, options)
    }
    #[cfg(not(windows))]
    {
        let mut dialog = app.dialog().file();
        if image {
            dialog = dialog.add_filter("PNG", &["png"]);
            if !options.png_only {
                dialog = dialog
                    .add_filter("JPG / JPEG", &["jpg", "jpeg"])
                    .add_filter("GIF", &["gif"])
                    .add_filter("BMP", &["bmp"]);
            }
        } else {
            dialog = dialog.add_filter("SGF", &["sgf"]);
        }
        if let Some(directory) = directory {
            dialog = dialog.set_directory(directory);
        }
        if let Some(name) = &options.default_file_name {
            dialog = dialog.set_file_name(name);
        }
        dialog
            .blocking_save_file()
            .map(|picked| picked.into_path().map_err(|error| error.to_string()))
            .transpose()
    }
}

#[tauri::command]
pub async fn export_selected_line(
    app: AppHandle,
    state: State<'_, CurrentGameState>,
    generation: u64,
    selected_path: NodePath,
    leaf_path: NodePath,
    confirmation: ExportConfirmationDto,
) -> Result<Option<String>, String> {
    let bytes = state.capture_selected_line(generation, &selected_path, &leaf_path)?;
    tauri::async_runtime::spawn_blocking(move || {
        let Some(picked) = pick_target(&app, false, None, &RenderedImageExportOptionsDto::default())? else {
            return Ok(None);
        };
        let target = sgf_target(&picked);
        if !confirm_target(&app, &target, &confirmation) {
            return Ok(None);
        }
        sgf::write_file_atomic(&target, bytes.as_bytes())?;
        Ok(Some(target.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn export_rendered_image(
    app: AppHandle,
    preferences: State<'_, PreferencesState>,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    confirmation: ExportConfirmationDto,
    options: Option<RenderedImageExportOptionsDto>,
) -> Result<Option<String>, String> {
    let directory = preferences.image_export_directory()?;
    let preferences_path = crate::app_preferences_path(&app)?;
    let options = options.unwrap_or_default();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(picked) = pick_target(&app, true, directory.as_deref(), &options)? else {
            return Ok(None);
        };
        let picked = if options.png_only {
            png_target(&picked)
        } else {
            picked
        };
        let (target, format) = image_export::normalize_target(&picked)?;
        if !confirm_target(&app, &target, &confirmation) {
            return Ok(None);
        }
        image_export::write_rgba_atomic(&target, width, height, rgba, format)?;
        app.state::<PreferencesState>()
            .record_image_export(&preferences_path, &target)
            .map_err(|error| {
                format!(
                    "Image written to {}, but failed to persist its directory: {error}",
                    target.display()
                )
            })?;
        Ok(Some(target.to_string_lossy().into_owned()))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chart_configuration_defaults_and_png_final_target() {
        let defaults: RenderedImageExportOptionsDto = serde_json::from_str("{}").unwrap();
        assert!(defaults.default_file_name.is_none());
        assert!(!defaults.png_only);
        let chart: RenderedImageExportOptionsDto =
            serde_json::from_str(r#"{"defaultFileName":"study-winrate-m3.png","pngOnly":true}"#).unwrap();
        assert_eq!(chart.default_file_name.as_deref(), Some("study-winrate-m3.png"));
        assert!(chart.png_only);
        assert_eq!(png_target(Path::new("chart.PNG")), Path::new("chart.PNG"));
        assert_eq!(png_target(Path::new("chart.jpg")), Path::new("chart.jpg.png"));
    }
    #[test]
    fn sgf_final_target_retains_uppercase_and_normalizes_before_confirmation() {
        assert_eq!(sgf_target(Path::new("game.SGF")), Path::new("game.SGF"));
        assert_eq!(
            sgf_target(Path::new("protected-sgf-中文")),
            Path::new("protected-sgf-中文.sgf")
        );
    }
}
