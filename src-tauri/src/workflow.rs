use std::path::PathBuf;

use chrono::Utc;
use serde_json::json;
use tauri::AppHandle;
use tempfile::tempdir;
use uuid::Uuid;

use crate::{
    domain::{
        ProcessSourceAssetRequest, ProcessSourceAssetResponse, ReviewTile, SourceFormat,
        MAX_TILE_HEIGHT, MAX_TILE_WIDTH,
    },
    error::{AppError, AppResult},
    files::{copy_source_to_workdir, summarize_source},
    remote::{
        finalize_asset_processing, request_presigned_put_urls, upload_file_to_presigned_url,
        PresignPutObject, PresignedPutUrl,
    },
    sidecars::{encode_avif_tiles, render_source},
};

pub async fn process_source_asset(
    app: AppHandle,
    request: ProcessSourceAssetRequest,
) -> AppResult<ProcessSourceAssetResponse> {
    validate_request(&request)?;

    let source_path = PathBuf::from(&request.source_path);
    let source_summary = summarize_source(&source_path)?;
    let source_format = source_summary.format;
    let asset_id = Uuid::new_v4().to_string();
    let review_asset_id = Uuid::new_v4().to_string();
    let asset_processing_job_id = Uuid::new_v4().to_string();

    let workdir = tempdir()?;
    let source_copy = copy_source_to_workdir(&source_path, workdir.path())?;
    let render_dir = workdir.path().join("rendered_png_tiles");
    let avif_dir = workdir.path().join("avif_tiles");

    let manifest = render_source(&app, &request.tools, &source_copy, &render_dir)?;
    let avif_tile_paths = encode_avif_tiles(&app, &request.tools, &manifest.tiles, &avif_dir)?;

    let original_key = original_object_key(
        &request.workspace.organization_id,
        &request.workspace.project_id,
        &request.workspace.episode_id,
        &request.workspace.stage_id,
        &asset_id,
        &source_summary.file_name,
    );
    let tiles = build_review_tiles(
        &request.workspace.organization_id,
        &request.workspace.project_id,
        &request.workspace.episode_id,
        &request.workspace.stage_id,
        &review_asset_id,
        &manifest.tiles,
        &avif_tile_paths,
    )?;

    let presigned_urls = request_presigned_put_urls(
        &request.backend,
        build_presign_objects(
            &original_key,
            &source_copy,
            source_format,
            &tiles,
            &avif_tile_paths,
        )?,
    )
    .await?;

    upload_original_and_tiles(
        &presigned_urls,
        &original_key,
        &source_copy,
        source_format,
        &tiles,
        &avif_tile_paths,
    )
    .await?;

    let finalize_payload = build_finalize_payload(
        &request,
        &source_summary,
        &asset_id,
        &review_asset_id,
        &asset_processing_job_id,
        &original_key,
        &manifest,
        &tiles,
    );
    finalize_asset_processing(&request.backend, finalize_payload).await?;

    Ok(ProcessSourceAssetResponse {
        asset_id,
        review_asset_id,
        asset_processing_job_id,
        original_key,
        tile_count: tiles.len(),
        rendered_width: manifest.rendered_width,
        rendered_height: manifest.rendered_height,
        tiles,
    })
}

fn validate_request(request: &ProcessSourceAssetRequest) -> AppResult<()> {
    if request.source_path.trim().is_empty() {
        return Err(AppError::MissingField("sourcePath"));
    }
    if request.backend.worker_base_url.trim().is_empty() {
        return Err(AppError::MissingField("workerBaseUrl"));
    }
    if request.backend.supabase_url.trim().is_empty() {
        return Err(AppError::MissingField("supabaseUrl"));
    }
    if request.backend.supabase_anon_key.trim().is_empty() {
        return Err(AppError::MissingField("supabaseAnonKey"));
    }
    if request.backend.supabase_access_token.trim().is_empty() {
        return Err(AppError::MissingField("supabaseAccessToken"));
    }
    if request.workspace.organization_id.trim().is_empty() {
        return Err(AppError::MissingField("organizationId"));
    }
    if request.workspace.project_id.trim().is_empty() {
        return Err(AppError::MissingField("projectId"));
    }
    if request.workspace.episode_id.trim().is_empty() {
        return Err(AppError::MissingField("episodeId"));
    }
    if request.workspace.stage_id.trim().is_empty() {
        return Err(AppError::MissingField("stageId"));
    }
    if request.workspace.created_by.trim().is_empty() {
        return Err(AppError::MissingField("createdBy"));
    }
    Ok(())
}

fn build_presign_objects(
    original_key: &str,
    source_copy: &PathBuf,
    source_format: SourceFormat,
    tiles: &[ReviewTile],
    avif_tile_paths: &[PathBuf],
) -> AppResult<Vec<PresignPutObject>> {
    let mut objects = vec![PresignPutObject {
        key: original_key.to_string(),
        content_type: source_format.content_type().to_string(),
        size_bytes: source_copy.metadata()?.len(),
    }];

    for (tile, path) in tiles.iter().zip(avif_tile_paths) {
        objects.push(PresignPutObject {
            key: tile.key.clone(),
            content_type: "image/avif".to_string(),
            size_bytes: path.metadata()?.len(),
        });
    }

    Ok(objects)
}

