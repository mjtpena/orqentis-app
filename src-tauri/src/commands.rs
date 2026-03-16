use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tauri::Emitter;

use crate::arm;
use crate::auth;
use crate::foundry;

const ARM_SCOPE: &str = "https://management.azure.com/.default offline_access";

// ---------------------------------------------------------------------------
// Helper types exposed to the frontend
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub signed_in: bool,
    pub user_name: Option<String>,
    pub tenant_id: Option<String>,
    pub auth_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfigResponse {
    pub tenant_id: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRunResult {
    pub thread_id: String,
    pub run_id: String,
    pub status: String,
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatTokenPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    done: Option<bool>,
}

/// Decode JWT payload (base64url, no validation) to extract user info.
fn jwt_user_name(token: &str) -> Option<String> {
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .or_else(|_| {
            // Some JWTs use standard base64 with padding
            base64::engine::general_purpose::STANDARD.decode(parts[1])
        })
        .ok()?;
    let val: serde_json::Value = serde_json::from_slice(&payload).ok()?;
    val.get("name")
        .or_else(|| val.get("preferred_username"))
        .or_else(|| val.get("upn"))
        .and_then(|v| v.as_str())
        .map(String::from)
}

use base64::Engine;

// ===========================================================================
// Auth commands
// ===========================================================================

#[tauri::command]
pub async fn sign_in() -> Result<AuthStatus, String> {
    let scope = "https://cognitiveservices.azure.com/.default offline_access openid profile";

    // Try Azure CLI first for a non-interactive flow
    if let Ok(cli_token) = auth::try_az_cli_token("https://cognitiveservices.azure.com").await {
        let user_name = jwt_user_name(&cli_token);
        return Ok(AuthStatus {
            signed_in: true,
            user_name,
            tenant_id: None,
            auth_mode: "az_cli".to_string(),
        });
    }

    // Fall back to interactive OAuth PKCE
    let token = auth::sign_in(scope).await.map_err(|e| e.to_string())?;
    let user_name = jwt_user_name(&token.access_token);

    Ok(AuthStatus {
        signed_in: true,
        user_name,
        tenant_id: Some(token.tenant_id),
        auth_mode: "oauth".to_string(),
    })
}

#[tauri::command]
pub async fn sign_out() -> Result<(), String> {
    auth::sign_out().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_auth_status() -> Result<AuthStatus, String> {
    match auth::get_stored_token().map_err(|e| e.to_string())? {
        Some(token) if !token.is_expired() => {
            let user_name = jwt_user_name(&token.access_token);
            Ok(AuthStatus {
                signed_in: true,
                user_name,
                tenant_id: Some(token.tenant_id),
                auth_mode: "oauth".to_string(),
            })
        }
        _ => {
            // Check if Azure CLI is available
            if let Ok(cli_token) =
                auth::try_az_cli_token("https://cognitiveservices.azure.com").await
            {
                let user_name = jwt_user_name(&cli_token);
                return Ok(AuthStatus {
                    signed_in: true,
                    user_name,
                    tenant_id: None,
                    auth_mode: "az_cli".to_string(),
                });
            }

            Ok(AuthStatus {
                signed_in: false,
                user_name: None,
                tenant_id: None,
                auth_mode: "none".to_string(),
            })
        }
    }
}

#[tauri::command]
pub async fn get_auth_config() -> Result<AuthConfigResponse, String> {
    let config = auth::get_config(None);
    Ok(AuthConfigResponse {
        tenant_id: config.tenant_id,
        client_id: config.client_id,
    })
}

// ===========================================================================
// Discovery commands (ARM)
// ===========================================================================

fn arm_token_sync_err(msg: &str) -> String {
    format!("ARM token error: {msg}")
}

async fn get_arm_token() -> Result<String, String> {
    auth::get_scoped_token(ARM_SCOPE)
        .await
        .map_err(|e| arm_token_sync_err(&e.to_string()))
}

#[tauri::command]
pub async fn discover_resources() -> Result<arm::DiscoveryResult, String> {
    let token = get_arm_token().await?;
    arm::discover_all(&token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_subscriptions() -> Result<Vec<arm::Subscription>, String> {
    let token = get_arm_token().await?;
    arm::list_subscriptions(&token)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn list_deployments(ai_services_resource_id: String) -> Result<Vec<arm::ArmDeployment>, String> {
    let token = get_arm_token().await?;
    arm::list_deployments(&token, &ai_services_resource_id)
        .await
        .map_err(|e| e.to_string())
}

// ===========================================================================
// Foundry commands
// ===========================================================================

async fn foundry_token() -> Result<String, String> {
    auth::get_token().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_foundry_deployments(
    endpoint: String,
) -> Result<Vec<foundry::Deployment>, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let list = client
        .list_deployments(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(list.data)
}

#[tauri::command]
pub async fn list_agents(endpoint: String) -> Result<Vec<foundry::Agent>, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let list = client
        .list_agents(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(list.data)
}

#[tauri::command]
pub async fn create_agent(
    endpoint: String,
    name: String,
    model: String,
    instructions: String,
) -> Result<foundry::Agent, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let req = foundry::CreateAgentRequest {
        name,
        model,
        instructions: if instructions.is_empty() {
            None
        } else {
            Some(instructions)
        },
        tools: None,
    };
    client
        .create_agent(&token, &req)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_agent(endpoint: String, agent_id: String) -> Result<(), String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    client
        .delete_agent(&token, &agent_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_files(endpoint: String) -> Result<Vec<foundry::FileObject>, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let list = client
        .list_files(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(list.data)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn delete_file(endpoint: String, file_id: String) -> Result<(), String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    client
        .delete_file(&token, &file_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn list_vector_stores(
    endpoint: String,
) -> Result<Vec<foundry::VectorStore>, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let list = client
        .list_vector_stores(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(list.data)
}

#[tauri::command]
pub async fn list_fine_tuning_jobs(
    endpoint: String,
) -> Result<Vec<foundry::FineTuningJob>, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let list = client
        .list_fine_tuning_jobs(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(list.data)
}

#[tauri::command]
pub async fn list_batch_jobs(endpoint: String) -> Result<Vec<foundry::BatchJob>, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let list = client
        .list_batches(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(list.data)
}

#[tauri::command]
pub async fn list_connections(
    endpoint: String,
) -> Result<Vec<foundry::FoundryConnection>, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let list = client
        .list_connections(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(list.value)
}

#[tauri::command]
pub async fn list_models(endpoint: String) -> Result<Vec<foundry::Model>, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);
    let list = client
        .list_models(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(list.data)
}

// ===========================================================================
// Chat commands
// ===========================================================================

#[tauri::command(rename_all = "camelCase")]
pub async fn send_chat_message(
    app: tauri::AppHandle,
    endpoint: String,
    deployment_name: String,
    messages: Vec<ChatMessage>,
) -> Result<(), String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);

    let foundry_messages: Vec<foundry::ChatMessage> = messages
        .into_iter()
        .map(|m| foundry::ChatMessage {
            role: m.role,
            content: m.content,
        })
        .collect();

    let req = foundry::ChatCompletionRequest {
        messages: foundry_messages,
        stream: true,
        max_tokens: 4096,
        temperature: 0.7,
    };

    let mut stream = client
        .stream_chat(&token, &deployment_name, &req)
        .await
        .map_err(|e| e.to_string())?;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(content) => {
                let _ = app.emit(
                    "chat-token",
                    ChatTokenPayload {
                        content: Some(content),
                        done: None,
                    },
                );
            }
            Err(e) => {
                log::error!("stream error: {e}");
                break;
            }
        }
    }

    let _ = app.emit(
        "chat-token",
        ChatTokenPayload {
            content: None,
            done: Some(true),
        },
    );

    Ok(())
}

#[tauri::command(rename_all = "camelCase")]
pub async fn send_agent_message(
    endpoint: String,
    agent_id: String,
    thread_id: Option<String>,
    message: String,
) -> Result<AgentRunResult, String> {
    let token = foundry_token().await?;
    let client = foundry::FoundryClient::new(&endpoint);

    // Create or reuse thread
    let tid = match thread_id {
        Some(id) if !id.is_empty() => id,
        _ => {
            let thread = client
                .create_thread(&token)
                .await
                .map_err(|e| e.to_string())?;
            thread.id
        }
    };

    // Post user message
    let msg_req = foundry::CreateMessageRequest {
        role: "user".to_string(),
        content: message,
    };
    client
        .create_message(&token, &tid, &msg_req)
        .await
        .map_err(|e| e.to_string())?;

    // Create run
    let run_req = foundry::CreateRunRequest {
        assistant_id: agent_id.clone(),
    };
    let run = client
        .create_run(&token, &tid, &run_req)
        .await
        .map_err(|e| e.to_string())?;

    // Poll until terminal state
    let terminal_states = [
        "completed",
        "failed",
        "cancelled",
        "expired",
        "requires_action",
    ];
    let mut current_run = run;
    let max_polls = 120; // ~2 minutes at 1s intervals
    for _ in 0..max_polls {
        if terminal_states.contains(&current_run.status.as_str()) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        current_run = client
            .get_run(&token, &tid, &current_run.id)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Fetch messages
    let msg_list = client
        .list_messages(&token, &tid)
        .await
        .map_err(|e| e.to_string())?;

    let chat_messages: Vec<ChatMessage> = msg_list
        .data
        .into_iter()
        .map(|m| {
            let text = m
                .content
                .into_iter()
                .map(|c| match c {
                    foundry::MessageContent::Text { text } => text.value,
                })
                .collect::<Vec<_>>()
                .join("\n");
            ChatMessage {
                role: m.role,
                content: text,
            }
        })
        .collect();

    Ok(AgentRunResult {
        thread_id: tid,
        run_id: current_run.id,
        status: current_run.status,
        messages: chat_messages,
    })
}
