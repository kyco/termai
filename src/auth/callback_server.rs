//! Local HTTP callback server for OAuth redirects

use anyhow::{anyhow, Result};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use tiny_http::{Response, Server};

/// Result from the OAuth callback
#[derive(Debug)]
pub struct CallbackResult {
    pub code: String,
    pub state: String,
}

/// Local HTTP server to receive OAuth callbacks
pub struct CallbackServer {
    port: u16,
}

impl CallbackServer {
    /// Create a new callback server on the specified port
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Start the server and wait for the OAuth callback
    ///
    /// Returns the authorization code and state from the callback.
    /// Times out after the specified duration.
    pub fn wait_for_callback(&self, timeout: Duration) -> Result<CallbackResult> {
        let server = Server::http(format!("127.0.0.1:{}", self.port))
            .map_err(|e| anyhow!("Failed to start callback server: {}", e))?;

        // Channel to communicate callback result
        let (tx, rx): (Sender<Result<CallbackResult>>, Receiver<Result<CallbackResult>>) =
            mpsc::channel();

        // Handle requests in a separate thread
        let server_handle = thread::spawn(move || {
            // Set a short timeout for receiving requests
            if let Ok(Some(mut request)) = server.recv_timeout(timeout) {
                let url = request.url().to_string();

                // Parse the callback URL
                let result = Self::parse_callback_url(&url);

                // Send response to browser
                let response_html = match &result {
                    Ok(_) => Self::success_html(),
                    Err(e) => Self::error_html(&e.to_string()),
                };

                let response = Response::from_string(response_html)
                    .with_header(
                        tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"text/html"[..])
                            .unwrap(),
                    );

                // Consume the request body to avoid connection issues
                let mut body = Vec::new();
                let _ = request.as_reader().read_to_end(&mut body);

                let _ = request.respond(response);

                let _ = tx.send(result);
            } else {
                let _ = tx.send(Err(anyhow!("Timeout waiting for OAuth callback")));
            }
        });

        // Wait for the result
        let result = rx
            .recv_timeout(timeout + Duration::from_secs(5))
            .map_err(|_| anyhow!("Timeout waiting for callback response"))??;

        // Wait for the server thread to finish
        let _ = server_handle.join();

        Ok(result)
    }

    /// Parse the callback URL to extract the authorization code and state
    fn parse_callback_url(url: &str) -> Result<CallbackResult> {
        // URL format: /auth/callback?code=xxx&state=yyy
        let query_start = url.find('?').ok_or_else(|| anyhow!("No query parameters in callback URL"))?;
        let query = &url[query_start + 1..];

        let mut code = None;
        let mut state = None;
        let mut error = None;
        let mut error_description = None;

        for param in query.split('&') {
            let mut parts = param.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                let decoded = urlencoding_decode(value);
                match key {
                    "code" => code = Some(decoded),
                    "state" => state = Some(decoded),
                    "error" => error = Some(decoded),
                    "error_description" => error_description = Some(decoded),
                    _ => {}
                }
            }
        }

        // Check for OAuth errors
        if let Some(err) = error {
            let description = error_description.unwrap_or_else(|| "Unknown error".to_string());
            return Err(anyhow!("OAuth error: {} - {}", err, description));
        }

        let code = code.ok_or_else(|| anyhow!("No authorization code in callback"))?;
        let state = state.ok_or_else(|| anyhow!("No state in callback"))?;

        Ok(CallbackResult { code, state })
    }

    /// HTML response for successful authentication
    fn success_html() -> String {
        r#"<!DOCTYPE html>
<html>
<head>
    <title>TermAI - Authentication Successful</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 40px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 16px;
            backdrop-filter: blur(10px);
        }
        h1 { margin-bottom: 16px; }
        p { opacity: 0.9; }
        .checkmark {
            font-size: 64px;
            margin-bottom: 16px;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="checkmark">&#10003;</div>
        <h1>Authentication Successful!</h1>
        <p>You can close this window and return to TermAI.</p>
    </div>
</body>
</html>"#.to_string()
    }

    /// HTML response for authentication error
    fn error_html(error: &str) -> String {
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>TermAI - Authentication Error</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #e74c3c 0%, #c0392b 100%);
            color: white;
        }}
        .container {{
            text-align: center;
            padding: 40px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 16px;
            backdrop-filter: blur(10px);
        }}
        h1 {{ margin-bottom: 16px; }}
        p {{ opacity: 0.9; }}
        .error-icon {{
            font-size: 64px;
            margin-bottom: 16px;
        }}
        .error-details {{
            background: rgba(0, 0, 0, 0.2);
            padding: 12px;
            border-radius: 8px;
            margin-top: 16px;
            font-family: monospace;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error-icon">&#10060;</div>
        <h1>Authentication Failed</h1>
        <p>There was an error during authentication.</p>
        <div class="error-details">{}</div>
        <p style="margin-top: 20px;">Please close this window and try again.</p>
    </div>
</body>
</html>"#, error)
    }
}

/// Simple URL decoding (handles %XX sequences and + for space)
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                    } else {
                        result.push('%');
                        result.push_str(&hex);
                    }
                } else {
                    result.push('%');
                    result.push_str(&hex);
                }
            }
            '+' => result.push(' '),
            _ => result.push(c),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_callback_url_success() {
        let url = "/auth/callback?code=abc123&state=xyz789";
        let result = CallbackServer::parse_callback_url(url).unwrap();
        assert_eq!(result.code, "abc123");
        assert_eq!(result.state, "xyz789");
    }

    #[test]
    fn test_parse_callback_url_with_encoding() {
        let url = "/auth/callback?code=abc%2B123&state=xyz%3D789";
        let result = CallbackServer::parse_callback_url(url).unwrap();
        assert_eq!(result.code, "abc+123");
        assert_eq!(result.state, "xyz=789");
    }

    #[test]
    fn test_parse_callback_url_error() {
        let url = "/auth/callback?error=access_denied&error_description=User+cancelled";
        let result = CallbackServer::parse_callback_url(url);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("access_denied"));
    }

    #[test]
    fn test_parse_callback_url_missing_code() {
        let url = "/auth/callback?state=xyz789";
        let result = CallbackServer::parse_callback_url(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_urlencoding_decode() {
        assert_eq!(urlencoding_decode("hello%20world"), "hello world");
        assert_eq!(urlencoding_decode("hello+world"), "hello world");
        assert_eq!(urlencoding_decode("a%2Bb%3Dc"), "a+b=c");
    }
}
