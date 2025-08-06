//use anyhow::{anyhow, Result};

/// Represents different slash commands available in chat mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatCommand {
    Help,
    Save(Option<String>),
    Context,
    Clear,
    Exit,
    #[allow(dead_code)]
    Quit,
    Retry,
    Branch(Option<String>),
    AddContext(String),
    RemoveContext(String),
}

impl ChatCommand {
    /// Parse a line of input to determine if it's a slash command
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        
        if !input.starts_with('/') {
            return None;
        }
        
        let parts: Vec<&str> = input[1..].split_whitespace().collect();
        if parts.is_empty() {
            return None;
        }
        
        match parts[0].to_lowercase().as_str() {
            "help" | "h" => Some(ChatCommand::Help),
            "save" | "s" => {
                let name = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Some(ChatCommand::Save(name))
            }
            "context" | "ctx" => Some(ChatCommand::Context),
            "clear" | "c" => Some(ChatCommand::Clear),
            "exit" | "quit" | "q" => Some(ChatCommand::Exit),
            "retry" | "r" => Some(ChatCommand::Retry),
            "branch" | "b" => {
                let name = if parts.len() > 1 {
                    Some(parts[1..].join(" "))
                } else {
                    None
                };
                Some(ChatCommand::Branch(name))
            }
            "add" => {
                if parts.len() > 1 {
                    Some(ChatCommand::AddContext(parts[1..].join(" ")))
                } else {
                    None
                }
            }
            "remove" | "rm" => {
                if parts.len() > 1 {
                    Some(ChatCommand::RemoveContext(parts[1..].join(" ")))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    
    /// Get help text for a specific command
    #[allow(dead_code)]
    pub fn help_text(&self) -> &'static str {
        match self {
            ChatCommand::Help => "Show this help message",
            ChatCommand::Save(_) => "Save current session with optional name",
            ChatCommand::Context => "Show current context information",
            ChatCommand::Clear => "Clear conversation history",
            ChatCommand::Exit | ChatCommand::Quit => "Exit chat mode",
            ChatCommand::Retry => "Regenerate the last AI response",
            ChatCommand::Branch(_) => "Create a new conversation branch",
            ChatCommand::AddContext(_) => "Add file or directory to context",
            ChatCommand::RemoveContext(_) => "Remove file or directory from context",
        }
    }
    
    /// Get all available commands for help display
    pub fn all_commands() -> Vec<(&'static str, &'static str)> {
        vec![
            ("/help, /h", "Show this help message"),
            ("/save [name], /s", "Save current session with optional name"),
            ("/context, /ctx", "Show current context information"),
            ("/clear, /c", "Clear conversation history"),
            ("/exit, /quit, /q", "Exit chat mode"),
            ("/retry, /r", "Regenerate the last AI response"),
            ("/branch [name], /b", "Create a new conversation branch"),
            ("/add <path>", "Add file or directory to context"),
            ("/remove <path>, /rm", "Remove file or directory from context"),
        ]
    }
}

/// Represents the result of processing user input
#[derive(Debug, Clone)]
pub enum InputType {
    Command(ChatCommand),
    Message(String),
}

impl InputType {
    /// Classify user input as either a command or a regular message
    pub fn classify(input: &str) -> Self {
        if let Some(command) = ChatCommand::parse(input) {
            InputType::Command(command)
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
        assert_eq!(ChatCommand::parse("/save my_session"), Some(ChatCommand::Save(Some("my_session".to_string()))));
        assert_eq!(ChatCommand::parse("/context"), Some(ChatCommand::Context));
        assert_eq!(ChatCommand::parse("/clear"), Some(ChatCommand::Clear));
        assert_eq!(ChatCommand::parse("/exit"), Some(ChatCommand::Exit));
        assert_eq!(ChatCommand::parse("/quit"), Some(ChatCommand::Exit));
        assert_eq!(ChatCommand::parse("/retry"), Some(ChatCommand::Retry));
        assert_eq!(ChatCommand::parse("/add src/main.rs"), Some(ChatCommand::AddContext("src/main.rs".to_string())));
        
        // Test non-commands
        assert_eq!(ChatCommand::parse("hello world"), None);
        assert_eq!(ChatCommand::parse("not a command"), None);
        assert_eq!(ChatCommand::parse(""), None);
    }

    #[test]
    fn test_input_classification() {
        match InputType::classify("/help") {
            InputType::Command(ChatCommand::Help) => (),
            _ => panic!("Expected Help command"),
        }
        
        match InputType::classify("Hello, how are you?") {
            InputType::Message(msg) => assert_eq!(msg, "Hello, how are you?"),
            _ => panic!("Expected regular message"),
        }
    }
}