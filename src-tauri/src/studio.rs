use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};

use crate::auth;

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
    pub environment_name: Option<String>,
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
    #[serde(default)]
    pub dataverse_url: Option<String>,
    #[serde(default)]
    pub dataverse_api_url: Option<String>,
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
    linked_environment_metadata: Option<LinkedEnvMetadata>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
struct LinkedEnvMetadata {
    instance_url: Option<String>,
    instance_api_url: Option<String>,
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
        .map(|e| {
            let linked = e.properties.linked_environment_metadata.as_ref();
            PowerPlatformEnvironment {
                id: e.name.clone(),
                name: e.name,
                display_name: e.properties.display_name,
                location: e.properties.azure_region,
                dataverse_url: linked.and_then(|l| l.instance_url.clone()),
                dataverse_api_url: linked.and_then(|l| l.instance_api_url.clone()),
            }
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
                environment_name: None,
                created_on: b.created_on,
                modified_on: b.modified_on,
            }
        })
        .collect())
}

/// Discover all Copilot Studio bots across all Power Platform environments.
/// Uses BAP token for environment listing and Azure CLI for per-env Dataverse tokens.
pub async fn discover_studio_bots(
    bap_token: &str,
    _graph_token: &str,
) -> Result<Vec<StudioBot>, String> {
    let envs = list_environments(bap_token).await.unwrap_or_default();
    log::info!("[studio] found {} Power Platform environments", envs.len());
    let mut all_bots: Vec<StudioBot> = Vec::new();

    for env in &envs {
        let api_url = match &env.dataverse_api_url {
            Some(u) if !u.is_empty() => u.clone(),
            _ => {
                log::info!("[studio] env '{}' has no Dataverse API URL, skipping", env.display_name.as_deref().unwrap_or(&env.name));
                continue;
            }
        };

        // The Dataverse resource URL (without path) for token scoping
        let dv_url = env.dataverse_url.as_deref().unwrap_or(&api_url);
        let dv_resource = dv_url.trim_end_matches('/');
        let scope = format!("{dv_resource}/.default");

        let dv_token = match auth::get_scoped_token(&scope).await {
            Ok(t) => t,
            Err(e) => {
                log::warn!("[studio] failed to get Dataverse token for env '{}': {e}", env.display_name.as_deref().unwrap_or(&env.name));
                continue;
            }
        };

        match list_bots(&api_url, &dv_token).await {
            Ok(mut bots) => {
                let env_display = env.display_name.clone().unwrap_or_else(|| env.name.clone());
                log::info!("[studio] env '{}': {} bots", env_display, bots.len());
                for b in &mut bots {
                    b.environment_id = Some(env.id.clone());
                    b.environment_name = Some(env_display.clone());
                }
                all_bots.extend(bots);
            }
            Err(e) => {
                log::warn!("[studio] failed to list bots in env '{}': {e}", env.display_name.as_deref().unwrap_or(&env.name));
            }
        }
    }

    Ok(all_bots)
}
