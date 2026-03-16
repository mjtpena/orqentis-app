use reqwest::Client;
use serde::{Deserialize, Serialize};

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
