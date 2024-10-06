pub enum ConfigKeys {
    ChatGptApiKey,
    Redacted,
}

impl ConfigKeys {
    pub fn to_key(&self) -> String {
        match self {
            Self::ChatGptApiKey => "chat_gpt_api_key".to_owned(),
            Self::Redacted => "redacted".to_owned(),
        }
    }
}
