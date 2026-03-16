use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};
use std::fmt;

const BASE_URL: &str = "https://management.azure.com";

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArmError {
    HttpError(String),
    DeserializationError(String),
    NotFound(String),
    Unauthorized(String),
    Unknown(String),
}

impl fmt::Display for ArmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArmError::HttpError(msg) => write!(f, "HTTP error: {msg}"),
            ArmError::DeserializationError(msg) => write!(f, "Deserialization error: {msg}"),
            ArmError::NotFound(msg) => write!(f, "Not found: {msg}"),
            ArmError::Unauthorized(msg) => write!(f, "Unauthorized: {msg}"),
            ArmError::Unknown(msg) => write!(f, "Unknown error: {msg}"),
        }
    }
}

impl std::error::Error for ArmError {}

impl From<reqwest::Error> for ArmError {
    fn from(err: reqwest::Error) -> Self {
        ArmError::HttpError(err.to_string())
    }
}

impl From<serde_json::Error> for ArmError {
    fn from(err: serde_json::Error) -> Self {
        ArmError::DeserializationError(err.to_string())
    }
}

// ---------------------------------------------------------------------------
// Domain structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    pub subscription_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub location: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub properties: WorkspaceProperties,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkspaceProperties {
    pub discovery_url: Option<String>,
    pub endpoint: Option<String>,
    pub hub_resource_id: Option<String>,
}

impl Workspace {
    pub fn is_hub(&self) -> bool {
        let k = self.kind.to_lowercase();
        k == "hub" || k == "default"
    }

