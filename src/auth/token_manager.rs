//! Token storage, refresh, and validation management

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use crate::auth::models::OAuthTokens;
use crate::auth::oauth_client::OAuthClient;
use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;

/// Manager for OAuth token lifecycle
pub struct TokenManager<'a, R: ConfigRepository> {
    repo: &'a R,
    oauth_client: OAuthClient,
}

impl<'a, R: ConfigRepository> TokenManager<'a, R> {
    /// Create a new token manager
    pub fn new(repo: &'a R) -> Self {
        Self {
            repo,
            oauth_client: OAuthClient::new(),
        }
    }

    /// Load tokens from the configuration database
    pub fn load_tokens(&self) -> Result<Option<OAuthTokens>> {
        let access_token = match config_service::fetch_by_key(self.repo, &ConfigKeys::CodexAccessToken.to_key()) {
            Ok(config) if !config.value.is_empty() => config.value,
            _ => return Ok(None),
        };

        let refresh_token = config_service::fetch_by_key(self.repo, &ConfigKeys::CodexRefreshToken.to_key())
            .ok()
            .and_then(|c| if c.value.is_empty() { None } else { Some(c.value) });

        let expires_at = config_service::fetch_by_key(self.repo, &ConfigKeys::CodexTokenExpiry.to_key())
            .ok()
            .and_then(|c| DateTime::parse_from_rfc3339(&c.value).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let id_token = config_service::fetch_by_key(self.repo, &ConfigKeys::CodexIdToken.to_key())
            .ok()
            .and_then(|c| if c.value.is_empty() { None } else { Some(c.value) });

        Ok(Some(OAuthTokens {
            access_token,
            refresh_token,
            expires_at,
            id_token,
            token_type: "Bearer".to_string(),
        }))
    }

    /// Save tokens to the configuration database
    pub fn save_tokens(&self, tokens: &OAuthTokens) -> Result<()> {
        config_service::write_config(
            self.repo,
            &ConfigKeys::CodexAccessToken.to_key(),
            &tokens.access_token,
        )?;

        if let Some(ref refresh_token) = tokens.refresh_token {
            config_service::write_config(
                self.repo,
                &ConfigKeys::CodexRefreshToken.to_key(),
                refresh_token,
            )?;
        }

        config_service::write_config(
            self.repo,
            &ConfigKeys::CodexTokenExpiry.to_key(),
            &tokens.expires_at.to_rfc3339(),
        )?;

        if let Some(ref id_token) = tokens.id_token {
            config_service::write_config(
                self.repo,
                &ConfigKeys::CodexIdToken.to_key(),
                id_token,
            )?;
        }

        Ok(())
    }

    /// Clear all stored tokens (logout)
    pub fn clear_tokens(&self) -> Result<()> {
        let _ = config_service::write_config(self.repo, &ConfigKeys::CodexAccessToken.to_key(), "");
        let _ = config_service::write_config(self.repo, &ConfigKeys::CodexRefreshToken.to_key(), "");
        let _ = config_service::write_config(self.repo, &ConfigKeys::CodexTokenExpiry.to_key(), "");
        let _ = config_service::write_config(self.repo, &ConfigKeys::CodexIdToken.to_key(), "");
        Ok(())
    }

    /// Get a valid access token, auto-refreshing if necessary
    ///
    /// Returns None if not authenticated.
    /// Returns an error if refresh fails.
    pub async fn get_valid_token(&self) -> Result<Option<String>> {
        let tokens = match self.load_tokens()? {
            Some(t) => t,
            None => return Ok(None),
        };

        // If token is still valid, return it
        if !tokens.is_expired() {
            return Ok(Some(tokens.access_token));
        }

        // Try to refresh if we have a refresh token
        if let Some(ref refresh_token) = tokens.refresh_token {
            let new_tokens = self.oauth_client.refresh_token(refresh_token).await?;
            self.save_tokens(&new_tokens)?;
            return Ok(Some(new_tokens.access_token));
        }

        // Token expired and no refresh token available
        Err(anyhow!("Access token expired and no refresh token available. Please run 'termai config login-codex' to re-authenticate."))
    }

    /// Check if the user is authenticated with valid tokens
    pub fn is_authenticated(&self) -> bool {
        match self.load_tokens() {
            Ok(Some(tokens)) => !tokens.access_token.is_empty(),
            _ => false,
        }
    }

    /// Get the expiration time of the current token
    pub fn token_expires_at(&self) -> Option<DateTime<Utc>> {
        self.load_tokens().ok().flatten().map(|t| t.expires_at)
    }

    /// Get authentication status information
    pub fn auth_status(&self) -> AuthStatus {
        match self.load_tokens() {
            Ok(Some(tokens)) => {
                if tokens.access_token.is_empty() {
                    AuthStatus::NotAuthenticated
                } else if tokens.is_expired() {
                    if tokens.can_refresh() {
                        AuthStatus::Expired { can_refresh: true }
                    } else {
                        AuthStatus::Expired { can_refresh: false }
                    }
                } else {
                    AuthStatus::Authenticated {
                        expires_at: tokens.expires_at,
                    }
                }
            }
            _ => AuthStatus::NotAuthenticated,
        }
    }
}

/// Authentication status
#[derive(Debug)]
pub enum AuthStatus {
    /// Not authenticated - no tokens stored
    NotAuthenticated,
    /// Authenticated with valid tokens
    Authenticated { expires_at: DateTime<Utc> },
    /// Token expired
    Expired { can_refresh: bool },
}

impl std::fmt::Display for AuthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthStatus::NotAuthenticated => write!(f, "Not authenticated"),
            AuthStatus::Authenticated { expires_at } => {
                write!(f, "Authenticated (expires {})", expires_at.format("%Y-%m-%d %H:%M:%S UTC"))
            }
            AuthStatus::Expired { can_refresh } => {
                if *can_refresh {
                    write!(f, "Token expired (can be refreshed)")
                } else {
                    write!(f, "Token expired (re-authentication required)")
                }
            }
        }
    }
}
