use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Duration, Utc};
use keyring::Entry;
use rand::Rng;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::process::Command;
use url::Url;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const AUTHORITY_BASE: &str = "https://login.microsoftonline.com";
const DEFAULT_TENANT: &str = "organizations";
const DEFAULT_CLIENT_ID: &str = "04b07795-a71b-4346-a5c6-e4b5f6a3b052";
const DEFAULT_SCOPE: &str =
    "https://cognitiveservices.azure.com/.default offline_access openid profile";

const KEYRING_SERVICE: &str = "com.orqentis.app";
const KEYRING_ACCOUNT: &str = "foundry-access-token";

const ENV_TENANT_ID: &str = "FOUNDRY_BAR_TENANT_ID";
const ENV_CLIENT_ID: &str = "FOUNDRY_BAR_CLIENT_ID";

const PKCE_VERIFIER_LEN: usize = 64;
const PKCE_CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._~";

const AZ_CLI_PATHS: &[&str] = &[
    "/opt/homebrew/bin/az",
    "/usr/local/bin/az",
    "/usr/bin/az",
];

const LOCALHOST_TIMEOUT_SECS: u64 = 300;

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing configuration: {0}")]
    MissingConfiguration(String),

    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),

    #[error("server error: {0}")]
    Server(String),

    #[error("invalid response: {0}")]
    InvalidResponse(String),

    #[error("user cancelled authentication")]
    UserCancelled,

    #[error("unable to start auth session: {0}")]
    UnableToStartAuthSession(String),

    #[error("missing authorization code")]
    MissingAuthorizationCode,

    #[error("state mismatch (possible CSRF)")]
    StateMismatch,

    #[error("token expired and no refresh token available")]
    NoRefreshToken,

    #[error("keychain error: {0}")]
    Keychain(String),

    #[error("Azure CLI not found")]
    CliNotFound,

    #[error("Azure CLI command failed: {0}")]
    CliCommandFailed(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("timeout waiting for authorization callback")]
    Timeout,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AuthError>;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredAuthToken {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    #[serde(rename = "tenantID")]
    pub tenant_id: String,
    #[serde(rename = "clientID")]
    pub client_id: String,
}

impl StoredAuthToken {
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    pub fn is_near_expiry(&self) -> bool {
        Utc::now() + Duration::minutes(5) >= self.expires_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub tenant_id: String,
    pub client_id: String,
}

/// Raw OAuth token endpoint response.
#[derive(Debug, Deserialize)]
struct OAuthTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    #[allow(dead_code)]
    token_type: Option<String>,
}

/// Azure CLI `az account get-access-token` JSON output.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AzCliTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    expires_on: String,
    #[allow(dead_code)]
    tenant: Option<String>,
}

// ---------------------------------------------------------------------------
// PKCE helpers
// ---------------------------------------------------------------------------

fn generate_pkce_verifier() -> String {
    let mut rng = rand::thread_rng();
    (0..PKCE_VERIFIER_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..PKCE_CHARSET.len());
            PKCE_CHARSET[idx] as char
        })
        .collect()
}

fn pkce_challenge(verifier: &str) -> String {
    let hash = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(hash)
}

fn generate_random_state() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

// ---------------------------------------------------------------------------
// Configuration resolution
// ---------------------------------------------------------------------------

/// Resolve OAuth configuration using priority: env vars → passed config → defaults.
pub fn get_config(stored: Option<&AuthConfig>) -> AuthConfig {
    let tenant_id = std::env::var(ENV_TENANT_ID)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| stored.map(|c| c.tenant_id.clone()).filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| DEFAULT_TENANT.to_string());

    let client_id = std::env::var(ENV_CLIENT_ID)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| stored.map(|c| c.client_id.clone()).filter(|s| !s.trim().is_empty()))
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string());

    AuthConfig {
        tenant_id: tenant_id.trim().to_string(),
        client_id: client_id.trim().to_string(),
    }
}

fn token_endpoint(tenant_id: &str) -> String {
    format!("{}/{}/oauth2/v2.0/token", AUTHORITY_BASE, tenant_id)
}