async fn upload_original_and_tiles(
    urls: &std::collections::HashMap<String, PresignedPutUrl>,
    original_key: &str,
    source_copy: &PathBuf,
    source_format: SourceFormat,
    tiles: &[ReviewTile],
    avif_tile_paths: &[PathBuf],
) -> AppResult<()> {
    let original_url = urls
        .get(original_key)
        .ok_or_else(|| AppError::MissingPresignedUrl(original_key.to_string()))?;
    upload_file_to_presigned_url(original_url, source_copy, source_format.content_type()).await?;

    for (tile, path) in tiles.iter().zip(avif_tile_paths) {
        let tile_url = urls
            .get(&tile.key)
            .ok_or_else(|| AppError::MissingPresignedUrl(tile.key.clone()))?;
        upload_file_to_presigned_url(tile_url, path, "image/avif").await?;
    }

    Ok(())
}

fn build_review_tiles(
    organization_id: &str,
    project_id: &str,
    episode_id: &str,
    stage_id: &str,
    review_asset_id: &str,
    rendered_tiles: &[crate::domain::RenderedTile],
    avif_tile_paths: &[PathBuf],
) -> AppResult<Vec<ReviewTile>> {
    rendered_tiles
        .iter()
        .zip(avif_tile_paths)
        .map(|(tile, path)| {
            Ok(ReviewTile {
                index: tile.index,
                key: tile_object_key(
                    organization_id,
                    project_id,
                    episode_id,
                    stage_id,
                    review_asset_id,
                    tile.index,
                ),
                width: tile.width,
                height: tile.height,
                x: tile.x,
                y: tile.y,
                size_bytes: path.metadata()?.len(),
            })
        })
        .collect()
}

fn build_finalize_payload(
    request: &ProcessSourceAssetRequest,
    source: &crate::domain::SourceFileSummary,
    asset_id: &str,
    review_asset_id: &str,
    asset_processing_job_id: &str,
    original_key: &str,
    manifest: &crate::domain::RenderManifest,
    tiles: &[ReviewTile],
) -> serde_json::Value {
    let now = Utc::now();
    json!({
        "asset": {
            "id": asset_id,
            "organization_id": request.workspace.organization_id,
            "project_id": request.workspace.project_id,
            "episode_id": request.workspace.episode_id,
            "stage_id": request.workspace.stage_id,
            "object_key": original_key,
            "file_name": source.file_name,
            "format": source.format.as_str(),
            "content_type": source.format.content_type(),
            "size_bytes": source.size_bytes,
            "sha256": source.sha256,
            "created_by": request.workspace.created_by,
            "created_at": now,
        },
        "review_asset": {
            "id": review_asset_id,
            "organization_id": request.workspace.organization_id,
            "project_id": request.workspace.project_id,
            "episode_id": request.workspace.episode_id,
            "stage_id": request.workspace.stage_id,
            "asset_id": asset_id,
            "format": "avif",
            "max_tile_width": MAX_TILE_WIDTH,
            "tile_height": MAX_TILE_HEIGHT,
            "source_width": manifest.source_width,
            "source_height": manifest.source_height,
            "rendered_width": manifest.rendered_width,
            "rendered_height": manifest.rendered_height,
            "tiles": tiles,
            "created_by": request.workspace.created_by,
            "created_at": now,
        },
        "asset_processing_job": {
            "id": asset_processing_job_id,
            "organization_id": request.workspace.organization_id,
            "project_id": request.workspace.project_id,
            "episode_id": request.workspace.episode_id,
            "stage_id": request.workspace.stage_id,
            "asset_id": asset_id,
            "review_asset_id": review_asset_id,
            "input_format": source.format.as_str(),
            "output_format": "avif",
            "tile_count": tiles.len(),
            "original_width": manifest.source_width,
            "original_height": manifest.source_height,
            "created_by": request.workspace.created_by,
            "completed_at": now,
        },
    })
}

fn original_object_key(
    organization_id: &str,
    project_id: &str,
    episode_id: &str,
    stage_id: &str,
    asset_id: &str,
    file_name: &str,
) -> String {
    format!(
        "org/{}/projects/{}/episodes/{}/stages/{}/assets/{}/original/{}",
        sanitize_segment(organization_id),
        sanitize_segment(project_id),
        sanitize_segment(episode_id),
        sanitize_segment(stage_id),
        sanitize_segment(asset_id),
        sanitize_segment(file_name)
    )
}

fn tile_object_key(
    organization_id: &str,
    project_id: &str,
    episode_id: &str,
    stage_id: &str,
    review_asset_id: &str,
    index: u32,
) -> String {
    format!(
        "org/{}/projects/{}/episodes/{}/stages/{}/review/{}/tiles/tile_{:04}.avif",
        sanitize_segment(organization_id),
        sanitize_segment(project_id),
        sanitize_segment(episode_id),
        sanitize_segment(stage_id),
        sanitize_segment(review_asset_id),
        index
    )
}

fn sanitize_segment(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '=') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
