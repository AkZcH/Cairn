//! Talks to the Cairn backend over HTTP, authenticated with a per-user API
//! key, rather than connecting to Postgres directly. In a multi-tenant
//! world, a local CLI can't hold a shared database password, an API key
//! scoped to one account is the correct boundary.

use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct ChunkPayload {
    index: i32,
    content: String,
    embedding: Vec<f32>,
}

#[derive(Serialize)]
struct UploadPayload {
    source: String,
    title: Option<String>,
    source_path: String,
    content_hash: String,
    chunks: Vec<ChunkPayload>,
}

#[derive(serde::Deserialize)]
pub struct UploadResponse {
    pub document_id: Uuid,
    pub status: String, // "inserted" | "updated" | "unchanged"
}

pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl ApiClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self { client: reqwest::Client::new(), base_url, api_key }
    }

    pub async fn upload_document(
        &self,
        source: &str,
        title: Option<&str>,
        source_path: &str,
        content_hash: &str,
        chunks: Vec<(String, Vec<f32>)>, // (content, embedding) pairs
    ) -> anyhow::Result<UploadResponse> {
        let payload = UploadPayload {
            source: source.to_string(),
            title: title.map(String::from),
            source_path: source_path.to_string(),
            content_hash: content_hash.to_string(),
            chunks: chunks
                .into_iter()
                .enumerate()
                .map(|(i, (content, embedding))| ChunkPayload {
                    index: i as i32,
                    content,
                    embedding,
                })
                .collect(),
        };

        let response = self
            .client
            .post(format!("{}/documents/upload", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("upload failed ({status}): {body}");
        }

        Ok(response.json().await?)
    }
}