use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    // Legacy support for backwards compatibility during transition
    #[arg(long, hide = true)]
    pub chat_gpt_api_key: Option<String>,
    #[arg(long, hide = true)]
    pub claude_api_key: Option<String>,
    #[arg(long)]
    pub system_prompt: Option<String>,
    #[arg(long, hide = true)]
    pub redact_add: Option<String>,
    #[arg(long, hide = true)]
    pub redact_remove: Option<String>,
    #[arg(long, hide = true)]
    pub redact_list: bool,
    #[arg(short, long, hide = true)]
    pub print_config: bool,
    #[arg(long, hide = true)]
    pub sessions_all: bool,
    #[arg(long)]
    pub session: Option<String>,
    pub data: Option<String>,
    pub(crate) directory: Option<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub(crate) exclude: Vec<String>,
    #[arg(long, value_enum, hide = true)]
    pub provider: Option<Provider>,
    #[arg(short = 'd', long, value_delimiter = ',')]
    pub(crate) directories: Vec<String>,
    
    /// Enable smart context discovery (automatically finds relevant files)
    #[arg(long)]
    pub smart_context: bool,
    
    /// Maximum tokens for smart context (default: 4000) 
    #[arg(long, requires = "smart_context")]
    pub max_context_tokens: Option<usize>,
    
    /// Preview selected context before proceeding
    #[arg(long, requires = "smart_context")]
    pub preview_context: bool,
    
    /// Enable chunked analysis for large projects
    #[arg(long, requires = "smart_context")]
    pub chunked_analysis: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Interactive setup wizard to configure TermAI
    Setup,
    /// Manage configuration settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
    /// Manage redaction patterns for privacy
    Redact {
        #[command(subcommand)]
        action: RedactAction,
    },
    /// Manage chat sessions
    Sessions {
        #[command(subcommand)]
        action: SessionAction,
    },
    /// Start a chat session (default if no subcommand provided)
    Chat {
        /// Chat input
        input: Option<String>,
        /// Local directory to include as context
        directory: Option<String>,
        /// Multiple directories to include as context  
        #[arg(short = 'd', long, value_delimiter = ',')]
        directories: Vec<String>,
        /// Patterns to exclude from context
        #[arg(short, long, value_delimiter = ',')]
        exclude: Vec<String>,
        /// System prompt to use for this session
        #[arg(long)]
        system_prompt: Option<String>,
        /// Session name to use or create
        #[arg(long)]
        session: Option<String>,
        /// Enable smart context discovery (automatically finds relevant files)
        #[arg(long)]
        smart_context: bool,
        /// Query to guide smart context selection
        #[arg(long, requires = "smart_context")]
        context_query: Option<String>,
        /// Maximum tokens for smart context (default: 4000)
        #[arg(long, requires = "smart_context")]
        max_context_tokens: Option<usize>,
        /// Preview selected context before proceeding
        #[arg(long, requires = "smart_context")]
        preview_context: bool,
        /// Enable chunked analysis for large projects (breaks into manageable pieces)
        #[arg(long, requires = "smart_context")]
        chunked_analysis: bool,
        /// Chunking strategy: module, functional, token, hierarchical
        #[arg(long, requires = "chunked_analysis", default_value = "hierarchical")]
        chunk_strategy: String,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set OpenAI API key
    SetOpenai { api_key: String },
    /// Set Claude API key  
    SetClaude { api_key: String },
    /// Set default provider
    SetProvider {
        #[arg(value_enum)]
        provider: Provider,
    },
    /// Reset all configuration (clears API keys and settings)
    Reset,
}

#[derive(Subcommand, Debug)]
pub enum RedactAction {
    /// Add a new redaction pattern
    Add { pattern: String },
    /// Remove a redaction pattern
    Remove { pattern: String },
    /// List all redaction patterns
    List,
}

