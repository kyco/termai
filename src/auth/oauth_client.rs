//! OAuth client for OpenAI Codex authentication

#![allow(dead_code)]

use anyhow::{anyhow, Result};
use std::io::{self, BufRead, BufReader, Write};
use std::time::Duration;

use crate::auth::callback_server::CallbackServer;
use crate::auth::models::{OAuthConfig, OAuthTokens, TokenResponse};
use crate::auth::pkce::{generate_code_challenge, generate_code_verifier, generate_state};

/// OAuth client for handling the PKCE authorization flow
pub struct OAuthClient {
    config: OAuthConfig,
    http_client: reqwest::Client,
}

const AUTOMATIC_CALLBACK_TIMEOUT: Duration = Duration::from_secs(90);

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
        println!("If the browser redirects to localhost but TermAI does not detect it, you can paste the full redirect URL here after the automatic wait.");

        // Wait for callback, then fall back to a pasted redirect URL if needed.
        let callback_port = 1455;
        let server = CallbackServer::new(callback_port);
        let callback = match server.wait_for_callback(AUTOMATIC_CALLBACK_TIMEOUT) {
            Ok(callback) => callback,
            Err(err) => {
                println!();
                println!("Automatic localhost callback was not captured: {}", err);
                println!("Paste the final redirect URL from your browser to continue.");

                let stdin = io::stdin();
                let stdout = io::stdout();
                let mut reader = BufReader::new(stdin.lock());
                let mut writer = stdout.lock();
                Self::prompt_for_manual_callback(&mut reader, &mut writer, &state)?
            }
        };

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

    fn prompt_for_manual_callback<R: BufRead, W: Write>(
        reader: &mut R,
        writer: &mut W,
        expected_state: &str,
    ) -> Result<crate::auth::callback_server::CallbackResult> {
        writeln!(writer)?;
        writeln!(
            writer,
            "Paste the full redirect URL from the browser address bar."
        )?;
        writeln!(
            writer,
            "Example: http://localhost:1455/auth/callback?code=...&state=..."
        )?;

        loop {
            write!(writer, "Redirect URL: ")?;
            writer.flush()?;

            let mut input = String::new();
            let bytes_read = reader.read_line(&mut input)?;
            if bytes_read == 0 {
                return Err(anyhow!("No redirect URL provided"));
            }

            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }

            match CallbackServer::parse_callback_url(trimmed) {
                Ok(callback) if callback.state == expected_state => return Ok(callback),
                Ok(_) => {
                    writeln!(
                        writer,
                        "State mismatch. Paste the redirect URL from the same login attempt."
                    )?;
                }
                Err(err) => {
                    writeln!(writer, "Could not parse redirect URL: {}", err)?;
                }
            }
        }
    }
}

impl Default for OAuthClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_prompt_for_manual_callback_accepts_full_redirect_url() {
        let input = b"http://localhost:1455/auth/callback?code=abc123&state=xyz789\n";
        let mut reader = Cursor::new(input);
        let mut output = Vec::new();

        let result =
            OAuthClient::prompt_for_manual_callback(&mut reader, &mut output, "xyz789").unwrap();

        assert_eq!(result.code, "abc123");
        assert_eq!(result.state, "xyz789");
    }

    #[test]
    fn test_prompt_for_manual_callback_retries_until_valid_url() {
        let input = b"\nnot-a-url\nhttp://localhost:1455/auth/callback?code=abc123&state=xyz789\n";
        let mut reader = Cursor::new(input);
        let mut output = Vec::new();

        let result =
            OAuthClient::prompt_for_manual_callback(&mut reader, &mut output, "xyz789").unwrap();

        let rendered_output = String::from_utf8(output).unwrap();
        assert_eq!(result.code, "abc123");
        assert!(rendered_output.contains("Paste the full redirect URL"));
        assert!(rendered_output.contains("Could not parse redirect URL"));
    }

    #[test]
    fn test_prompt_for_manual_callback_rejects_state_mismatch() {
        let input = b"http://localhost:1455/auth/callback?code=abc123&state=wrong\nhttp://localhost:1455/auth/callback?code=def456&state=xyz789\n";
        let mut reader = Cursor::new(input);
        let mut output = Vec::new();

        let result =
            OAuthClient::prompt_for_manual_callback(&mut reader, &mut output, "xyz789").unwrap();

        let rendered_output = String::from_utf8(output).unwrap();
        assert_eq!(result.code, "def456");
        assert!(rendered_output.contains("State mismatch"));
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
