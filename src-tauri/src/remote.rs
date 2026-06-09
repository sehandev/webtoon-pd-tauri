use std::{collections::HashMap, path::Path};

use reqwest::{
    header::{HeaderName, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE},
    Body, Client,
};
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;

use crate::{
    domain::BackendConfig,
    error::{AppError, AppResult},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignPutObject {
    pub key: String,
    pub content_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignBatchResponse {
    pub urls: Vec<PresignedPutUrl>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignedPutUrl {
    pub key: String,
    pub url: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

pub async fn request_presigned_put_urls(
    config: &BackendConfig,
    objects: Vec<PresignPutObject>,
) -> AppResult<HashMap<String, PresignedPutUrl>> {
    let client = Client::new();
    let endpoint = format!(
        "{}/api/r2/presigned-put-urls",
        config.worker_base_url.trim_end_matches('/')
    );
    let response = client
        .post(endpoint)
        .bearer_auth(&config.supabase_access_token)
        .json(&serde_json::json!({ "objects": objects }))
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::RemoteStatus {
            context: "requesting R2 presigned URLs",
            status: status.as_u16(),
            body: response.text().await.unwrap_or_default(),
        });
    }

    let payload: PresignBatchResponse = response.json().await?;
    Ok(payload
        .urls
        .into_iter()
        .map(|url| (url.key.clone(), url))
        .collect())
}

pub async fn upload_file_to_presigned_url(
    presigned: &PresignedPutUrl,
    file_path: &Path,
    content_type: &str,
) -> AppResult<()> {
    let client = Client::new();
    let metadata = tokio::fs::metadata(file_path).await?;
    let file = tokio::fs::File::open(file_path).await?;
    let stream = ReaderStream::new(file);

    let mut request = client
        .put(&presigned.url)
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_LENGTH, metadata.len())
        .body(Body::wrap_stream(stream));

    for (name, value) in parse_presigned_headers(&presigned.headers)? {
        request = request.header(name, value);
    }

    let response = request.send().await?;
    let status = response.status();
    if !status.is_success() {
        return Err(AppError::RemoteStatus {
            context: "uploading to R2",
            status: status.as_u16(),
            body: response.text().await.unwrap_or_default(),
        });
    }

    Ok(())
}

pub async fn finalize_asset_processing(
    config: &BackendConfig,
    payload: serde_json::Value,
) -> AppResult<serde_json::Value> {
    let client = Client::new();
    let endpoint = format!(
        "{}/rest/v1/rpc/finalize_asset_processing",
        config.supabase_url.trim_end_matches('/')
    );
    let response = client
        .post(endpoint)
        .header("apikey", &config.supabase_anon_key)
        .header(
            AUTHORIZATION,
            format!("Bearer {}", config.supabase_access_token),
        )
        .json(&serde_json::json!({ "payload": payload }))
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        return Err(AppError::RemoteStatus {
            context: "finalizing Supabase asset processing",
            status: status.as_u16(),
            body: response.text().await.unwrap_or_default(),
        });
    }

    Ok(response
        .json()
        .await
        .unwrap_or_else(|_| serde_json::json!({})))
}

fn parse_presigned_headers(
    headers: &HashMap<String, String>,
) -> AppResult<Vec<(HeaderName, HeaderValue)>> {
    let mut parsed = Vec::with_capacity(headers.len());
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|_| AppError::InvalidRemoteHeader(name.clone()))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|_| AppError::InvalidRemoteHeader(name.clone()))?;
        parsed.push((header_name, header_value));
    }
    Ok(parsed)
}
