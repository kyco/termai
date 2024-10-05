pub enum ConfigKeys {
    chat_gpt_api_key,
}

impl ConfigKeys {
    pub fn to_key(&self) -> String {
        match self {
            Self::chat_gpt_api_key => "chat_gpt_api_key".to_owned(),
        }
    }
}
