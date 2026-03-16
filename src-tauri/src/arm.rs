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
// Usage metrics types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricResponse {
    pub value: Vec<MetricItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricItem {
    pub name: MetricName,
    pub timeseries: Vec<TimeSeries>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricName {
    pub value: String,
    pub localized_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeSeries {
    pub data: Vec<MetricDataPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricDataPoint {
    pub time_stamp: String,
    #[serde(default)]
    pub total: Option<f64>,
}

/// Aggregated usage metrics returned to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetrics {
    pub total_calls: f64,
    pub successful_calls: f64,
    pub prompt_tokens: f64,
    pub completion_tokens: f64,
    pub total_tokens: f64,
    /// Daily breakdown for charts
    pub daily: Vec<DailyMetric>,
    /// Cost data from Azure Cost Management
    pub cost: Option<CostSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyMetric {
    pub date: String,
    pub total_calls: f64,
    pub prompt_tokens: f64,
    pub completion_tokens: f64,
}

// ---------------------------------------------------------------------------
// Cost Management types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSummary {
    pub total_cost: f64,
    pub currency: String,
    pub daily: Vec<DailyCost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyCost {
    pub date: String,
    pub cost: f64,
}

/// Raw response from Azure Cost Management Query API
#[derive(Debug, Clone, Deserialize)]
pub struct CostQueryResponse {
    pub properties: CostQueryProperties,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CostQueryProperties {
    pub columns: Vec<CostQueryColumn>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CostQueryColumn {
    pub name: String,
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
    parse_arm_response(resp, url).await
}

async fn arm_post<T: serde::de::DeserializeOwned, B: Serialize>(
    token: &str,
    url: &str,
    body: &B,
) -> Result<T, ArmError> {
    let resp = client()
        .post(url)
        .headers(build_headers(token))
        .json(body)
        .send()
        .await?;
    parse_arm_response(resp, url).await
}

async fn parse_arm_response<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
    url: &str,
) -> Result<T, ArmError> {
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

/// Query Azure Monitor metrics for an AI Services resource.
///
/// Fetches processed prompt/completion tokens, successful calls, and total calls
/// for the last 30 days with daily granularity.
pub async fn query_usage_metrics(
    token: &str,
    ai_services_resource_id: &str,
) -> Result<UsageMetrics, ArmError> {
    let now = chrono::Utc::now();
    let thirty_days_ago = now - chrono::Duration::days(30);
    let timespan = format!(
        "{}/{}",
        thirty_days_ago.format("%Y-%m-%dT00:00:00Z"),
        now.format("%Y-%m-%dT23:59:59Z")
    );

    let metric_names = "ProcessedPromptTokens,GeneratedTokens,SuccessfulCalls,TotalCalls";

    let url = format!(
        "{BASE_URL}{ai_services_resource_id}/providers/microsoft.insights/metrics\
         ?api-version=2024-02-01\
         &metricnames={metric_names}\
         &timespan={timespan}\
         &interval=P1D\
         &aggregation=total"
    );

    let resp: MetricResponse = arm_get(token, &url).await?;

    // Parse the metric response into our aggregated structure
    let mut total_calls = 0.0_f64;
    let mut successful_calls = 0.0_f64;
    let mut prompt_tokens = 0.0_f64;
    let mut completion_tokens = 0.0_f64;

    // Collect daily data keyed by date
    let mut daily_map: std::collections::BTreeMap<String, DailyMetric> =
        std::collections::BTreeMap::new();

    for metric in &resp.value {
        let name = metric.name.value.as_str();
        for ts in &metric.timeseries {
            for dp in &ts.data {
                let val = dp.total.unwrap_or(0.0);
                let date = dp.time_stamp.split('T').next().unwrap_or("").to_string();

                let entry = daily_map.entry(date.clone()).or_insert(DailyMetric {
                    date: date.clone(),
                    total_calls: 0.0,
                    prompt_tokens: 0.0,
                    completion_tokens: 0.0,
                });

                match name {
                    "ProcessedPromptTokens" => {
                        prompt_tokens += val;
                        entry.prompt_tokens += val;
                    }
                    "GeneratedTokens" => {
                        completion_tokens += val;
                        entry.completion_tokens += val;
                    }
                    "SuccessfulCalls" => {
                        successful_calls += val;
                    }
                    "TotalCalls" => {
                        total_calls += val;
                        entry.total_calls += val;
                    }
                    _ => {}
                }
            }
        }
    }

    // Fetch cost data from Azure Cost Management (best-effort)
    let cost = query_cost_data(token, ai_services_resource_id).await.ok();

    Ok(UsageMetrics {
        total_calls,
        successful_calls,
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
        daily: daily_map.into_values().collect(),
        cost,
    })
}

/// Query Azure Cost Management for actual dollar costs of an AI Services resource.
pub async fn query_cost_data(
    token: &str,
    ai_services_resource_id: &str,
) -> Result<CostSummary, ArmError> {
    // Extract subscription ID from resource ID
    // Format: /subscriptions/{sub}/resourceGroups/{rg}/providers/...
    let parts: Vec<&str> = ai_services_resource_id.split('/').collect();
    let sub_idx = parts.iter().position(|p| p.eq_ignore_ascii_case("subscriptions"))
        .ok_or_else(|| ArmError::NotFound("cannot extract subscription from resource ID".into()))?;
    let subscription_id = parts.get(sub_idx + 1)
        .ok_or_else(|| ArmError::NotFound("invalid resource ID format".into()))?;

    let now = chrono::Utc::now();
    let thirty_days_ago = now - chrono::Duration::days(30);

    let body = serde_json::json!({
        "type": "ActualCost",
        "timeframe": "Custom",
        "timePeriod": {
            "from": thirty_days_ago.format("%Y-%m-%dT00:00:00Z").to_string(),
            "to": now.format("%Y-%m-%dT23:59:59Z").to_string()
        },
        "dataset": {
            "granularity": "Daily",
            "aggregation": {
                "totalCost": {
                    "name": "Cost",
                    "function": "Sum"
                }
            },
            "filter": {
                "dimensions": {
                    "name": "ResourceId",
                    "operator": "In",
                    "values": [ai_services_resource_id]
                }
            }
        }
    });

    let url = format!(
        "{BASE_URL}/subscriptions/{subscription_id}/providers/Microsoft.CostManagement/query?api-version=2023-11-01"
    );

    let resp: CostQueryResponse = arm_post(token, &url, &body).await?;

    // Find column indices
    let cost_idx = resp.properties.columns.iter().position(|c| c.name == "Cost").unwrap_or(0);
    let date_idx = resp.properties.columns.iter().position(|c| c.name == "UsageDate").unwrap_or(1);
    let currency_idx = resp.properties.columns.iter().position(|c| c.name == "Currency").unwrap_or(2);

    let mut total_cost = 0.0_f64;
    let mut currency = String::from("USD");
    let mut daily: Vec<DailyCost> = Vec::new();

    for row in &resp.properties.rows {
        let cost_val = row.get(cost_idx)
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let date_val = row.get(date_idx)
            .and_then(|v| v.as_i64().map(|n| n.to_string()).or_else(|| v.as_str().map(String::from)))
            .unwrap_or_default();
        if let Some(c) = row.get(currency_idx).and_then(|v| v.as_str()) {
            currency = c.to_string();
        }

        // UsageDate comes as YYYYMMDD integer
        let date_str = if date_val.len() == 8 {
            format!("{}-{}-{}", &date_val[..4], &date_val[4..6], &date_val[6..8])
        } else {
            date_val
        };

        total_cost += cost_val;
        daily.push(DailyCost {
            date: date_str,
            cost: cost_val,
        });
    }

    Ok(CostSummary {
        total_cost,
        currency,
        daily,
    })
}
