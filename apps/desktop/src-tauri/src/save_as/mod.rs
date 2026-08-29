use crate::current_game_state::CurrentGameState;
use app_model::{CurrentGameResultDto, NodePath};
use save_as_dialog::{classify_save_as_session, persist_save_as, SaveAsDialogOutcome};
use tauri::{AppHandle, Runtime};

#[cfg(windows)]
mod windows_picker;

pub fn persist_current_game_save_as(
    state: &CurrentGameState,
    outcome: SaveAsDialogOutcome,
    selected_path: NodePath,
) -> Result<Option<CurrentGameResultDto>, String> {
    persist_save_as(outcome, |path| state.save_to_path(path, selected_path))
}

pub fn pick_save_as_outcome<R: Runtime>(
    app: &AppHandle<R>,
    default_file_name: &str,
) -> Result<SaveAsDialogOutcome, String> {
    #[cfg(windows)]
    {
        let _ = app;
        windows_picker::pick_save_as_outcome(default_file_name)
    }
    #[cfg(not(windows))]
    {
        pick_save_as_outcome_dialog(app, default_file_name)
    }
}

#[cfg(not(windows))]
fn pick_save_as_outcome_dialog<R: Runtime>(
    app: &AppHandle<R>,
    default_file_name: &str,
) -> Result<SaveAsDialogOutcome, String> {
    use tauri_plugin_dialog::DialogExt;

    let returned = app
        .dialog()
        .file()
        .add_filter("SGF files", &["sgf", "txt"])
        .set_file_name(default_file_name)
        .blocking_save_file()
        .map(|picked| picked.into_path().map_err(|error| error.to_string()))
        .transpose()?
        .map(|path| path.to_string_lossy().into_owned());
    Ok(classify_save_as_session(None, returned))
}