#[derive(Subcommand, Debug)]
pub enum SessionAction {
    /// List all sessions
    List,
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
        matches!(self.command, Some(Commands::Redact { .. }))
            || self.redact_add.is_some()
            || self.redact_remove.is_some()
            || self.redact_list
    }

    pub fn is_chat_gpt_api_key(&self) -> bool {
        matches!(
            self.command,
            Some(Commands::Config {
                action: ConfigAction::SetOpenai { .. }
            })
        ) || self.chat_gpt_api_key.is_some()
    }

    pub fn is_claude_api_key(&self) -> bool {
        matches!(
            self.command,
            Some(Commands::Config {
                action: ConfigAction::SetClaude { .. }
            })
        ) || self.claude_api_key.is_some()
    }

    pub fn is_sessions_all(&self) -> bool {
        matches!(
            self.command,
            Some(Commands::Sessions {
                action: SessionAction::List
            })
        ) || self.sessions_all
    }

    pub fn is_session(&self) -> bool {
        self.session.is_some()
    }

    pub fn is_provider(&self) -> bool {
        matches!(
            self.command,
            Some(Commands::Config {
                action: ConfigAction::SetProvider { .. }
            })
        ) || self.provider.is_some()
    }

    #[allow(dead_code)]
    pub fn is_setup(&self) -> bool {
        matches!(self.command, Some(Commands::Setup))
    }

    pub fn is_config_show(&self) -> bool {
        matches!(
            self.command,
            Some(Commands::Config {
                action: ConfigAction::Show
            })
        ) || self.print_config
    }

    #[allow(dead_code)]
    pub fn should_handle_chat(&self) -> bool {
        match &self.command {
            Some(Commands::Chat { .. }) => true,
            None => true, // Default to chat if no subcommand
            _ => false,
        }
    }

    pub fn get_chat_data(&self) -> Option<String> {
        match &self.command {
            Some(Commands::Chat { input, .. }) => input.clone(),
            _ => self.data.clone(),
        }
    }

    pub fn get_chat_directory(&self) -> Option<String> {
        match &self.command {
            Some(Commands::Chat { directory, .. }) => directory.clone(),
            _ => self.directory.clone(),
        }
    }

    pub fn get_chat_directories(&self) -> Vec<String> {
        match &self.command {
            Some(Commands::Chat { directories, .. }) => directories.clone(),
            _ => self.directories.clone(),
        }
    }

    pub fn get_chat_exclude(&self) -> Vec<String> {
        match &self.command {
            Some(Commands::Chat { exclude, .. }) => exclude.clone(),
            _ => self.exclude.clone(),
        }
    }

    pub fn get_chat_system_prompt(&self) -> Option<String> {
        match &self.command {
            Some(Commands::Chat { system_prompt, .. }) => system_prompt.clone(),
            _ => self.system_prompt.clone(),
        }
    }

    pub fn get_chat_session(&self) -> Option<String> {
        match &self.command {
            Some(Commands::Chat { session, .. }) => session.clone(),
            _ => self.session.clone(),
        }
    }

    #[allow(dead_code)]
    pub fn get_smart_context(&self) -> bool {
        match &self.command {
            Some(Commands::Chat { smart_context, .. }) => *smart_context,
            _ => self.smart_context,
        }
    }

    #[allow(dead_code)]
    pub fn get_max_context_tokens(&self) -> Option<usize> {
        match &self.command {
            Some(Commands::Chat { max_context_tokens, .. }) => *max_context_tokens,
            _ => self.max_context_tokens,
        }
    }

    #[allow(dead_code)]
    pub fn get_preview_context(&self) -> bool {
        match &self.command {
            Some(Commands::Chat { preview_context, .. }) => *preview_context,
            _ => self.preview_context,
        }
    }

    #[allow(dead_code)]
    pub fn get_chunked_analysis(&self) -> bool {
        match &self.command {
            Some(Commands::Chat { chunked_analysis, .. }) => *chunked_analysis,
            _ => self.chunked_analysis,
        }
    }
}
