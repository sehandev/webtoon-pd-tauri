use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    domain::{BackendConfig, WorkspaceContext},
    error::{AppError, AppResult},
};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWysiwygDocumentRequest {
    pub backend: BackendConfig,
    pub workspace: WorkspaceContext,
    pub content_html: String,
    pub content_text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WysiwygDocument {
    pub id: String,
    pub content_html: String,
    pub content_text: String,
    pub updated_at: Option<String>,
}

pub async fn save_wysiwyg_document(request: SaveWysiwygDocumentRequest) -> AppResult<()> {
    validate_save_request(&request)?;

    let endpoint = format!(
        "{}/rest/v1/rpc/upsert_wysiwyg_document",
        request.backend.supabase_url.trim_end_matches('/')
    );
    let payload = json!({
        "organization_id": request.workspace.organization_id,
        "project_id": request.workspace.project_id,
        "episode_id": request.workspace.episode_id,
        "stage_id": request.workspace.stage_id,
        "content_html": request.content_html,
        "content_text": request.content_text,
        "updated_by": request.workspace.created_by,
    });

    let response = reqwest::Client::new()
        .post(endpoint)
        .header("apikey", &request.backend.supabase_anon_key)
        .header(
            AUTHORIZATION,
            format!("Bearer {}", request.backend.supabase_access_token),
        )
        .header(CONTENT_TYPE, "application/json")
        .json(&json!({ "payload": payload }))
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::RemoteStatus {
            context: "saving WYSIWYG document",
            status: status.as_u16(),
            body: response.text().await.unwrap_or_default(),
        });
    }

    Ok(())
}

pub async fn load_wysiwyg_document(
    backend: BackendConfig,
    stage_id: String,
) -> AppResult<Option<WysiwygDocument>> {
    if backend.supabase_url.trim().is_empty() {
        return Err(AppError::MissingField("supabaseUrl"));
    }
    if backend.supabase_anon_key.trim().is_empty() {
        return Err(AppError::MissingField("supabaseAnonKey"));
    }
    if backend.supabase_access_token.trim().is_empty() {
        return Err(AppError::MissingField("supabaseAccessToken"));
    }
    if stage_id.trim().is_empty() {
        return Err(AppError::MissingField("stageId"));
    }

    let endpoint = format!(
        "{}/rest/v1/wysiwyg_documents?stage_id=eq.{}&select=id,content_html,content_text,updated_at&limit=1",
        backend.supabase_url.trim_end_matches('/'),
        stage_id
    );
    let response = reqwest::Client::new()
        .get(endpoint)
        .header("apikey", &backend.supabase_anon_key)
        .header(
            AUTHORIZATION,
            format!("Bearer {}", backend.supabase_access_token),
        )
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::RemoteStatus {
            context: "loading WYSIWYG document",
            status: status.as_u16(),
            body: response.text().await.unwrap_or_default(),
        });
    }

    let mut rows: Vec<WysiwygDocument> = response.json().await?;
    Ok(rows.pop())
}

fn validate_save_request(request: &SaveWysiwygDocumentRequest) -> AppResult<()> {
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
