use std::path::PathBuf;

use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::{
    domain::{supported_extensions, BackendConfig, ProcessSourceAssetRequest, SourceFileSummary},
    error::{AppError, AppResult},
    files::summarize_source,
    workflow,
    wysiwyg::{self, SaveWysiwygDocumentRequest, WysiwygDocument},
};

#[tauri::command]
pub fn select_source_file(app: AppHandle) -> Result<Option<SourceFileSummary>, String> {
    select_source_file_inner(app).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn process_source_asset(
    app: AppHandle,
    request: ProcessSourceAssetRequest,
) -> Result<crate::domain::ProcessSourceAssetResponse, String> {
    workflow::process_source_asset(app, request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn save_wysiwyg_document(request: SaveWysiwygDocumentRequest) -> Result<(), String> {
    wysiwyg::save_wysiwyg_document(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn load_wysiwyg_document(
    backend: BackendConfig,
    stage_id: String,
) -> Result<Option<WysiwygDocument>, String> {
    wysiwyg::load_wysiwyg_document(backend, stage_id)
        .await
        .map_err(|error| error.to_string())
}

fn select_source_file_inner(app: AppHandle) -> AppResult<Option<SourceFileSummary>> {
    let selected = app
        .dialog()
        .file()
        .add_filter("Supported artwork files", supported_extensions())
        .blocking_pick_file();

    let Some(file_path) = selected else {
        return Ok(None);
    };

    let selected_display = file_path.to_string();
    let path: PathBuf = file_path
        .into_path()
        .map_err(|_| AppError::NonLocalPath(selected_display))?;
    summarize_source(&path).map(Some)
}
