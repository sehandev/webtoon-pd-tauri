use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const MAX_TILE_WIDTH: u32 = 2048;
pub const MAX_TILE_HEIGHT: u32 = 2048;

const SUPPORTED_EXTENSIONS: [&str; 8] =
    ["clip", "psd", "psb", "jpg", "jpeg", "png", "webp", "avif"];

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceFormat {
    Clip,
    Psd,
    Psb,
    Jpg,
    Png,
    Webp,
    Avif,
}

impl SourceFormat {
    pub fn from_path(path: &Path) -> AppResult<Self> {
        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .ok_or_else(|| AppError::UnsupportedFormat("missing extension".to_string()))?;

        match extension.as_str() {
            "clip" => Ok(Self::Clip),
            "psd" => Ok(Self::Psd),
            "psb" => Ok(Self::Psb),
            "jpg" | "jpeg" => Ok(Self::Jpg),
            "png" => Ok(Self::Png),
            "webp" => Ok(Self::Webp),
            "avif" => Ok(Self::Avif),
            other => Err(AppError::UnsupportedFormat(other.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clip => "clip",
            Self::Psd => "psd",
            Self::Psb => "psb",
            Self::Jpg => "jpg",
            Self::Png => "png",
            Self::Webp => "webp",
            Self::Avif => "avif",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            Self::Clip => "application/octet-stream",
            Self::Psd => "image/vnd.adobe.photoshop",
            Self::Psb => "image/vnd.adobe.photoshop",
            Self::Jpg => "image/jpeg",
            Self::Png => "image/png",
            Self::Webp => "image/webp",
            Self::Avif => "image/avif",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFileSummary {
    pub path: String,
    pub file_name: String,
    pub format: SourceFormat,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendConfig {
    pub worker_base_url: String,
    pub supabase_url: String,
    pub supabase_anon_key: String,
    pub supabase_access_token: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceContext {
    pub organization_id: String,
    pub project_id: String,
    pub episode_id: String,
    pub stage_id: String,
    pub created_by: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolConfig {
    pub renderer_bin: Option<String>,
    pub avifenc_bin: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSourceAssetRequest {
    pub source_path: String,
    pub backend: BackendConfig,
    pub workspace: WorkspaceContext,
    #[serde(default)]
    pub tools: ToolConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderManifest {
    pub source_width: u32,
    pub source_height: u32,
    pub rendered_width: u32,
    pub rendered_height: u32,
    pub tiles: Vec<RenderedTile>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedTile {
    pub index: u32,
    pub path: PathBuf,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReviewTile {
    pub index: u32,
    pub key: String,
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessSourceAssetResponse {
    pub asset_id: String,
    pub review_asset_id: String,
    pub asset_processing_job_id: String,
    pub original_key: String,
    pub tile_count: usize,
    pub rendered_width: u32,
    pub rendered_height: u32,
    pub tiles: Vec<ReviewTile>,
}

pub fn supported_extensions() -> &'static [&'static str] {
    &SUPPORTED_EXTENSIONS
}