    pub fn is_project(&self) -> bool {
        self.kind.eq_ignore_ascii_case("project")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmDeployment {
    pub name: String,
    #[serde(default)]
    pub properties: DeploymentProperties,
    #[serde(default)]
    pub sku: Option<DeploymentSku>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct DeploymentProperties {
    pub provisioning_state: Option<String>,
    pub model: Option<DeploymentModel>,
}

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
    pub capacity: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmConnection {
    pub name: String,
    #[serde(default)]
    pub properties: ConnectionProperties,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ConnectionProperties {
    pub category: Option<String>,
    pub target: Option<String>,
    pub metadata: Option<ConnectionMetadata>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct ConnectionMetadata {
    pub resource_id: Option<String>,
}

// ---------------------------------------------------------------------------
// Response wrappers
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListResponse<T> {
    value: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CognitiveAccountResponse {
    properties: CognitiveAccountProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CognitiveAccountProperties {
    endpoint: String,
}

// ---------------------------------------------------------------------------
// Discovery result
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub subscriptions: Vec<Subscription>,
    pub workspaces: Vec<Workspace>,
    pub hubs: Vec<HubDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubDetail {
    pub workspace: Workspace,
    pub endpoint: Option<String>,
    pub deployments: Vec<ArmDeployment>,
    pub connections: Vec<ArmConnection>,
    pub projects: Vec<Workspace>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        format!("Bearer {token}")
            .parse()
            .expect("invalid token header value"),
    );
    headers.insert(ACCEPT, "application/json".parse().unwrap());
    headers
}

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn arm_get<T: serde::de::DeserializeOwned>(token: &str, url: &str) -> Result<T, ArmError> {
    let resp = client().get(url).headers(build_headers(token)).send().await?;

    let status = resp.status();
    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        let body = resp.text().await.unwrap_or_default();
        return Err(ArmError::Unauthorized(body));
    }
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(ArmError::NotFound(url.to_string()));
    }
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(ArmError::HttpError(format!("status {status}: {body}")));
    }

    let body = resp.text().await?;
    serde_json::from_str::<T>(&body).map_err(|e| {
        ArmError::DeserializationError(format!("{e} — body: {}", &body[..body.len().min(512)]))
    })
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all Azure subscriptions accessible with the given ARM token.
pub async fn list_subscriptions(token: &str) -> Result<Vec<Subscription>, ArmError> {
    let url = format!("{BASE_URL}/subscriptions?api-version=2022-12-01");
    let resp: ListResponse<Subscription> = arm_get(token, &url).await?;
    Ok(resp.value)
}

/// List all Machine Learning Services workspaces (hubs and projects) in a subscription.
pub async fn list_workspaces(
    token: &str,
    subscription_id: &str,
) -> Result<Vec<Workspace>, ArmError> {
    let url = format!(
        "{BASE_URL}/subscriptions/{subscription_id}/providers/Microsoft.MachineLearningServices/workspaces?api-version=2024-10-01"
    );
    let resp: ListResponse<Workspace> = arm_get(token, &url).await?;
    Ok(resp.value)
}

/// Resolve the data-plane endpoint for a hub by inspecting its AI Services connection.
///
/// Walks the hub's connections, finds the first `AIServices` or `AzureOpenAI` connection whose
/// metadata contains a Cognitive Services resource ID, then fetches the endpoint from that resource.
pub async fn resolve_endpoint(
    token: &str,
    hub_resource_id: &str,
) -> Result<String, ArmError> {
    let connections = list_connections(token, hub_resource_id).await?;

    let ai_connection = connections.iter().find(|c| {
        matches!(
            c.properties.category.as_deref(),
            Some("AIServices") | Some("AzureOpenAI")
        )
    });

    let connection = ai_connection.ok_or_else(|| {
        ArmError::NotFound("no AIServices or AzureOpenAI connection found on hub".into())
    })?;

    let resource_id = connection
        .properties
        .metadata
        .as_ref()
        .and_then(|m| m.resource_id.as_deref())
        .ok_or_else(|| {
            ArmError::NotFound("connection metadata missing ResourceId".into())
        })?;

    let url = format!("{BASE_URL}{resource_id}?api-version=2023-05-01");
    let account: CognitiveAccountResponse = arm_get(token, &url).await?;
    Ok(account.properties.endpoint)
}

/// List model deployments under a Cognitive Services / AI Services resource.
pub async fn list_deployments(
    token: &str,
    ai_services_resource_id: &str,
) -> Result<Vec<ArmDeployment>, ArmError> {
    let url = format!(
        "{BASE_URL}{ai_services_resource_id}/deployments?api-version=2024-10-01"
    );
    let resp: ListResponse<ArmDeployment> = arm_get(token, &url).await?;
    Ok(resp.value)
}

/// List connections on a workspace or hub.
pub async fn list_connections(
    token: &str,
    resource_id: &str,
) -> Result<Vec<ArmConnection>, ArmError> {
    let url = format!("{BASE_URL}{resource_id}/connections?api-version=2024-10-01");
    let resp: ListResponse<ArmConnection> = arm_get(token, &url).await?;
    Ok(resp.value)
}

/// Orchestrate full discovery: subscriptions → workspaces → hub details.
///
/// For every hub found, resolves its data-plane endpoint and lists deployments
/// and connections. Projects are grouped under their parent hub.
pub async fn discover_all(token: &str) -> Result<DiscoveryResult, ArmError> {
    let subscriptions = list_subscriptions(token).await?;

    let mut all_workspaces: Vec<Workspace> = Vec::new();
    for sub in &subscriptions {
        match list_workspaces(token, &sub.subscription_id).await {
            Ok(ws) => all_workspaces.extend(ws),
            Err(ArmError::Unauthorized(_)) => continue, // skip inaccessible subscriptions
            Err(e) => return Err(e),
        }
    }

    let hubs: Vec<&Workspace> = all_workspaces.iter().filter(|w| w.is_hub()).collect();
    let projects: Vec<&Workspace> = all_workspaces.iter().filter(|w| w.is_project()).collect();

    let mut hub_details: Vec<HubDetail> = Vec::new();

    for hub in &hubs {
        let hub_projects: Vec<Workspace> = projects
            .iter()
            .filter(|p| {
                p.properties
                    .hub_resource_id
                    .as_deref()
                    .map(|id| id.eq_ignore_ascii_case(&hub.id))
                    .unwrap_or(false)
            })
            .cloned()
            .cloned()
            .collect();

        // Resolve endpoint + list deployments; tolerate failures per-hub
        let endpoint = resolve_endpoint(token, &hub.id).await.ok();

        let ai_resource_id = list_connections(token, &hub.id)
            .await
            .unwrap_or_default()
            .iter()
            .find(|c| {
                matches!(
                    c.properties.category.as_deref(),
                    Some("AIServices") | Some("AzureOpenAI")
                )
            })
            .and_then(|c| {
                c.properties
                    .metadata
                    .as_ref()
                    .and_then(|m| m.resource_id.clone())
            });

        let deployments = match &ai_resource_id {
            Some(id) => list_deployments(token, id).await.unwrap_or_default(),
            None => Vec::new(),
        };

        let connections = list_connections(token, &hub.id)
            .await
            .unwrap_or_default();

        hub_details.push(HubDetail {
            workspace: (*hub).clone(),
            endpoint,
            deployments,
            connections,
            projects: hub_projects,
        });
    }

    Ok(DiscoveryResult {
        subscriptions,
        workspaces: all_workspaces,
        hubs: hub_details,
    })
}
