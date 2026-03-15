use futures_util::{Stream, StreamExt};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum FoundryError {
    Http(reqwest::Error),
    Json(serde_json::Error),
    Stream(String),
    Api { status: u16, message: String },
}

impl std::fmt::Display for FoundryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Stream(e) => write!(f, "Stream error: {e}"),
            Self::Api { status, message } => write!(f, "API {status}: {message}"),
        }
    }
}

impl std::error::Error for FoundryError {}

impl From<reqwest::Error> for FoundryError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

impl From<serde_json::Error> for FoundryError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, FoundryError>;

// ---------------------------------------------------------------------------
// API version constants
// ---------------------------------------------------------------------------

const API_AGENTS: &str = "2024-05-01-preview";
const API_DEPLOYMENTS: &str = "2024-10-01-preview";
const API_MODELS: &str = "2024-10-21";

// ---------------------------------------------------------------------------
// Response / request types
// ---------------------------------------------------------------------------

// ---- Deployments ----------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentModel {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentSku {
    pub name: String,
    #[serde(default)]
    pub capacity: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub name: String,
    pub model: DeploymentModel,
    pub status: String,
    #[serde(default)]
    pub sku: Option<DeploymentSku>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentList {
    pub data: Vec<Deployment>,
}

// ---- Agents / Assistants --------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    pub model: String,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    #[serde(default)]
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentList {
    pub data: Vec<Agent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub id: String,
    pub deleted: bool,
}

// ---- Threads --------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    pub id: String,
    pub object: String,
    pub created_at: i64,
}