fn authorize_endpoint(tenant_id: &str) -> String {
    format!("{}/{}/oauth2/v2.0/authorize", AUTHORITY_BASE, tenant_id)
}

// ---------------------------------------------------------------------------
// Keyring storage
// ---------------------------------------------------------------------------

fn keyring_entry() -> std::result::Result<Entry, AuthError> {
    Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT).map_err(|e| AuthError::Keychain(e.to_string()))
}

pub fn get_stored_token() -> Result<Option<StoredAuthToken>> {
    let entry = keyring_entry()?;
    match entry.get_password() {
        Ok(json) => {
            let token: StoredAuthToken = serde_json::from_str(&json)?;
            Ok(Some(token))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AuthError::Keychain(e.to_string())),
    }
}

pub fn save_token(token: &StoredAuthToken) -> Result<()> {
    let entry = keyring_entry()?;
    let json = serde_json::to_string(token)?;
    entry
        .set_password(&json)
        .map_err(|e| AuthError::Keychain(e.to_string()))
}

pub fn sign_out() -> Result<()> {
    let entry = keyring_entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AuthError::Keychain(e.to_string())),
    }
}

// ---------------------------------------------------------------------------
// OAuth PKCE sign-in flow
// ---------------------------------------------------------------------------

/// Full interactive sign-in: opens browser, captures code via localhost redirect,
/// exchanges for tokens, stores in keychain, and returns the token.
pub async fn sign_in(scope: &str) -> Result<StoredAuthToken> {
    let config = get_config(None);
    sign_in_with_config(&config, scope).await
}

pub async fn sign_in_with_config(config: &AuthConfig, scope: &str) -> Result<StoredAuthToken> {
    let verifier = generate_pkce_verifier();
    let challenge = pkce_challenge(&verifier);
    let state = generate_random_state();

    // Bind an ephemeral port for the redirect listener.
    let listener = TcpListener::bind("127.0.0.1:0")
        .map_err(|e| AuthError::UnableToStartAuthSession(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| AuthError::UnableToStartAuthSession(e.to_string()))?
        .port();
    let redirect_uri = format!("http://localhost:{}", port);

    // Build authorization URL.
    let mut auth_url = Url::parse(&authorize_endpoint(&config.tenant_id))
        .map_err(|e| AuthError::InvalidEndpoint(e.to_string()))?;
    auth_url.query_pairs_mut()
        .append_pair("client_id", &config.client_id)
        .append_pair("response_type", "code")
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_mode", "query")
        .append_pair("scope", scope)
        .append_pair("state", &state)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256");

    // Open system browser.
    open_browser(auth_url.as_str())?;

    // Wait for the redirect callback on a blocking thread to avoid tying up
    // the async runtime.
    let expected_state = state.clone();
    let auth_code = tokio::task::spawn_blocking(move || {
        wait_for_auth_code(listener, &expected_state)
    })
    .await
    .map_err(|e| AuthError::Server(e.to_string()))??;

    // Exchange authorization code for tokens.
    let token = exchange_code(config, &auth_code, &verifier, &redirect_uri, scope).await?;
    save_token(&token)?;
    Ok(token)
}

/// Block on the TcpListener until we receive the redirect with an auth code.
fn wait_for_auth_code(listener: TcpListener, expected_state: &str) -> Result<String> {
    listener
        .set_nonblocking(false)
        .map_err(|e| AuthError::UnableToStartAuthSession(e.to_string()))?;

    // Use SO_RCVTIMEO for a hard timeout.
    let timeout = std::time::Duration::from_secs(LOCALHOST_TIMEOUT_SECS);
    listener
        .set_nonblocking(false)
        .ok();

    let deadline = std::time::Instant::now() + timeout;

    // Accept one connection.
    loop {
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() {
            return Err(AuthError::Timeout);
        }

        // We can't set accept timeout on TcpListener directly, so set a
        // short non-blocking poll interval.
        listener.set_nonblocking(true).ok();
        match listener.accept() {
            Ok((mut stream, _)) => {
                stream.set_nonblocking(false).ok();
                let mut reader = BufReader::new(&stream);
                let mut request_line = String::new();
                reader.read_line(&mut request_line).ok();

                // Parse GET /?code=...&state=... HTTP/1.1
                let (code, returned_state, error) = parse_callback_params(&request_line);

                // Send response HTML.
                let (status, body) = if error.is_some() || code.is_none() {
                    ("400 Bad Request", error_html(error.as_deref().unwrap_or("Unknown error")))
                } else if returned_state.as_deref() != Some(expected_state) {
                    ("400 Bad Request", error_html("State mismatch – possible CSRF attack"))
                } else {
                    ("200 OK", success_html())
                };

                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n{}",
                    status, body
                );
                stream.write_all(response.as_bytes()).ok();
                stream.flush().ok();

                if let Some(err) = error {
                    return Err(AuthError::Server(err));
                }
                let code = code.ok_or(AuthError::MissingAuthorizationCode)?;
                if returned_state.as_deref() != Some(expected_state) {
                    return Err(AuthError::StateMismatch);
                }
                return Ok(code);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            Err(e) => return Err(AuthError::Io(e)),
        }
    }
}

fn parse_callback_params(request_line: &str) -> (Option<String>, Option<String>, Option<String>) {
    // Expected: "GET /?code=...&state=... HTTP/1.1\r\n"
    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or("/");

    let fake_base = format!("http://localhost{}", path);
    let parsed = match Url::parse(&fake_base) {
        Ok(u) => u,
        Err(_) => return (None, None, Some("Failed to parse callback URL".into())),
    };

    let mut code = None;
    let mut state = None;
    let mut error = None;

    for (key, value) in parsed.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.to_string()),
            "state" => state = Some(value.to_string()),
            "error" => error = Some(value.to_string()),
            "error_description" => {
                if error.is_none() {
                    error = Some(value.to_string());
                } else {
                    error = Some(format!("{}: {}", error.unwrap(), value));
                }
            }
            _ => {}
        }
    }

    (code, state, error)
}

