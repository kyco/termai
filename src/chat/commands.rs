/// Represents different slash commands available in chat mode, organized by category
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatCommand {
    // General
    Help,
    Commands,
    Exit,
    #[allow(dead_code)]
    Quit,

    // Session management
    Save(Option<String>),
    NewSession(Option<String>),
    ListSessions,
    LoadSession(String),
    RenameSession(String),
    Clear,
    Retry,
    Branch(Option<String>),

    // Context management
    Context,
    AddContext(String),
    RemoveContext(String),

    // AI settings
    Model(Option<String>),
    Provider(Option<String>),
    Tools(Option<bool>),
    Status,
    Theme(Option<String>),
    Streaming(Option<bool>),
    Settings,
}

/// Categories for grouping commands in the palette
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandCategory {
    General,
    Session,
    Context,
    AiSettings,
}

impl CommandCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::General => "General",
            Self::Session => "Session",
            Self::Context => "Context",
            Self::AiSettings => "AI & Settings",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::General => ">>",
            Self::Session => "[]",
            Self::Context => "()",
            Self::AiSettings => "{}",
        }
    }

    /// Return all categories in display order
    pub fn all() -> &'static [CommandCategory] {
        &[
            Self::General,
            Self::Session,
            Self::Context,
            Self::AiSettings,
        ]
    }
}

/// A command entry for display in the palette
pub struct CommandEntry {
    pub command: &'static str,
    pub aliases: &'static str,
    pub description: &'static str,
    pub category: CommandCategory,
}

impl ChatCommand {
    /// Parse a line of input to determine if it's a slash command.
    /// Also supports `?` as a shortcut for opening the command palette.
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();

        // Support `?` as a shortcut for the command palette
        if input == "?" {
            return Some(ChatCommand::Commands);
        }

