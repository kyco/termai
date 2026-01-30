pub enum ConfigKeys {
    ChatGptApiKey,
    ClaudeApiKey,
    ProviderKey,
    Redacted,
    /// OAuth access token for Codex API
    CodexAccessToken,
    /// OAuth refresh token for Codex API
    CodexRefreshToken,
    /// Token expiry time (RFC3339 format)
    CodexTokenExpiry,
    /// OAuth ID token for Codex API
    CodexIdToken,
    /// Default model for OpenAI provider
    OpenAIDefaultModel,
    /// Default model for Claude provider
    ClaudeDefaultModel,
    /// Default model for Codex provider
    CodexDefaultModel,
    /// Cached OpenAI models (JSON)
    OpenAIModelsCache,
    /// Timestamp when models were last fetched
    OpenAIModelsCacheTimestamp,
}

impl ConfigKeys {
    pub fn to_key(&self) -> String {
        match self {
            Self::ChatGptApiKey => "chat_gpt_api_key".to_owned(),
            Self::ClaudeApiKey => "claude_api_key".to_owned(),
            Self::ProviderKey => "provider_key".to_owned(),
            Self::Redacted => "redacted".to_owned(),
            Self::CodexAccessToken => "codex_access_token".to_owned(),
            Self::CodexRefreshToken => "codex_refresh_token".to_owned(),
            Self::CodexTokenExpiry => "codex_token_expiry".to_owned(),
            Self::CodexIdToken => "codex_id_token".to_owned(),
            Self::OpenAIDefaultModel => "openai_default_model".to_owned(),
            Self::ClaudeDefaultModel => "claude_default_model".to_owned(),
            Self::CodexDefaultModel => "codex_default_model".to_owned(),
            Self::OpenAIModelsCache => "openai_models_cache".to_owned(),
            Self::OpenAIModelsCacheTimestamp => "openai_models_cache_timestamp".to_owned(),
        }
    }
}