fn success_html() -> String {
    r#"<!DOCTYPE html><html><body style="font-family:system-ui;text-align:center;padding:60px">
<h1>&#10003; Signed in successfully</h1>
<p>You can close this window and return to the app.</p>
</body></html>"#
        .to_string()
}

fn error_html(msg: &str) -> String {
    format!(
        r#"<!DOCTYPE html><html><body style="font-family:system-ui;text-align:center;padding:60px">
<h1>&#10007; Authentication failed</h1>
<p>{}</p>
</body></html>"#,
        msg
    )
}

fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| AuthError::UnableToStartAuthSession(e.to_string()))?;
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .map_err(|e| AuthError::UnableToStartAuthSession(e.to_string()))?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| AuthError::UnableToStartAuthSession(e.to_string()))?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Token exchange & refresh
// ---------------------------------------------------------------------------

async fn exchange_code(
    config: &AuthConfig,
    code: &str,
    verifier: &str,
    redirect_uri: &str,
    scope: &str,
) -> Result<StoredAuthToken> {
    let mut params = HashMap::new();
    params.insert("grant_type", "authorization_code");
    params.insert("client_id", &config.client_id);
    params.insert("code", code);
    params.insert("code_verifier", verifier);
    params.insert("redirect_uri", redirect_uri);
    params.insert("scope", scope);

    let client = Client::new();
    let resp = client
        .post(&token_endpoint(&config.tenant_id))
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AuthError::Server(body));
    }

    let token_resp: OAuthTokenResponse = resp.json().await?;
    Ok(StoredAuthToken {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token,
        expires_at: Utc::now() + Duration::seconds(token_resp.expires_in),
        tenant_id: config.tenant_id.clone(),
        client_id: config.client_id.clone(),
    })
}

