//! OAuth data structures and types

#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// OpenAI Codex OAuth configuration
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub auth_endpoint: String,
    pub token_endpoint: String,
    pub callback_url: String,
    pub scopes: Vec<String>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            client_id: "app_EMoamEEZ73f0CkXaXp7hrann".to_string(),
            auth_endpoint: "https://auth.openai.com/oauth/authorize".to_string(),
            token_endpoint: "https://auth.openai.com/oauth/token".to_string(),
            callback_url: "http://localhost:1455/auth/callback".to_string(),
            scopes: vec![
                "openid".to_string(),
                "profile".to_string(),
                "email".to_string(),
                "offline_access".to_string(),
            ],
        }
    }
}

impl OAuthConfig {
    /// Get the scopes as a space-separated string
    pub fn scopes_string(&self) -> String {
        self.scopes.join(" ")
    }
}

/// OAuth tokens returned from the token endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub id_token: Option<String>,
    pub token_type: String,
}

impl OAuthTokens {
    /// Check if the access token is expired or will expire soon (within 5 minutes)
    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        let buffer = chrono::Duration::minutes(5);
        self.expires_at <= now + buffer
    }

    /// Check if we have a refresh token available
    pub fn can_refresh(&self) -> bool {
        self.refresh_token.is_some()
    }
}

/// Token response from the OAuth token endpoint
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub scope: Option<String>,
}

impl TokenResponse {
    /// Convert to OAuthTokens with calculated expiration time
    pub fn into_tokens(self) -> OAuthTokens {
        let expires_at = Utc::now() + chrono::Duration::seconds(self.expires_in);
        OAuthTokens {
            access_token: self.access_token,
            refresh_token: self.refresh_token,
            expires_at,
            id_token: self.id_token,
            token_type: self.token_type,
        }
    }
}

/// Authentication method - either API key or OAuth tokens
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Traditional API key authentication
    ApiKey(String),
    /// OAuth access token authentication
    OAuth(OAuthTokens),
}

impl AuthMethod {
    /// Get the authorization header value
    pub fn auth_header(&self) -> String {
        match self {
            AuthMethod::ApiKey(key) => format!("Bearer {}", key),
            AuthMethod::OAuth(tokens) => format!("Bearer {}", tokens.access_token),
        }
    }

    /// Check if authentication is valid (non-empty and not expired for OAuth)
    pub fn is_valid(&self) -> bool {
        match self {
            AuthMethod::ApiKey(key) => !key.is_empty(),
            AuthMethod::OAuth(tokens) => !tokens.is_expired(),
        }
    }
}

/// OAuth state for CSRF protection
#[derive(Debug, Clone)]
pub struct OAuthState {
    pub state: String,
    pub code_verifier: String,
}

/// Error types for OAuth operations
#[derive(Debug)]
pub enum OAuthError {
    /// PKCE generation failed
    PkceError(String),
    /// Callback server error
    CallbackError(String),
    /// Token exchange failed
    TokenExchangeError(String),
    /// Token refresh failed
    RefreshError(String),
    /// State mismatch (potential CSRF)
    StateMismatch,
    /// User cancelled the flow
    UserCancelled,
    /// Network error
    NetworkError(String),
    /// Storage error
    StorageError(String),
}

impl std::fmt::Display for OAuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OAuthError::PkceError(msg) => write!(f, "PKCE error: {}", msg),
            OAuthError::CallbackError(msg) => write!(f, "Callback error: {}", msg),
            OAuthError::TokenExchangeError(msg) => write!(f, "Token exchange error: {}", msg),
            OAuthError::RefreshError(msg) => write!(f, "Token refresh error: {}", msg),
            OAuthError::StateMismatch => write!(f, "State mismatch - potential CSRF attack"),
            OAuthError::UserCancelled => write!(f, "User cancelled authentication"),
            OAuthError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            OAuthError::StorageError(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for OAuthError {}
