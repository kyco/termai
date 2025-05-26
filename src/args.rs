use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub chat_gpt_api_key: Option<String>,
    #[arg(long)]
    pub claude_api_key: Option<String>,
    #[arg(long)]
    pub system_prompt: Option<String>,
    #[arg(long)]
    pub redact_add: Option<String>,
    #[arg(long)]
    pub redact_remove: Option<String>,
    #[arg(long)]
    pub redact_list: bool,
    #[arg(short, long)]
    pub print_config: bool,
    #[arg(long)]
    pub sessions_all: bool,
    #[arg(long)]
    pub session: Option<String>,
    #[arg(long)]
    pub ui: bool,
    pub data: Option<String>,
    pub(crate) directory: Option<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub(crate) exclude: Vec<String>,
    #[arg(long, value_enum)]
    pub provider: Option<Provider>,
    #[arg(short = 'd', long, value_delimiter = ',')]
    pub(crate) directories: Vec<String>,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, ValueEnum)]
pub enum Provider {
    Openapi,
    Claude,
}

impl Provider {
    pub fn new(s: &str) -> Provider {
        match s {
            "openapi" => Provider::Openapi,
            _ => Provider::Claude,
        }
    }

    pub fn to_str(self) -> &'static str {
        match self {
            Provider::Openapi => "openapi",
            Provider::Claude => "claude",
        }
    }
}

impl Args {
    pub fn is_redaction(&self) -> bool {
        self.redact_add.is_some() || self.redact_remove.is_some() || self.redact_list
    }

    pub fn is_chat_gpt_api_key(&self) -> bool {
        self.chat_gpt_api_key.is_some()
    }

    pub fn is_claude_api_key(&self) -> bool {
        self.claude_api_key.is_some()
    }

    pub fn is_sessions_all(&self) -> bool {
        self.sessions_all
    }

    pub fn is_session(&self) -> bool {
        self.session.is_some()
    }

    pub fn is_provider(&self) -> bool {
        self.provider.is_some()
    }

    pub fn is_ui(&self) -> bool {
        self.ui
    }
}