/// Refresh an access token using the stored refresh token.
async fn refresh_token(
    config: &AuthConfig,
    refresh_token_value: &str,
    scope: &str,
) -> Result<StoredAuthToken> {
    let mut params = HashMap::new();
    params.insert("grant_type", "refresh_token");
    params.insert("client_id", &config.client_id);
    params.insert("refresh_token", refresh_token_value);
    params.insert("scope", scope);

    let client = Client::new();
    let resp = client
        .post(&token_endpoint(&config.tenant_id))
        .form(&params)
        .send()
        .await?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(AuthError::Server(body));
    }

    let token_resp: OAuthTokenResponse = resp.json().await?;
    Ok(StoredAuthToken {
        access_token: token_resp.access_token,
        refresh_token: token_resp.refresh_token.or(Some(refresh_token_value.to_string())),
        expires_at: Utc::now() + Duration::seconds(token_resp.expires_in),
        tenant_id: config.tenant_id.clone(),
        client_id: config.client_id.clone(),
    })
}

// ---------------------------------------------------------------------------
// Public token access
// ---------------------------------------------------------------------------

/// Return a valid access token, refreshing silently if near expiry.
pub async fn get_token() -> Result<String> {
    let stored = get_stored_token()?.ok_or(AuthError::NoRefreshToken)?;

    if !stored.is_near_expiry() {
        return Ok(stored.access_token);
    }

    let rt = stored
        .refresh_token
        .as_deref()
        .ok_or(AuthError::NoRefreshToken)?;

    let config = AuthConfig {
        tenant_id: stored.tenant_id.clone(),
        client_id: stored.client_id.clone(),
    };

    let refreshed = refresh_token(&config, rt, DEFAULT_SCOPE).await?;
    save_token(&refreshed)?;
    Ok(refreshed.access_token)
}

/// Exchange the stored refresh token for an access token with a different scope
/// (e.g. ARM `https://management.azure.com/.default offline_access` or Graph).
pub async fn get_scoped_token(scope: &str) -> Result<String> {
    let stored = get_stored_token()?.ok_or(AuthError::NoRefreshToken)?;

    let rt = stored
        .refresh_token
        .as_deref()
        .ok_or(AuthError::NoRefreshToken)?;

    let config = AuthConfig {
        tenant_id: stored.tenant_id.clone(),
        client_id: stored.client_id.clone(),
    };

    let scoped = refresh_token(&config, rt, scope).await?;
    Ok(scoped.access_token)
}

// ---------------------------------------------------------------------------
// Azure CLI fallback
// ---------------------------------------------------------------------------

