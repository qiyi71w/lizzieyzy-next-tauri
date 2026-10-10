use app_model::{DiagnosticExportStatusDto, DiagnosticFolderOutcomeDto, EngineDiagnosticSnapshotDto};
use engine_manager::diagnostic_export::DiagnosticExport;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Default)]
pub struct FolderOpener(pub Arc<AtomicBool>);

#[tauri::command]
pub fn estimate_diagnostic_export(
    owner: State<'_, DiagnosticExport>,
    snapshot: EngineDiagnosticSnapshotDto,
) -> Result<u64, String> {
    owner.estimate(snapshot)
}
#[tauri::command]
pub fn diagnostic_export_status(owner: State<'_, DiagnosticExport>) -> DiagnosticExportStatusDto {
    owner.status()
}
#[tauri::command]
pub fn start_diagnostic_export(owner: State<'_, DiagnosticExport>, generation: u64) -> Result<(), String> {
    owner.export(generation)
}
#[tauri::command]
pub fn cancel_diagnostic_export(owner: State<'_, DiagnosticExport>, generation: u64) -> Result<(), String> {
    owner.cancel(generation)
}

#[tauri::command]
pub async fn open_diagnostic_export_folder(
    app: AppHandle,
    owner: State<'_, DiagnosticExport>,
    opener: State<'_, FolderOpener>,
    generation: u64,
) -> Result<DiagnosticFolderOutcomeDto, String> {
    let Ok(directory) = owner.completed_directory(generation) else {
        return Ok(DiagnosticFolderOutcomeDto::Failed);
    };
    if opener.0.swap(true, Ordering::AcqRel) {
        return Ok(DiagnosticFolderOutcomeDto::Failed);
    }
    let busy = opener.0.clone();
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let result = app.opener().open_path(directory.to_string_lossy(), None::<&str>);
        busy.store(false, Ordering::Release);
        let _ = send.send(result.is_ok());
    });
    Ok(
        tauri::async_runtime::spawn_blocking(move || match receive.recv_timeout(Duration::from_secs(2)) {
            Ok(true) => DiagnosticFolderOutcomeDto::Opened,
            Ok(false) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                DiagnosticFolderOutcomeDto::Failed
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => DiagnosticFolderOutcomeDto::TimedOut,
        })
        .await
        .unwrap_or(DiagnosticFolderOutcomeDto::Failed),
    )
}
