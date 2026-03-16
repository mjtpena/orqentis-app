use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A declarative agent or M365 Copilot extension discovered via Microsoft Graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M365Agent {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub app_id: Option<String>,
    #[serde(default)]
    pub status: String,
}

// ---------------------------------------------------------------------------
// Internal response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct GraphListResponse {
    value: Vec<serde_json::Value>,
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

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List M365 Copilot agents — declarative agents from the Teams app catalog
/// and Graph connectors that extend Microsoft 365 Copilot.
pub async fn list_m365_agents(graph_token: &str) -> Result<Vec<M365Agent>, String> {
    let mut agents = Vec::new();

    // 1. List org-published Teams apps (includes declarative agents / message extensions)
    //    Requires AppCatalog.Read.All — may fail with Azure CLI tokens
    match list_teams_declarative_agents(graph_token).await {
        Ok(teams_agents) => agents.extend(teams_agents),
        Err(e) => log::warn!("[m365] Teams app catalog unavailable (likely missing AppCatalog.Read.All scope): {e}"),
    }

    // 2. List Graph connectors (these extend M365 Copilot with external data)
    let connectors = list_graph_connectors(graph_token).await.unwrap_or_default();
    agents.extend(connectors);

    Ok(agents)
}

/// List Teams apps published to the organization that act as declarative agents,
/// message extensions, or Copilot plugins.
async fn list_teams_declarative_agents(graph_token: &str) -> Result<Vec<M365Agent>, String> {
    // Get all Teams apps published within the org
    let url = "https://graph.microsoft.com/v1.0/appCatalogs/teamsApps?$filter=distributionMethod eq 'organization'&$expand=appDefinitions($select=displayName,shortDescription,description,bot)";

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

    let body: GraphListResponse = resp.json().await.map_err(|e| format!("Graph parse: {e}"))?;
    let mut agents = Vec::new();

    for app in &body.value {
        let id = app.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let external_id = app.get("externalId").and_then(|v| v.as_str());

        let defs = app
            .get("appDefinitions")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        if let Some(def) = defs.first() {
            let name = def
                .get("displayName")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown App");
            let desc = def
                .get("shortDescription")
                .and_then(|v| v.as_str())
                .or_else(|| def.get("description").and_then(|v| v.as_str()));

            agents.push(M365Agent {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.map(String::from),
                app_id: external_id.map(String::from),
                status: "published".to_string(),
            });
        }
    }

    Ok(agents)
}

/// List Microsoft Graph connectors — external connections that feed data into M365 Copilot.
async fn list_graph_connectors(graph_token: &str) -> Result<Vec<M365Agent>, String> {
    let url = "https://graph.microsoft.com/v1.0/external/connections?$select=id,name,description,state";

    let resp = reqwest::Client::new()
        .get(url)
        .headers(build_headers(graph_token))
        .send()
        .await
        .map_err(|e| format!("Graph connectors request failed: {e}"))?;

    if !resp.status().is_success() {
        // Graph connectors API may not be available — that's fine
        return Ok(Vec::new());
    }

    let body: GraphListResponse = resp.json().await.map_err(|e| format!("Graph parse: {e}"))?;
    let mut agents = Vec::new();

    for conn in &body.value {
        let id = conn.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let name = conn.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown Connector");
        let desc = conn.get("description").and_then(|v| v.as_str());
        let state = conn
            .get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        agents.push(M365Agent {
            id: format!("connector-{id}"),
            name: name.to_string(),
            description: desc.map(String::from),
            app_id: Some(id.to_string()),
            status: if state == "ready" {
                "active".to_string()
            } else {
                state.to_string()
            },
        });
    }

    Ok(agents)
}