/// Attempt to get an access token via the Azure CLI.
pub async fn try_az_cli_token(resource: &str) -> Result<String> {
    let az_path = AZ_CLI_PATHS
        .iter()
        .find(|p| std::path::Path::new(p).exists())
        .ok_or(AuthError::CliNotFound)?;

    let output = tokio::task::spawn_blocking({
        let az_path = az_path.to_string();
        let resource = resource.to_string();
        move || {
            Command::new(az_path)
                .args(["account", "get-access-token", "--resource", &resource, "--output", "json"])
                .output()
        }
    })
    .await
    .map_err(|e| AuthError::CliCommandFailed(e.to_string()))?
    .map_err(|e| AuthError::CliCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AuthError::CliCommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cli_resp: AzCliTokenResponse =
        serde_json::from_str(&stdout).map_err(|e| AuthError::InvalidResponse(e.to_string()))?;

    Ok(cli_resp.access_token)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_verifier_length_and_charset() {
        let v = generate_pkce_verifier();
        assert_eq!(v.len(), PKCE_VERIFIER_LEN);
        assert!(v.chars().all(|c| PKCE_CHARSET.contains(&(c as u8))));
    }

    #[test]
    fn pkce_challenge_is_base64url_sha256() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = pkce_challenge(verifier);
        // SHA-256 of the above verifier, base64url-encoded without padding
        let expected_hash = Sha256::digest(verifier.as_bytes());
        let expected = URL_SAFE_NO_PAD.encode(expected_hash);
        assert_eq!(challenge, expected);
        assert!(!challenge.contains('+'));
        assert!(!challenge.contains('/'));
        assert!(!challenge.contains('='));
    }

    #[test]
    fn random_state_is_nonempty_base64url() {
        let s = generate_random_state();
        assert!(!s.is_empty());
        assert!(URL_SAFE_NO_PAD.decode(&s).is_ok());
    }

    #[test]
    fn config_uses_defaults_when_env_unset() {
        // Remove env vars to ensure defaults.
        std::env::remove_var(ENV_TENANT_ID);
        std::env::remove_var(ENV_CLIENT_ID);

        let config = get_config(None);
        assert_eq!(config.tenant_id, DEFAULT_TENANT);
        assert_eq!(config.client_id, DEFAULT_CLIENT_ID);
    }

    #[test]
    fn config_prefers_env_over_stored() {
        std::env::set_var(ENV_TENANT_ID, "env-tenant");
        std::env::set_var(ENV_CLIENT_ID, "env-client");

        let stored = AuthConfig {
            tenant_id: "stored-tenant".into(),
            client_id: "stored-client".into(),
        };
        let config = get_config(Some(&stored));
        assert_eq!(config.tenant_id, "env-tenant");
        assert_eq!(config.client_id, "env-client");

        std::env::remove_var(ENV_TENANT_ID);
        std::env::remove_var(ENV_CLIENT_ID);
    }

    #[test]
    fn config_falls_back_to_stored() {
        std::env::remove_var(ENV_TENANT_ID);
        std::env::remove_var(ENV_CLIENT_ID);

        let stored = AuthConfig {
            tenant_id: "my-tenant".into(),
            client_id: "my-client".into(),
        };
        let config = get_config(Some(&stored));
        assert_eq!(config.tenant_id, "my-tenant");
        assert_eq!(config.client_id, "my-client");
    }

    #[test]
    fn stored_auth_token_roundtrip_json() {
        let token = StoredAuthToken {
            access_token: "at".into(),
            refresh_token: Some("rt".into()),
            expires_at: Utc::now() + Duration::hours(1),
            tenant_id: "organizations".into(),
            client_id: DEFAULT_CLIENT_ID.into(),
        };

        let json = serde_json::to_string(&token).unwrap();
        assert!(json.contains("\"accessToken\""));
        assert!(json.contains("\"refreshToken\""));
        assert!(json.contains("\"tenantID\""));
        assert!(json.contains("\"clientID\""));

        let decoded: StoredAuthToken = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.access_token, "at");
        assert_eq!(decoded.tenant_id, "organizations");
    }

    #[test]
    fn token_expiry_helpers() {
        let future = StoredAuthToken {
            access_token: "a".into(),
            refresh_token: None,
            expires_at: Utc::now() + Duration::hours(1),
            tenant_id: "t".into(),
            client_id: "c".into(),
        };
        assert!(!future.is_expired());
        assert!(!future.is_near_expiry());

        let past = StoredAuthToken {
            access_token: "a".into(),
            refresh_token: None,
            expires_at: Utc::now() - Duration::hours(1),
            tenant_id: "t".into(),
            client_id: "c".into(),
        };
        assert!(past.is_expired());
        assert!(past.is_near_expiry());
    }

    #[test]
    fn parse_callback_extracts_code_and_state() {
        let line = "GET /?code=MYCODE&state=MYSTATE HTTP/1.1\r\n";
        let (code, state, error) = parse_callback_params(line);
        assert_eq!(code.as_deref(), Some("MYCODE"));
        assert_eq!(state.as_deref(), Some("MYSTATE"));
        assert!(error.is_none());
    }

    #[test]
    fn parse_callback_extracts_error() {
        let line = "GET /?error=access_denied&error_description=User+cancelled HTTP/1.1\r\n";
        let (code, _state, error) = parse_callback_params(line);
        assert!(code.is_none());
        assert!(error.is_some());
    }

    #[test]
    fn token_endpoint_format() {
        let ep = token_endpoint("my-tenant");
        assert_eq!(
            ep,
            "https://login.microsoftonline.com/my-tenant/oauth2/v2.0/token"
        );
    }

    #[test]
    fn authorize_endpoint_format() {
        let ep = authorize_endpoint("organizations");
        assert_eq!(
            ep,
            "https://login.microsoftonline.com/organizations/oauth2/v2.0/authorize"
        );
    }
}
