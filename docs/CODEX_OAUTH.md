# OpenAI Codex OAuth Authentication

This document describes the implementation of OAuth authentication for OpenAI Codex, allowing users to use their ChatGPT Plus/Pro subscription instead of paying for API credits.

## Overview

TermAI supports three authentication methods for OpenAI:

1. **OpenAI API Key** - Traditional API key authentication (pay-per-use)
2. **OpenAI Codex (OAuth)** - ChatGPT Plus/Pro subscription authentication
3. **Claude API Key** - Anthropic's Claude API

The Codex OAuth flow uses the same authentication mechanism as OpenAI's official Codex CLI, enabling users to leverage their existing ChatGPT subscription.

## OAuth Configuration

### Endpoints

| Parameter | Value |
|-----------|-------|
| Client ID | `app_EMoamEEZ73f0CkXaXp7hrann` |
| Auth Endpoint | `https://auth.openai.com/oauth/authorize` |
| Token Endpoint | `https://auth.openai.com/oauth/token` |
| Callback URL | `http://localhost:1455/auth/callback` |
| API Endpoint | `https://chatgpt.com/backend-api/codex/responses` |

### Scopes

```
openid profile email offline_access
```

### Critical Parameters

The OAuth flow requires two special parameters to enable ChatGPT subscription authentication instead of API organization access:

```
codex_cli_simplified_flow=true
id_token_add_organizations=true
```

Without these parameters, the flow defaults to requesting API organization access (which consumes API credits).

## Implementation Architecture

### Module Structure

```
src/auth/
├── mod.rs                 # Module exports
├── models.rs              # OAuthConfig, OAuthTokens, AuthMethod, TokenResponse
├── pkce.rs                # PKCE code verifier/challenge generation
├── callback_server.rs     # Local HTTP server on port 1455
├── oauth_client.rs        # OAuth PKCE flow implementation
└── token_manager.rs       # Token storage, refresh, validation
```

### Key Components

#### 1. PKCE Implementation (`pkce.rs`)

Implements Proof Key for Code Exchange (RFC 7636) for secure OAuth:

- `generate_code_verifier()` - Creates a 64-byte random string, base64url encoded
- `generate_code_challenge()` - SHA256 hash of verifier, base64url encoded (S256 method)
- `generate_state()` - Random state for CSRF protection

#### 2. Callback Server (`callback_server.rs`)

Local HTTP server that:
- Listens on `localhost:1455`
- Waits for OAuth callback with authorization code
- Parses query parameters (code, state, error)
- Returns success/error HTML to browser
- Implements 5-minute timeout

#### 3. OAuth Client (`oauth_client.rs`)

Handles the complete OAuth flow:

```rust
pub async fn authorize(&self) -> Result<OAuthTokens> {
    // 1. Generate PKCE parameters
    let code_verifier = generate_code_verifier();
    let code_challenge = generate_code_challenge(&code_verifier);
    let state = generate_state();

    // 2. Build authorization URL with special parameters
    let auth_url = self.build_auth_url(&state, &code_challenge);
    // Includes: codex_cli_simplified_flow=true, id_token_add_organizations=true

    // 3. Open browser
    webbrowser::open(&auth_url)?;

    // 4. Wait for callback
    let callback = server.wait_for_callback(timeout)?;

    // 5. Verify state (CSRF protection)
    if callback.state != state { return Err(...); }

    // 6. Exchange code for tokens
    self.exchange_code(&callback.code, &code_verifier).await
}
```

#### 4. Token Manager (`token_manager.rs`)

Manages token lifecycle:

- `load_tokens()` - Load from config database
- `save_tokens()` - Persist to database
- `get_valid_token()` - Return valid token, auto-refresh if expiring within 5 minutes
- `clear_tokens()` - Logout
- `is_authenticated()` - Check authentication status
- `auth_status()` - Detailed status (NotAuthenticated, Authenticated, Expired)

### Config Keys

Tokens are stored in the SQLite config database:

| Key | Description |
|-----|-------------|
| `codex_access_token` | OAuth access token |
| `codex_refresh_token` | OAuth refresh token |
| `codex_token_expiry` | Token expiration (RFC3339) |
| `codex_id_token` | OAuth ID token |

### Codex API Adapter

The Codex API uses a different endpoint than the standard OpenAI API:

```rust
// Standard OpenAI API
POST https://api.openai.com/v1/responses

// Codex API (ChatGPT subscription)
POST https://chatgpt.com/backend-api/codex/responses
```

Request/response format is similar but routed through ChatGPT backend.

## User Commands

### Authentication

```bash
# Start OAuth flow (opens browser)
termai config login-codex

# Check authentication status
termai config codex-status

# Clear tokens (logout)
termai config logout-codex

# Set Codex as default provider
termai config set-provider openai-codex
```

### Setup Wizard

The setup wizard (`termai setup`) includes Codex as a provider option:

```
? Which AI provider would you like to use?
    Claude (Anthropic) - Best for analysis & coding
    OpenAI (API Key) - Versatile general purpose
  > OpenAI Codex (ChatGPT Plus/Pro) - Use your subscription
    Both providers (recommended)
```

## Token Refresh

Tokens are automatically refreshed when:
- Access token expires within 5 minutes
- A request is made via `get_valid_token()`

The refresh flow uses the stored refresh token to obtain new tokens without requiring browser authentication.

## Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| "State mismatch" | CSRF protection triggered | Retry authentication |
| "Token expired, no refresh token" | Refresh token missing/invalid | Run `login-codex` again |
| "401 Unauthorized" | Session expired | Run `login-codex` again |
| "Timeout waiting for callback" | Browser auth not completed | Complete auth within 5 minutes |

## Security Considerations

1. **PKCE** - Prevents authorization code interception
2. **State Parameter** - CSRF protection
3. **Local Storage** - Tokens stored in local SQLite database
4. **Token Refresh** - Auto-refresh minimizes re-authentication
5. **Localhost Callback** - No external callback server needed

## Dependencies Added

```toml
base64 = "0.21"        # Base64 encoding for PKCE
sha2 = "0.10"          # SHA256 for PKCE challenge
rand = "0.8"           # Random generation for state/verifier
webbrowser = "1.0"     # Open browser for OAuth flow
```

Existing dependencies used:
- `tiny_http` - Callback server
- `reqwest` - HTTP client for token exchange
- `chrono` - Token expiry handling
- `serde` / `serde_json` - Token serialization

## Files Modified

| File | Changes |
|------|---------|
| `Cargo.toml` | Added OAuth dependencies |
| `src/main.rs` | Added `mod auth` |
| `src/args.rs` | Added `OpenaiCodex` provider, `LoginCodex`/`LogoutCodex`/`CodexStatus` actions |
| `src/config/model/keys.rs` | Added Codex token config keys |
| `src/commands/mod.rs` | Added `codex_auth` module |
| `src/commands/config.rs` | Handle Codex auth commands, made async |
| `src/commands/ask.rs` | Added `call_codex_api()` function |
| `src/chat/interactive.rs` | Support Codex provider in chat |
| `src/setup/wizard.rs` | Added Codex provider option |
| `src/llm/openai/adapter/mod.rs` | Export `codex_adapter` |
| `src/llm/openai/model/mod.rs` | Export `codex_api` |
| `src/llm/openai/service/mod.rs` | Export `codex` service |

## New Files

| File | Purpose |
|------|---------|
| `src/auth/mod.rs` | Auth module exports |
| `src/auth/models.rs` | OAuth data structures |
| `src/auth/pkce.rs` | PKCE implementation |
| `src/auth/callback_server.rs` | Local OAuth callback server |
| `src/auth/oauth_client.rs` | OAuth flow implementation |
| `src/auth/token_manager.rs` | Token persistence and refresh |
| `src/commands/codex_auth.rs` | Auth command handlers |
| `src/llm/openai/adapter/codex_adapter.rs` | Codex HTTP adapter |
| `src/llm/openai/model/codex_api.rs` | Codex request/response models |
| `src/llm/openai/service/codex.rs` | Codex chat service |

## References

- [OpenAI Codex CLI Authentication](https://developers.openai.com/codex/auth/)
- [OpenAI Codex CLI Source](https://github.com/openai/codex)
- [OAuth 2.0 PKCE (RFC 7636)](https://tools.ietf.org/html/rfc7636)
- [OpenCode Codex Auth Plugin](https://github.com/anomalyco/opencode/issues/3281)

## Disclaimer

This implementation uses the same OAuth flow as OpenAI's official Codex CLI. Users should ensure their usage complies with OpenAI's Terms of Service. For commercial applications or services serving multiple users, use the OpenAI Platform API with proper API keys.
