//! HTTP client for HeyoDrive API

use reqwest::{header, Client};
use serde::Deserialize;

/// Client errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] header::InvalidHeaderValue),
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
}

/// Storyboard summary from list endpoint
#[derive(Debug, Deserialize)]
pub struct StoryboardSummary {
    pub id: String,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<i64>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<i64>,
}

/// List storyboards response
#[derive(Debug, Deserialize)]
pub struct ListStoryboardsResponse {
    pub storyboards: Vec<StoryboardSummary>,
}

/// Latest file metadata response
#[derive(Debug, Deserialize)]
pub struct LatestSBFileResponse {
    pub sb_file_id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub size: Option<i64>,
}

/// API client for storyboard operations
pub struct HeyoClient {
    client: Client,
    base_url: String,
}

impl HeyoClient {
    /// Create a new client with the given base URL and auth token
    pub fn new(base_url: &str, token: &str) -> Result<Self, ClientError> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }

    /// GET /api/v1/storyboard - List all storyboards
    pub async fn list_storyboards(&self) -> Result<Vec<StoryboardSummary>, ClientError> {
        let url = format!("{}/api/v1/storyboard", self.base_url);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api { status, message });
        }

        // Response is a direct array, not wrapped
        let storyboards: Vec<StoryboardSummary> = resp.json().await?;
        Ok(storyboards)
    }

    /// GET /api/v1/storyboard/{id}/sb/latest - Get latest file metadata
    pub async fn get_latest_sb_file(&self, id: &str) -> Result<LatestSBFileResponse, ClientError> {
        let url = format!("{}/api/v1/storyboard/{}/sb/latest", self.base_url, id);
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api { status, message });
        }

        resp.json().await.map_err(Into::into)
    }

    /// GET /api/v1/drive/file/{fileId}/download - Download file bytes
    pub async fn download_file(&self, file_id: &str) -> Result<Vec<u8>, ClientError> {
        let url = format!(
            "{}/api/v1/drive/file/{}/download",
            self.base_url, file_id
        );
        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api { status, message });
        }

        Ok(resp.bytes().await?.to_vec())
    }

    /// PUT /api/v1/storyboard/{id}/sb - Upload storyboard file
    pub async fn upload_sb_file(
        &self,
        id: &str,
        data: Vec<u8>,
        filename: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/api/v1/storyboard/{}/sb", self.base_url, id);
        let resp = self
            .client
            .put(&url)
            .header(header::CONTENT_TYPE, "application/octet-stream")
            .header("X-Filename", filename)
            .body(data)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api { status, message });
        }

        Ok(())
    }
}