// ---- Messages -------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageContentText {
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    #[serde(rename = "text")]
    Text { text: MessageContentText },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub role: String,
    pub content: Vec<MessageContent>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageList {
    pub data: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessageRequest {
    pub role: String,
    pub content: String,
}

// ---- Runs -----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub id: String,
    pub status: String,
    pub assistant_id: String,
    pub thread_id: String,
    pub created_at: i64,
    #[serde(default)]
    pub completed_at: Option<i64>,
    #[serde(default)]
    pub failed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRunRequest {
    pub assistant_id: String,
}

// ---- Chat Completions -----------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default = "default_true")]
    pub stream: bool,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_true() -> bool {
    true
}
fn default_max_tokens() -> u32 {
    4096
}
fn default_temperature() -> f32 {
    0.7
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatDelta {
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    pub delta: ChatDelta,
    #[serde(default)]
    pub index: u32,
    #[serde(default)]
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    pub choices: Vec<ChatChoice>,
}

// ---- Files ----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileObject {
    pub id: String,
    pub filename: String,
    pub bytes: u64,
    pub purpose: String,
    #[serde(default)]
    pub status: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileList {
    pub data: Vec<FileObject>,
}

// ---- Vector Stores --------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileCounts {
    #[serde(default)]
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStore {
    pub id: String,
    pub name: String,
    pub status: String,
    #[serde(default)]
    pub file_counts: Option<VectorStoreFileCounts>,
    #[serde(default)]
    pub usage_bytes: Option<u64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreList {
    pub data: Vec<VectorStore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVectorStoreRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFile {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub vector_store_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorStoreFileList {
    pub data: Vec<VectorStoreFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddVectorStoreFileRequest {
    pub file_id: String,
}

// ---- Fine-Tuning ----------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hyperparameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n_epochs: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJob {
    pub id: String,
    pub model: String,
    pub status: String,
    #[serde(default)]
    pub training_file: Option<String>,
    #[serde(default)]
    pub fine_tuned_model: Option<String>,
    pub created_at: i64,
    #[serde(default)]
    pub finished_at: Option<i64>,
    #[serde(default)]
    pub hyperparameters: Option<Hyperparameters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningJobList {
    pub data: Vec<FineTuningJob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFineTuningRequest {
    pub model: String,
    pub training_file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperparameters: Option<Hyperparameters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningEvent {
    pub id: String,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningEventList {
    pub data: Vec<FineTuningEvent>,
}

// ---- Batch ----------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJob {
    pub id: String,
    pub status: String,
    #[serde(default)]
    pub input_file_id: Option<String>,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub completion_window: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchJobList {
    pub data: Vec<BatchJob>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBatchRequest {
    pub input_file_id: String,
    pub endpoint: String,
    pub completion_window: String,
}

// ---- Connections (data-plane) ---------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryConnection {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub properties: FoundryConnectionProperties,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct FoundryConnectionProperties {
    pub category: Option<String>,
    pub target: Option<String>,
    pub auth_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundryConnectionList {
    pub value: Vec<FoundryConnection>,
}

// ---- Models ---------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub object: Option<String>,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub owned_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelList {
    pub data: Vec<Model>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct FoundryClient {
    pub endpoint: String,
    client: reqwest::Client,
}

impl FoundryClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into().trim_end_matches('/').to_string();
        Self {
            endpoint,
            client: reqwest::Client::new(),
        }
    }

    // -- helpers ------------------------------------------------------------

    fn url(&self, path: &str, api_version: &str) -> String {
        format!(
            "{}{path}?api-version={api_version}",
            self.endpoint
        )
    }

    async fn check_response(&self, resp: reqwest::Response) -> Result<reqwest::Response> {
        if resp.status().is_success() {
            Ok(resp)
        } else {
            let status = resp.status().as_u16();
            let message = resp.text().await.unwrap_or_default();
            Err(FoundryError::Api { status, message })
        }
    }

    fn auth_header(&self, token: &str) -> String {
        format!("Bearer {token}")
    }

    // -----------------------------------------------------------------------
    // 1. Deployments
    // -----------------------------------------------------------------------

    pub async fn list_deployments(&self, token: &str) -> Result<DeploymentList> {
        let resp = self
            .client
            .get(self.url("/deployments", API_DEPLOYMENTS))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 2. Agents (Assistants)
    // -----------------------------------------------------------------------

    pub async fn list_agents(&self, token: &str) -> Result<AgentList> {
        let resp = self
            .client
            .get(self.url("/openai/assistants", API_AGENTS))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn create_agent(
        &self,
        token: &str,
        req: &CreateAgentRequest,
    ) -> Result<Agent> {
        let resp = self
            .client
            .post(self.url("/openai/assistants", API_AGENTS))
            .header("Authorization", self.auth_header(token))
            .json(req)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn delete_agent(&self, token: &str, agent_id: &str) -> Result<DeleteResponse> {
        let resp = self
            .client
            .delete(self.url(
                &format!("/openai/assistants/{agent_id}"),
                API_AGENTS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 3. Threads
    // -----------------------------------------------------------------------

    pub async fn create_thread(&self, token: &str) -> Result<Thread> {
        let resp = self
            .client
            .post(self.url("/openai/threads", API_AGENTS))
            .header("Authorization", self.auth_header(token))
            .json(&serde_json::json!({}))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn list_messages(
        &self,
        token: &str,
        thread_id: &str,
    ) -> Result<MessageList> {
        let resp = self
            .client
            .get(self.url(
                &format!("/openai/threads/{thread_id}/messages"),
                API_AGENTS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 4. Messages & Runs
    // -----------------------------------------------------------------------

    pub async fn create_message(
        &self,
        token: &str,
        thread_id: &str,
        req: &CreateMessageRequest,
    ) -> Result<Message> {
        let resp = self
            .client
            .post(self.url(
                &format!("/openai/threads/{thread_id}/messages"),
                API_AGENTS,
            ))
            .header("Authorization", self.auth_header(token))
            .json(req)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn create_run(
        &self,
        token: &str,
        thread_id: &str,
        req: &CreateRunRequest,
    ) -> Result<Run> {
        let resp = self
            .client
            .post(self.url(
                &format!("/openai/threads/{thread_id}/runs"),
                API_AGENTS,
            ))
            .header("Authorization", self.auth_header(token))
            .json(req)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn get_run(
        &self,
        token: &str,
        thread_id: &str,
        run_id: &str,
    ) -> Result<Run> {
        let resp = self
            .client
            .get(self.url(
                &format!("/openai/threads/{thread_id}/runs/{run_id}"),
                API_AGENTS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 5. Chat Completions (streaming SSE)
    // -----------------------------------------------------------------------

    pub async fn stream_chat(
        &self,
        token: &str,
        deployment: &str,
        req: &ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let resp = self
            .client
            .post(self.url(
                &format!("/openai/deployments/{deployment}/chat/completions"),
                API_DEPLOYMENTS,
            ))
            .header("Authorization", self.auth_header(token))
            .json(req)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;

        let stream = resp.bytes_stream();

        let mapped = stream
            .map(|chunk_result| -> Vec<Result<String>> {
                let chunk = match chunk_result {
                    Ok(b) => b,
                    Err(e) => return vec![Err(FoundryError::Http(e))],
                };

                let text = match std::str::from_utf8(&chunk) {
                    Ok(s) => s.to_string(),
                    Err(e) => {
                        return vec![Err(FoundryError::Stream(format!(
                            "UTF-8 decode error: {e}"
                        )))]
                    }
                };

                let mut results = Vec::new();
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with(':') {
                        continue;
                    }
                    if let Some(data) = line.strip_prefix("data: ") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            break;
                        }
                        match serde_json::from_str::<ChatChunk>(data) {
                            Ok(chunk) => {
                                if let Some(choice) = chunk.choices.first() {
                                    if let Some(content) = &choice.delta.content {
                                        results.push(Ok(content.clone()));
                                    }
                                }
                            }
                            Err(e) => {
                                results.push(Err(FoundryError::Json(e)));
                            }
                        }
                    }
                }
                results
            })
            .flat_map(futures_util::stream::iter);

        Ok(Box::pin(mapped))
    }

    // -----------------------------------------------------------------------
    // 6. Files
    // -----------------------------------------------------------------------

    pub async fn list_files(&self, token: &str) -> Result<FileList> {
        let resp = self
            .client
            .get(self.url("/openai/files", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn upload_file(
        &self,
        token: &str,
        purpose: &str,
        filename: &str,
        data: Vec<u8>,
    ) -> Result<FileObject> {
        let file_part = multipart::Part::bytes(data)
            .file_name(filename.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| FoundryError::Http(e))?;

        let form = multipart::Form::new()
            .text("purpose", purpose.to_string())
            .part("file", file_part);

        let resp = self
            .client
            .post(self.url("/openai/files", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .multipart(form)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn delete_file(&self, token: &str, file_id: &str) -> Result<DeleteResponse> {
        let resp = self
            .client
            .delete(self.url(
                &format!("/openai/files/{file_id}"),
                API_MODELS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 7. Vector Stores
    // -----------------------------------------------------------------------

    pub async fn list_vector_stores(&self, token: &str) -> Result<VectorStoreList> {
        let resp = self
            .client
            .get(self.url("/openai/vector_stores", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn create_vector_store(
        &self,
        token: &str,
        req: &CreateVectorStoreRequest,
    ) -> Result<VectorStore> {
        let resp = self
            .client
            .post(self.url("/openai/vector_stores", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .json(req)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn delete_vector_store(
        &self,
        token: &str,
        store_id: &str,
    ) -> Result<DeleteResponse> {
        let resp = self
            .client
            .delete(self.url(
                &format!("/openai/vector_stores/{store_id}"),
                API_MODELS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn list_vector_store_files(
        &self,
        token: &str,
        store_id: &str,
    ) -> Result<VectorStoreFileList> {
        let resp = self
            .client
            .get(self.url(
                &format!("/openai/vector_stores/{store_id}/files"),
                API_MODELS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn add_vector_store_file(
        &self,
        token: &str,
        store_id: &str,
        req: &AddVectorStoreFileRequest,
    ) -> Result<VectorStoreFile> {
        let resp = self
            .client
            .post(self.url(
                &format!("/openai/vector_stores/{store_id}/files"),
                API_MODELS,
            ))
            .header("Authorization", self.auth_header(token))
            .json(req)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 8. Fine-Tuning
    // -----------------------------------------------------------------------

    pub async fn list_fine_tuning_jobs(&self, token: &str) -> Result<FineTuningJobList> {
        let resp = self
            .client
            .get(self.url("/openai/fine_tuning/jobs", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn create_fine_tuning_job(
        &self,
        token: &str,
        req: &CreateFineTuningRequest,
    ) -> Result<FineTuningJob> {
        let resp = self
            .client
            .post(self.url("/openai/fine_tuning/jobs", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .json(req)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn cancel_fine_tuning_job(
        &self,
        token: &str,
        job_id: &str,
    ) -> Result<FineTuningJob> {
        let resp = self
            .client
            .post(self.url(
                &format!("/openai/fine_tuning/jobs/{job_id}/cancel"),
                API_MODELS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn list_fine_tuning_events(
        &self,
        token: &str,
        job_id: &str,
    ) -> Result<FineTuningEventList> {
        let resp = self
            .client
            .get(self.url(
                &format!("/openai/fine_tuning/jobs/{job_id}/events"),
                API_MODELS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 9. Batch Jobs
    // -----------------------------------------------------------------------

    pub async fn list_batches(&self, token: &str) -> Result<BatchJobList> {
        let resp = self
            .client
            .get(self.url("/openai/batches", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn create_batch(
        &self,
        token: &str,
        req: &CreateBatchRequest,
    ) -> Result<BatchJob> {
        let resp = self
            .client
            .post(self.url("/openai/batches", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .json(req)
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    pub async fn cancel_batch(&self, token: &str, batch_id: &str) -> Result<BatchJob> {
        let resp = self
            .client
            .post(self.url(
                &format!("/openai/batches/{batch_id}/cancel"),
                API_MODELS,
            ))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 10. Connections (data-plane)
    // -----------------------------------------------------------------------

    pub async fn list_connections(&self, token: &str) -> Result<FoundryConnectionList> {
        let resp = self
            .client
            .get(self.url("/connections", API_DEPLOYMENTS))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }

    // -----------------------------------------------------------------------
    // 11. Models
    // -----------------------------------------------------------------------

    pub async fn list_models(&self, token: &str) -> Result<ModelList> {
        let resp = self
            .client
            .get(self.url("/openai/models", API_MODELS))
            .header("Authorization", self.auth_header(token))
            .send()
            .await?;
        let resp = self.check_response(resp).await?;
        Ok(resp.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_construction() {
        let client = FoundryClient::new("https://account.cognitiveservices.azure.com/");
        assert_eq!(
            client.url("/deployments", API_DEPLOYMENTS),
            "https://account.cognitiveservices.azure.com/deployments?api-version=2024-10-01-preview"
        );
        assert_eq!(
            client.url("/openai/assistants", API_AGENTS),
            "https://account.cognitiveservices.azure.com/openai/assistants?api-version=2024-05-01-preview"
        );
        assert_eq!(
            client.url("/openai/models", API_MODELS),
            "https://account.cognitiveservices.azure.com/openai/models?api-version=2024-10-21"
        );
    }

    #[test]
    fn test_endpoint_trailing_slash_stripped() {
        let client = FoundryClient::new("https://example.com///");
        assert_eq!(client.endpoint, "https://example.com");
    }

    #[test]
    fn test_deserialize_agent() {
        let json = r#"{
            "id": "asst_abc123",
            "name": "Test Agent",
            "model": "gpt-4",
            "instructions": "You are helpful.",
            "tools": [{"type": "code_interpreter"}],
            "created_at": 1700000000
        }"#;
        let agent: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(agent.id, "asst_abc123");
        assert_eq!(agent.tools.len(), 1);
        assert_eq!(agent.tools[0].tool_type, "code_interpreter");
    }

    #[test]
    fn test_deserialize_deployment() {
        let json = r#"{
            "id": "dep-1",
            "name": "gpt-4o",
            "model": {"name": "gpt-4o", "version": "2024-05-13"},
            "status": "succeeded",
            "sku": {"name": "Standard", "capacity": 120}
        }"#;
        let dep: Deployment = serde_json::from_str(json).unwrap();
        assert_eq!(dep.name, "gpt-4o");
        assert_eq!(dep.model.version.as_deref(), Some("2024-05-13"));
        assert_eq!(dep.sku.unwrap().capacity, Some(120));
    }

    #[test]
    fn test_deserialize_chat_chunk() {
        let json = r#"{"choices":[{"delta":{"content":"Hello"},"index":0}]}"#;
        let chunk: ChatChunk = serde_json::from_str(json).unwrap();
        assert_eq!(
            chunk.choices[0].delta.content.as_deref(),
            Some("Hello")
        );
    }

    #[test]
    fn test_serialize_create_agent_optional_fields() {
        let req = CreateAgentRequest {
            name: "test".into(),
            model: "gpt-4".into(),
            instructions: None,
            tools: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("instructions"));
        assert!(!json.contains("tools"));
    }

    #[test]
    fn test_deserialize_file_object() {
        let json = r#"{
            "id": "file-abc",
            "filename": "data.jsonl",
            "bytes": 1024,
            "purpose": "fine-tune",
            "status": "processed",
            "created_at": 1700000000
        }"#;
        let file: FileObject = serde_json::from_str(json).unwrap();
        assert_eq!(file.id, "file-abc");
        assert_eq!(file.bytes, 1024);
    }

    #[test]
    fn test_deserialize_vector_store() {
        let json = r#"{
            "id": "vs-123",
            "name": "My Store",
            "status": "completed",
            "file_counts": {"total": 5},
            "usage_bytes": 2048,
            "created_at": 1700000000
        }"#;
        let vs: VectorStore = serde_json::from_str(json).unwrap();
        assert_eq!(vs.file_counts.unwrap().total, 5);
        assert_eq!(vs.usage_bytes, Some(2048));
    }
}