        if !input.starts_with('/') {
            return None;
        }

        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }

        match parts[0].to_lowercase().as_str() {
            // General
            "help" | "h" => Some(ChatCommand::Help),
            "commands" | "cmd" => Some(ChatCommand::Commands),
            "exit" | "quit" | "q" => Some(ChatCommand::Exit),

            // Session
            "save" | "s" => {
                let name = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Some(ChatCommand::Save(name))
            }
            "new" | "n" => {
                let name = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Some(ChatCommand::NewSession(name))
            }
            "sessions" | "ls" | "list" => Some(ChatCommand::ListSessions),
            "load" | "open" | "switch" => {
                // Missing arg returns an empty name; the handler shows usage.
                Some(ChatCommand::LoadSession(
                    parts.get(1..).map(|p| p.join(" ")).unwrap_or_default(),
                ))
            }
            "rename" => Some(ChatCommand::RenameSession(
                parts.get(1..).map(|p| p.join(" ")).unwrap_or_default(),
            )),
            "clear" | "c" => Some(ChatCommand::Clear),
            "retry" | "r" => Some(ChatCommand::Retry),
            "branch" | "b" => {
                let name = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Some(ChatCommand::Branch(name))
            }

            // Context
            "context" | "ctx" => Some(ChatCommand::Context),
            "add" => Some(ChatCommand::AddContext(
                parts.get(1..).map(|p| p.join(" ")).unwrap_or_default(),
            )),
            "remove" | "rm" => Some(ChatCommand::RemoveContext(
                parts.get(1..).map(|p| p.join(" ")).unwrap_or_default(),
            )),

            // AI settings
            "model" | "m" => {
                let model = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Some(ChatCommand::Model(model))
            }
            "provider" | "p" => {
                let provider = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Some(ChatCommand::Provider(provider))
            }
            "tools" | "t" => {
                let setting = if parts.len() > 1 {
                    match parts[1].to_lowercase().as_str() {
                        "on" | "true" | "enable" | "enabled" | "1" => Some(true),
                        "off" | "false" | "disable" | "disabled" | "0" => Some(false),
                        _ => None,
                    }
                } else {
                    None
                };
                Some(ChatCommand::Tools(setting))
            }
            "status" => Some(ChatCommand::Status),
            "theme" => {
                let theme = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Some(ChatCommand::Theme(theme))
            }
            "streaming" => {
                let setting = if parts.len() > 1 {
                    match parts[1].to_lowercase().as_str() {
                        "on" | "true" | "enable" | "enabled" | "1" => Some(true),
                        "off" | "false" | "disable" | "disabled" | "0" => Some(false),
                        _ => None,
                    }
                } else {
                    None
                };
                Some(ChatCommand::Streaming(setting))
            }
            "settings" | "config" => Some(ChatCommand::Settings),
            _ => None,
        }
    }

    /// Get help text for a specific command
    #[allow(dead_code)]
    pub fn help_text(&self) -> &'static str {
        match self {
            ChatCommand::Help => "Show quick help",
            ChatCommand::Commands => "Open command palette with all commands",
            ChatCommand::Exit | ChatCommand::Quit => "Exit chat mode",
            ChatCommand::Save(_) => "Save current session with optional name",
            ChatCommand::NewSession(_) => "Start a new session (saves current first)",
            ChatCommand::ListSessions => "List saved sessions",
            ChatCommand::LoadSession(_) => "Load/switch to a saved session",
            ChatCommand::RenameSession(_) => "Rename the active session",
            ChatCommand::Clear => "Clear conversation history",
            ChatCommand::Retry => "Regenerate the last AI response",
            ChatCommand::Branch(_) => "Create a new conversation branch",
            ChatCommand::Context => "Show current context information",
            ChatCommand::AddContext(_) => "Add file or directory to context",
            ChatCommand::RemoveContext(_) => "Remove file or directory from context",
            ChatCommand::Model(_) => "Switch AI model or show current",
            ChatCommand::Provider(_) => "Switch AI provider or show current",
            ChatCommand::Tools(_) => "Toggle tool usage (OpenAI only)",
            ChatCommand::Status => "Show current session status",
            ChatCommand::Theme(_) => "Switch display theme or list themes",
            ChatCommand::Streaming(_) => "Toggle streaming output",
            ChatCommand::Settings => "Show all current settings",
        }
    }

    /// Get the full command catalogue, organized by category
    pub fn command_palette() -> Vec<CommandEntry> {
        vec![
            // General
            CommandEntry {
                command: "/help",
                aliases: "/h",
                description: "Show quick help",
                category: CommandCategory::General,
            },
            CommandEntry {
                command: "/commands",
                aliases: "/cmd, ?",
                description: "Open this command palette",
                category: CommandCategory::General,
            },
            CommandEntry {
                command: "/exit",
                aliases: "/quit, /q",
                description: "Exit chat mode",
                category: CommandCategory::General,
            },
            // Session
            CommandEntry {
                command: "/save [name]",
                aliases: "/s",
                description: "Save / name the current conversation",
                category: CommandCategory::Session,
            },
            CommandEntry {
                command: "/sessions",
                aliases: "/ls, /list",
                description: "List saved sessions",
                category: CommandCategory::Session,
            },
            CommandEntry {
                command: "/new [name]",
                aliases: "/n",
                description: "Start a fresh session (auto-saves current)",
                category: CommandCategory::Session,
            },
            CommandEntry {
                command: "/load <name>",
                aliases: "/switch, /open",
                description: "Switch to a saved session's history",
                category: CommandCategory::Session,
            },
            CommandEntry {
                command: "/rename <name>",
                aliases: "",
                description: "Rename the active session",
                category: CommandCategory::Session,
            },
            CommandEntry {
                command: "/clear",
                aliases: "/c",
                description: "Clear conversation history",
                category: CommandCategory::Session,
            },
            CommandEntry {
                command: "/retry",
                aliases: "/r",
                description: "Regenerate last AI response",
                category: CommandCategory::Session,
            },
            CommandEntry {
                command: "/branch [name]",
                aliases: "/b",
                description: "Create conversation branch",
                category: CommandCategory::Session,
            },
            // Context
            CommandEntry {
                command: "/context",
                aliases: "/ctx",
                description: "Show current context info",
                category: CommandCategory::Context,
            },
            CommandEntry {
                command: "/add <path>",
                aliases: "",
                description: "Add file/directory to context",
                category: CommandCategory::Context,
            },
            CommandEntry {
                command: "/remove <path>",
                aliases: "/rm",
                description: "Remove from context",
                category: CommandCategory::Context,
            },
            // AI settings
            CommandEntry {
                command: "/model [name]",
                aliases: "/m",
                description: "Switch model or show current",
                category: CommandCategory::AiSettings,
            },
            CommandEntry {
                command: "/provider [name]",
                aliases: "/p",
                description: "Switch provider (claude/openai/codex)",
                category: CommandCategory::AiSettings,
            },
            CommandEntry {
                command: "/tools [on|off]",
                aliases: "/t",
                description: "Toggle tool usage (OpenAI)",
                category: CommandCategory::AiSettings,
            },
            CommandEntry {
                command: "/status",
                aliases: "",
                description: "Show session status overview",
                category: CommandCategory::AiSettings,
            },
            CommandEntry {
                command: "/theme [name]",
                aliases: "",
                description: "Switch theme or list themes",
                category: CommandCategory::AiSettings,
            },
            CommandEntry {
                command: "/streaming [on|off]",
                aliases: "",
                description: "Toggle streaming output",
                category: CommandCategory::AiSettings,
            },
            CommandEntry {
                command: "/settings",
                aliases: "/config",
                description: "Show all current settings",
                category: CommandCategory::AiSettings,
            },
        ]
    }

    /// Legacy flat list for backward compat (used by /help)
    pub fn all_commands() -> Vec<(&'static str, &'static str)> {
        Self::command_palette()
            .iter()
            .map(|entry| {
                if entry.aliases.is_empty() {
                    (entry.command, entry.description)
                } else {
                    // Leak a combined string so we can return &'static str
                    // This is fine since it's only called for display and the set is fixed
                    let combined: &'static str =
                        Box::leak(format!("{}, {}", entry.command, entry.aliases).into_boxed_str());
                    (combined, entry.description)
                }
            })
            .collect()
    }
}

