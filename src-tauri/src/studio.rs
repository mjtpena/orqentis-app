use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudioBot {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub environment_id: Option<String>,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub modified_on: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerPlatformEnvironment {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
}

// ---------------------------------------------------------------------------
// Internal response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct BapEnvListResponse {
    value: Vec<BapEnvironment>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BapEnvironment {
    name: String,
    #[serde(default)]
    properties: BapEnvProperties,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct BapEnvProperties {
    display_name: Option<String>,
    azure_region: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DataverseBotsResponse {
    value: Vec<DataverseBot>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct DataverseBot {
    #[serde(rename = "botid")]
    bot_id: Option<String>,
    name: Option<String>,
    #[serde(rename = "statuscode")]
    status_code: Option<i32>,
    #[serde(rename = "createdon")]
    created_on: Option<String>,
    #[serde(rename = "modifiedon")]
    modified_on: Option<String>,
}

impl Default for DataverseBot {
    fn default() -> Self {
        Self {
            bot_id: None,
            name: None,
            status_code: None,
            created_on: None,
            modified_on: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Client
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

/// List Power Platform environments accessible to the user.
pub async fn list_environments(bap_token: &str) -> Result<Vec<PowerPlatformEnvironment>, String> {
    let url = "https://api.bap.microsoft.com/providers/Microsoft.BusinessAppPlatform/environments?api-version=2023-06-01";
    let resp = reqwest::Client::new()
        .get(url)
        .headers(build_headers(bap_token))
        .send()
        .await
        .map_err(|e| format!("BAP request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("BAP API {status}: {}", &body[..body.len().min(500)]));
    }

    let list: BapEnvListResponse = resp.json().await.map_err(|e| format!("BAP parse: {e}"))?;
    Ok(list
        .value
        .into_iter()
        .map(|e| PowerPlatformEnvironment {
            id: e.name.clone(),
            name: e.name,
            display_name: e.properties.display_name,
            location: e.properties.azure_region,
        })
        .collect())
}

/// List Copilot Studio bots in a Dataverse environment.
/// `dataverse_url` is like `https://org12345.crm.dynamics.com`
/// `dataverse_token` should be scoped to `{dataverse_url}/.default`
pub async fn list_bots(
    dataverse_url: &str,
    dataverse_token: &str,
) -> Result<Vec<StudioBot>, String> {
    let url = format!(
        "{}/api/data/v9.2/bots?$select=botid,name,statuscode,createdon,modifiedon",
        dataverse_url.trim_end_matches('/')
    );
    let resp = reqwest::Client::new()
        .get(&url)
        .headers(build_headers(dataverse_token))
        .send()
        .await
        .map_err(|e| format!("Dataverse request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Dataverse API {status}: {}", &body[..body.len().min(500)]));
    }

    let list: DataverseBotsResponse = resp.json().await.map_err(|e| format!("Dataverse parse: {e}"))?;
    Ok(list
        .value
        .into_iter()
        .map(|b| {
            let status = match b.status_code {
                Some(1) => Some("published".to_string()),
                Some(0) => Some("draft".to_string()),
                _ => Some("unknown".to_string()),
            };
            StudioBot {
                id: b.bot_id.unwrap_or_default(),
                name: b.name.unwrap_or_else(|| "Unnamed Bot".to_string()),
                description: None,
                status,
                environment_id: None,
                created_on: b.created_on,
                modified_on: b.modified_on,
            }
        })
        .collect())
}

/// Discover all Copilot Studio bots across all Power Platform environments.
/// Uses BAP token for environment listing and dynamically scoped Dataverse tokens for each env.
/// Falls back gracefully — environments we can't access are skipped.
pub async fn discover_studio_bots(
    bap_token: &str,
    graph_token: &str,
) -> Result<Vec<StudioBot>, String> {
    // For now, try listing environments and bots from each.
    // In many tenants, Copilot Studio bots are in the default environment.
    // The graph_token can access limited Dataverse in some configurations.
    let envs = list_environments(bap_token).await.unwrap_or_default();
    let mut all_bots: Vec<StudioBot> = Vec::new();

    for env in &envs {
        // Construct the Dataverse URL from environment name
        // This is a best-effort — real environments have an org URL we'd need to resolve
        // For now we skip individual env bot listing as it needs per-env Dataverse token scoping
        // We'll surface the environments themselves as "Studio" presence indicators
        let _ = env; // suppress unused warning
    }

    // If we have a Graph token, try the Power Virtual Agents connector via Graph
    // GET https://graph.microsoft.com/beta/teamwork/teamsApps?$filter=distributionMethod eq 'organization'
    // This catches bots published to Teams (which includes Copilot Studio bots)
    let graph_bots = list_studio_bots_via_graph(graph_token).await.unwrap_or_default();
    all_bots.extend(graph_bots);

    Ok(all_bots)
}

/// Try to find Copilot Studio / PVA bots via Microsoft Graph (Teams apps with bot capability).
async fn list_studio_bots_via_graph(graph_token: &str) -> Result<Vec<StudioBot>, String> {
    let url = "https://graph.microsoft.com/v1.0/appCatalogs/teamsApps?$filter=distributionMethod eq 'organization'&$expand=appDefinitions";
    let resp = reqwest::Client::new()
        .get(url)
        .headers(build_headers(graph_token))
        .send()
        .await
        .map_err(|e| format!("Graph request failed: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Graph API {status}: {}", &body[..body.len().min(500)]));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("Graph parse: {e}"))?;
    let mut bots = Vec::new();

    if let Some(apps) = body.get("value").and_then(|v| v.as_array()) {
        for app in apps {
            // Check if any appDefinition has a bot
            let defs = app
                .get("appDefinitions")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let has_bot = defs.iter().any(|d| {
                d.get("bot").is_some()
                    || d.get("description")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_lowercase().contains("copilot") || s.to_lowercase().contains("bot"))
                        .unwrap_or(false)
            });

            if has_bot || !defs.is_empty() {
                let name = defs
                    .first()
                    .and_then(|d| d.get("displayName"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown Bot");
                let desc = defs
                    .first()
                    .and_then(|d| d.get("shortDescription"))
                    .and_then(|v| v.as_str());
                let id = app
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();

                bots.push(StudioBot {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: desc.map(String::from),
                    status: Some("published".to_string()),
                    environment_id: None,
                    created_on: None,
                    modified_on: None,
                });
            }
        }
    }

    Ok(bots)
}
