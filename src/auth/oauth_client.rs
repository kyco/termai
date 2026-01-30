//! OAuth client for OpenAI Codex authentication

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::time::Duration;

use crate::auth::callback_server::CallbackServer;
use crate::auth::models::{OAuthConfig, OAuthTokens, TokenResponse};
use crate::auth::pkce::{generate_code_challenge, generate_code_verifier, generate_state};

/// OAuth client for handling the PKCE authorization flow
pub struct OAuthClient {
    config: OAuthConfig,
    http_client: reqwest::Client,
}

impl OAuthClient {
    /// Create a new OAuth client with default configuration
    pub fn new() -> Self {
        Self {
            config: OAuthConfig::default(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Create a new OAuth client with custom configuration
    pub fn with_config(config: OAuthConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Build the authorization URL with PKCE parameters
    fn build_auth_url(&self, state: &str, code_challenge: &str) -> String {
        format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&code_challenge={}&code_challenge_method=S256&id_token_add_organizations=true&codex_cli_simplified_flow=true",
            self.config.auth_endpoint,
            urlencoding::encode(&self.config.client_id),
            urlencoding::encode(&self.config.callback_url),
            urlencoding::encode(&self.config.scopes_string()),
            urlencoding::encode(state),
            urlencoding::encode(code_challenge),
        )
    }

    /// Start the OAuth authorization flow
    ///
    /// Opens the browser for user authentication and waits for the callback.
    /// If the browser cannot be opened (e.g., on a headless server), displays
    /// the URL for the user to copy and paste manually.
    pub async fn authorize(&self) -> Result<OAuthTokens> {
        // Generate PKCE parameters
        let code_verifier = generate_code_verifier();
        let code_challenge = generate_code_challenge(&code_verifier);
        let state = generate_state();

        // Build authorization URL
        let auth_url = self.build_auth_url(&state, &code_challenge);

        // Always display the URL for users who need to copy it
        println!();
        println!("If the browser doesn't open, copy this URL:");
        println!();
        println!("  {}", auth_url);
        println!();

        // Try to open browser, but don't fail if it can't open
        if webbrowser::open(&auth_url).is_err() {
            println!("Could not open browser automatically.");
            println!("Please open the URL above in your browser.");
            println!();
        }

        println!("Waiting for authentication...");

        // Wait for callback
        let callback_port = 1455;
        let server = CallbackServer::new(callback_port);
        let callback = server.wait_for_callback(Duration::from_secs(300))?;

        // Verify state to prevent CSRF
        if callback.state != state {
            return Err(anyhow!("State mismatch - potential CSRF attack"));
        }

        // Exchange code for tokens
        self.exchange_code(&callback.code, &code_verifier).await
    }

    /// Exchange authorization code for tokens
    async fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<OAuthTokens> {
        let params = [
            ("grant_type", "authorization_code"),
            ("client_id", &self.config.client_id),
            ("code", code),
            ("redirect_uri", &self.config.callback_url),
            ("code_verifier", code_verifier),
        ];

        let response = self
            .http_client
            .post(&self.config.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to exchange code: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Token exchange failed with status {}: {}",
                status,
                body
            ));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse token response: {}", e))?;

        Ok(token_response.into_tokens())
    }

    /// Refresh an expired access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<OAuthTokens> {
        let params = [
            ("grant_type", "refresh_token"),
            ("client_id", &self.config.client_id),
            ("refresh_token", refresh_token),
        ];

        let response = self
            .http_client
            .post(&self.config.token_endpoint)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&params)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to refresh token: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Token refresh failed with status {}: {}",
                status,
                body
            ));
        }

        let token_response: TokenResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse refresh token response: {}", e))?;

        Ok(token_response.into_tokens())
    }
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

/// URL encoding helper module
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::with_capacity(s.len() * 3);
        for c in s.chars() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(c),
                ' ' => result.push_str("%20"),
                _ => {
                    for byte in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }
}
