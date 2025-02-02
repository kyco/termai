#[derive(Debug, Clone, PartialEq)]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Role {
    pub fn from_str(role: &str) -> Self {
        match role {
            "system" => Role::System,
            "user" => Role::User,
            "assistant" => Role::Assistant,
            _ => Role::User,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Role::System => "system".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
        }
    }
}