/// Represents the result of processing user input
#[derive(Debug, Clone)]
pub enum InputType {
    Command(ChatCommand),
    Message(String),
    /// Looks like a slash command (starts with `/`) but matched no known verb.
    UnknownCommand(String),
}

impl InputType {
    /// Classify user input as a command, an unknown command, or a regular message.
    pub fn classify(input: &str) -> Self {
        let trimmed = input.trim();
        if let Some(command) = ChatCommand::parse(trimmed) {
            InputType::Command(command)
        } else if trimmed.starts_with('/') {
            // Started with '/' but no verb matched — a typo'd command. Surface a
            // hint instead of silently billing it to the model as a chat message.
            let verb = trimmed.split_whitespace().next().unwrap_or(trimmed);
            InputType::UnknownCommand(verb.to_string())
        } else {
            InputType::Message(input.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_parsing() {
        assert_eq!(ChatCommand::parse("/help"), Some(ChatCommand::Help));
        assert_eq!(ChatCommand::parse("/h"), Some(ChatCommand::Help));
        assert_eq!(ChatCommand::parse("/save"), Some(ChatCommand::Save(None)));
        assert_eq!(
            ChatCommand::parse("/save my_session"),
            Some(ChatCommand::Save(Some("my_session".to_string())))
        );
        assert_eq!(ChatCommand::parse("/context"), Some(ChatCommand::Context));
        assert_eq!(ChatCommand::parse("/clear"), Some(ChatCommand::Clear));
        assert_eq!(ChatCommand::parse("/exit"), Some(ChatCommand::Exit));
        assert_eq!(ChatCommand::parse("/quit"), Some(ChatCommand::Exit));
        assert_eq!(ChatCommand::parse("/retry"), Some(ChatCommand::Retry));
        assert_eq!(
            ChatCommand::parse("/add src/main.rs"),
            Some(ChatCommand::AddContext("src/main.rs".to_string()))
        );
        assert_eq!(
            ChatCommand::parse("/model gpt-5.2"),
            Some(ChatCommand::Model(Some("gpt-5.2".to_string())))
        );
        assert_eq!(ChatCommand::parse("/model"), Some(ChatCommand::Model(None)));
        assert_eq!(
            ChatCommand::parse("/provider openai"),
            Some(ChatCommand::Provider(Some("openai".to_string())))
        );
        assert_eq!(
            ChatCommand::parse("/provider"),
            Some(ChatCommand::Provider(None))
        );

        // Non-commands
        assert_eq!(ChatCommand::parse("hello world"), None);
        assert_eq!(ChatCommand::parse("not a command"), None);
        assert_eq!(ChatCommand::parse(""), None);
    }

    #[test]
    fn test_question_mark_shortcut() {
        assert_eq!(ChatCommand::parse("?"), Some(ChatCommand::Commands));
    }

    #[test]
    fn test_new_commands() {
        assert_eq!(ChatCommand::parse("/commands"), Some(ChatCommand::Commands));
        assert_eq!(ChatCommand::parse("/cmd"), Some(ChatCommand::Commands));
        assert_eq!(ChatCommand::parse("/status"), Some(ChatCommand::Status));
        assert_eq!(ChatCommand::parse("/settings"), Some(ChatCommand::Settings));
        assert_eq!(ChatCommand::parse("/config"), Some(ChatCommand::Settings));
        assert_eq!(ChatCommand::parse("/theme"), Some(ChatCommand::Theme(None)));
        assert_eq!(
            ChatCommand::parse("/theme dark"),
            Some(ChatCommand::Theme(Some("dark".to_string())))
        );
        assert_eq!(
            ChatCommand::parse("/streaming on"),
            Some(ChatCommand::Streaming(Some(true)))
        );
        assert_eq!(
            ChatCommand::parse("/streaming off"),
            Some(ChatCommand::Streaming(Some(false)))
        );
        assert_eq!(
            ChatCommand::parse("/streaming"),
            Some(ChatCommand::Streaming(None))
        );
    }

    #[test]
    fn test_session_management_commands() {
        assert_eq!(
            ChatCommand::parse("/new"),
            Some(ChatCommand::NewSession(None))
        );
        assert_eq!(
            ChatCommand::parse("/new my-plan"),
            Some(ChatCommand::NewSession(Some("my-plan".to_string())))
        );
        assert_eq!(
            ChatCommand::parse("/n"),
            Some(ChatCommand::NewSession(None))
        );
        assert_eq!(
            ChatCommand::parse("/sessions"),
            Some(ChatCommand::ListSessions)
        );
        assert_eq!(ChatCommand::parse("/ls"), Some(ChatCommand::ListSessions));
        assert_eq!(ChatCommand::parse("/list"), Some(ChatCommand::ListSessions));
        assert_eq!(
            ChatCommand::parse("/load refactor"),
            Some(ChatCommand::LoadSession("refactor".to_string()))
        );
        assert_eq!(
            ChatCommand::parse("/switch refactor"),
            Some(ChatCommand::LoadSession("refactor".to_string()))
        );
        // Missing arg parses to an empty name (handler shows usage), NOT unknown.
        assert_eq!(
            ChatCommand::parse("/load"),
            Some(ChatCommand::LoadSession(String::new()))
        );
        assert_eq!(
            ChatCommand::parse("/rename new-name"),
            Some(ChatCommand::RenameSession("new-name".to_string()))
        );
    }

    #[test]
    fn test_unknown_command_classification() {
        match InputType::classify("/saev oops") {
            InputType::UnknownCommand(verb) => assert_eq!(verb, "/saev"),
            other => panic!("expected UnknownCommand, got {:?}", other),
        }
        // A known verb with a missing arg is NOT unknown.
        match InputType::classify("/load") {
            InputType::Command(ChatCommand::LoadSession(_)) => {}
            other => panic!("expected LoadSession command, got {:?}", other),
        }
        // A normal message is unaffected.
        match InputType::classify("what is 2 + 2?") {
            InputType::Message(_) => {}
            other => panic!("expected Message, got {:?}", other),
        }
    }

    #[test]
    fn test_input_classification() {
        match InputType::classify("/help") {
            InputType::Command(ChatCommand::Help) => (),
            _ => panic!("Expected Help command"),
        }

        match InputType::classify("?") {
            InputType::Command(ChatCommand::Commands) => (),
            _ => panic!("Expected Commands command from ?"),
        }

        match InputType::classify("Hello, how are you?") {
            InputType::Message(msg) => assert_eq!(msg, "Hello, how are you?"),
            _ => panic!("Expected regular message"),
        }
    }

    #[test]
    fn test_command_palette_has_all_categories() {
        let palette = ChatCommand::command_palette();
        for cat in CommandCategory::all() {
            assert!(
                palette.iter().any(|e| e.category == *cat),
                "Missing commands for category {:?}",
                cat
            );
        }
    }
}
