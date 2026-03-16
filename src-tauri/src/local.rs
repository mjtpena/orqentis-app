use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use futures_util::Stream;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A locally-running agent or model runtime discovered on the machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAgent {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub runtime: String,
    pub status: String,
    #[serde(default)]
    pub model: Option<String>,
    pub endpoint: String,
}

// ---------------------------------------------------------------------------
// Ollama
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize)]
struct OllamaModel {
    name: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    modified_at: Option<String>,
}

async fn probe_ollama() -> Vec<LocalAgent> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let endpoint = "http://localhost:11434";
    let url = format!("{endpoint}/api/tags");

    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return Vec::new(),
    };

    let tags: OllamaTagsResponse = match resp.json().await {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    tags.models
        .into_iter()
        .map(|m| LocalAgent {
            id: format!("ollama-{}", m.name),
            name: m.name.clone(),
            description: Some(format!(
                "Ollama model{}",
                m.size
                    .map(|s| format!(" · {:.1} GB", s as f64 / 1_073_741_824.0))
                    .unwrap_or_default()
            )),
            runtime: "Ollama".to_string(),
            status: "running".to_string(),
            model: Some(m.model.unwrap_or(m.name)),
            endpoint: endpoint.to_string(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// LM Studio
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
    #[serde(default)]
    owned_by: Option<String>,
}

async fn probe_lmstudio() -> Vec<LocalAgent> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let endpoint = "http://localhost:1234";
    let url = format!("{endpoint}/v1/models");

    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return Vec::new(),
    };

    let models: OpenAIModelsResponse = match resp.json().await {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    models
        .data
        .into_iter()
        .map(|m| LocalAgent {
            id: format!("lmstudio-{}", m.id),
            name: m.id.clone(),
            description: Some(format!(
                "LM Studio model{}",
                m.owned_by
                    .as_deref()
                    .map(|o| format!(" · {o}"))
                    .unwrap_or_default()
            )),
            runtime: "LM Studio".to_string(),
            status: "running".to_string(),
            model: Some(m.id),
            endpoint: endpoint.to_string(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// LocalAI / vLLM / generic OpenAI-compatible
// ---------------------------------------------------------------------------

async fn probe_openai_compatible(
    endpoint: &str,
    runtime_name: &str,
) -> Vec<LocalAgent> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap_or_default();

    let url = format!("{}/v1/models", endpoint.trim_end_matches('/'));

    let resp = match client.get(&url).send().await {
        Ok(r) if r.status().is_success() => r,
        _ => return Vec::new(),
    };

    let models: OpenAIModelsResponse = match resp.json().await {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };

    models
        .data
        .into_iter()
        .map(|m| LocalAgent {
            id: format!("{}-{}", runtime_name.to_lowercase().replace(' ', "-"), m.id),
            name: m.id.clone(),
            description: Some(format!("{runtime_name} model")),
            runtime: runtime_name.to_string(),
            status: "running".to_string(),
            model: Some(m.id),
            endpoint: endpoint.to_string(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Discover all locally-running AI runtimes by probing well-known ports.
pub async fn discover_local_agents() -> Vec<LocalAgent> {
    let (ollama, lmstudio, localai, vllm) = tokio::join!(
        probe_ollama(),
        probe_lmstudio(),
        probe_openai_compatible("http://localhost:8080", "LocalAI"),
        probe_openai_compatible("http://localhost:8000", "vLLM"),
    );

    let mut all = Vec::new();
    all.extend(ollama);
    all.extend(lmstudio);
    all.extend(localai);
    all.extend(vllm);
    all
}

// ---------------------------------------------------------------------------
// Local Chat (OpenAI-compatible streaming)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
struct LocalChatRequest {
    model: String,
    messages: Vec<LocalChatMessage>,
    stream: bool,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Debug, Deserialize)]
struct LocalChatChunk {
    choices: Vec<LocalChatChoice>,
}

#[derive(Debug, Deserialize)]
struct LocalChatChoice {
    delta: LocalChatDelta,
}

#[derive(Debug, Deserialize)]
struct LocalChatDelta {
    content: Option<String>,
}

/// Stream chat completions from a local OpenAI-compatible endpoint (Ollama, LM Studio, etc.)
pub async fn stream_local_chat(
    endpoint: &str,
    model: &str,
    messages: Vec<LocalChatMessage>,
) -> Result<Pin<Box<dyn Stream<Item = Result<String, String>> + Send>>, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| e.to_string())?;

    // Ollama uses /api/chat for its native API, but also supports /v1/chat/completions
    let url = format!("{}/v1/chat/completions", endpoint.trim_end_matches('/'));

    let req = LocalChatRequest {
        model: model.to_string(),
        messages,
        stream: true,
        max_tokens: 4096,
        temperature: 0.7,
    };

    let resp = client
        .post(&url)
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("Failed to connect to local model: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Local model returned {status}: {body}"));
    }

    let stream = resp.bytes_stream();

    let mapped = stream
        .map(|chunk_result| -> Vec<Result<String, String>> {
            let chunk = match chunk_result {
                Ok(b) => b,
                Err(e) => return vec![Err(e.to_string())],
            };

            let text = match std::str::from_utf8(&chunk) {
                Ok(s) => s.to_string(),
                Err(e) => return vec![Err(format!("UTF-8 decode error: {e}"))],
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
                    if let Ok(chunk) = serde_json::from_str::<LocalChatChunk>(data) {
                        if let Some(choice) = chunk.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                results.push(Ok(content.clone()));
                            }
                        }
                    }
                }
            }
            results
        })
        .flat_map(futures_util::stream::iter);

    Ok(Box::pin(mapped))
}
