use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("unsupported file format: {0}")]
    UnsupportedFormat(String),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
    #[error("missing presigned URL for R2 object key: {0}")]
    MissingPresignedUrl(String),
    #[error("tool was not found: {0}")]
    ToolNotFound(String),
    #[error("tool failed: {tool}\nstatus: {status}\nstderr: {stderr}")]
    ToolFailed {
        tool: String,
        status: String,
        stderr: String,
    },
    #[error("remote request failed during {context}: HTTP {status}\n{body}")]
    RemoteStatus {
        context: &'static str,
        status: u16,
        body: String,
    },
    #[error("invalid remote header: {0}")]
    InvalidRemoteHeader(String),
    #[error("path is not a local file path: {0}")]
    NonLocalPath(String),
    #[error("path has no file name: {0}")]
    MissingFileName(PathBuf),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("dialog error: {0}")]
    Dialog(#[from] tauri_plugin_dialog::Error),
}

pub type AppResult<T> = Result<T, AppError>;
