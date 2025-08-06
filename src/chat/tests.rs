#[cfg(test)]
mod tests {
    use crate::chat::commands::{ChatCommand, InputType};
    use crate::chat::formatter::ChatFormatter;
    use crate::llm::common::model::role::Role;
    use chrono::Local;

    #[test]
    fn test_command_parsing() {
        // Test basic commands
        assert_eq!(ChatCommand::parse("/help"), Some(ChatCommand::Help));
        assert_eq!(ChatCommand::parse("/h"), Some(ChatCommand::Help));
        assert_eq!(ChatCommand::parse("/exit"), Some(ChatCommand::Exit));
        assert_eq!(ChatCommand::parse("/quit"), Some(ChatCommand::Exit));
        assert_eq!(ChatCommand::parse("/clear"), Some(ChatCommand::Clear));
        assert_eq!(ChatCommand::parse("/context"), Some(ChatCommand::Context));
        assert_eq!(ChatCommand::parse("/retry"), Some(ChatCommand::Retry));
        
        // Test commands with arguments
        assert_eq!(ChatCommand::parse("/save my_session"), Some(ChatCommand::Save(Some("my_session".to_string()))));
        assert_eq!(ChatCommand::parse("/save"), Some(ChatCommand::Save(None)));
        assert_eq!(ChatCommand::parse("/branch my_branch"), Some(ChatCommand::Branch(Some("my_branch".to_string()))));
        assert_eq!(ChatCommand::parse("/add src/main.rs"), Some(ChatCommand::AddContext("src/main.rs".to_string())));
        assert_eq!(ChatCommand::parse("/remove src/main.rs"), Some(ChatCommand::RemoveContext("src/main.rs".to_string())));
        
        // Test invalid commands
        assert_eq!(ChatCommand::parse("/invalid"), None);
        assert_eq!(ChatCommand::parse("regular message"), None);
        assert_eq!(ChatCommand::parse(""), None);
        assert_eq!(ChatCommand::parse(" "), None);
        
        // Test case insensitivity
        assert_eq!(ChatCommand::parse("/HELP"), Some(ChatCommand::Help));
        assert_eq!(ChatCommand::parse("/Help"), Some(ChatCommand::Help));
    }
    
    #[test]
    fn test_input_classification() {
        // Test command classification
        match InputType::classify("/help") {
            InputType::Command(ChatCommand::Help) => (),
            _ => panic!("Expected Help command"),
        }
        
        match InputType::classify("/save test") {
            InputType::Command(ChatCommand::Save(Some(name))) => {
                assert_eq!(name, "test");
            },
            _ => panic!("Expected Save command with name"),
        }
        
        // Test message classification
        match InputType::classify("Hello, how are you?") {
            InputType::Message(msg) => assert_eq!(msg, "Hello, how are you?"),
            _ => panic!("Expected regular message"),
        }
        
        match InputType::classify("This is a regular message") {
            InputType::Message(msg) => assert_eq!(msg, "This is a regular message"),
            _ => panic!("Expected regular message"),
        }
    }
    
    #[test]
    fn test_command_help_text() {
        assert_eq!(ChatCommand::Help.help_text(), "Show this help message");
        assert_eq!(ChatCommand::Save(None).help_text(), "Save current session with optional name");
        assert_eq!(ChatCommand::Context.help_text(), "Show current context information");
        assert_eq!(ChatCommand::Clear.help_text(), "Clear conversation history");
        assert_eq!(ChatCommand::Exit.help_text(), "Exit chat mode");
        assert_eq!(ChatCommand::Retry.help_text(), "Regenerate the last AI response");
        assert_eq!(ChatCommand::Branch(None).help_text(), "Create a new conversation branch");
        assert_eq!(ChatCommand::AddContext("".to_string()).help_text(), "Add file or directory to context");
        assert_eq!(ChatCommand::RemoveContext("".to_string()).help_text(), "Remove file or directory from context");
    }
    
    #[test]
    fn test_all_commands_list() {
        let commands = ChatCommand::all_commands();
        assert!(!commands.is_empty());
        
        // Check that help command is included
        let help_found = commands.iter().any(|(cmd, _)| cmd.contains("/help"));
        assert!(help_found);
        
        // Check that exit commands are included
        let exit_found = commands.iter().any(|(cmd, _)| cmd.contains("/exit"));
        assert!(exit_found);
    }
    
    #[test]
    fn test_formatter_message_formatting() {
        let formatter = ChatFormatter::new();
        let timestamp = Local::now();
        
        // Test user message formatting
        let user_msg = formatter.format_message(&Role::User, "Hello world", Some(timestamp));
        assert!(user_msg.contains("💬 You"));
        assert!(user_msg.contains("Hello world"));
        assert!(user_msg.contains(&timestamp.format("%H:%M:%S").to_string()));
        
        // Test AI message formatting  
        let ai_msg = formatter.format_message(&Role::Assistant, "Hi there!", Some(timestamp));
        assert!(ai_msg.contains("🤖 AI"));
        assert!(ai_msg.contains("Hi there!"));
        
        // Test system message formatting
        let system_msg = formatter.format_message(&Role::System, "System message", Some(timestamp));
        assert!(system_msg.contains("⚙️ System"));
        assert!(system_msg.contains("System message"));
    }
    
    #[test]
    fn test_formatter_system_messages() {
        let formatter = ChatFormatter::new();
        
        let success = formatter.format_success("Operation completed");
        assert!(success.contains("✅"));
        assert!(success.contains("Operation completed"));
        
        let error = formatter.format_error("Something went wrong");
        assert!(error.contains("❌"));
        assert!(error.contains("Something went wrong"));
        
        let warning = formatter.format_warning("Be careful");
        assert!(warning.contains("⚠️"));
        assert!(warning.contains("Be careful"));
        
        let system = formatter.format_system_message("Info message");
        assert!(system.contains("💡"));
        assert!(system.contains("Info message"));
    }
    
    #[test]
    fn test_formatter_welcome_message() {
        let formatter = ChatFormatter::new();
        let welcome = formatter.format_welcome();
        
        assert!(welcome.contains("TermAI Interactive Chat Mode"));
        assert!(welcome.contains("/help"));
        assert!(welcome.contains("Type your message"));
        assert!(welcome.contains("┌"));  // Check for proper box formatting
        assert!(welcome.contains("└"));
    }
    
    #[test]
    fn test_formatter_help() {
        let formatter = ChatFormatter::new();
        let commands = ChatCommand::all_commands();
        let help = formatter.format_help(&commands);
        
        assert!(help.contains("Available Commands"));
        assert!(help.contains("/help"));
        assert!(help.contains("/exit"));
        assert!(help.contains("Tip:"));
    }
    
    #[test]
    fn test_formatter_context_info() {
        let formatter = ChatFormatter::new();
        let files = vec!["src/main.rs".to_string(), "src/lib.rs".to_string()];
        let info = formatter.format_context_info(2, &files);
        
        assert!(info.contains("Context Information"));
        assert!(info.contains("Total files: 2"));
        assert!(info.contains("src/main.rs"));
        assert!(info.contains("src/lib.rs"));
    }
    
    #[test]
    fn test_formatter_context_info_many_files() {
        let formatter = ChatFormatter::new();
        let files: Vec<String> = (0..15).map(|i| format!("file_{}.rs", i)).collect();
        let info = formatter.format_context_info(15, &files);
        
        assert!(info.contains("Total files: 15"));
        assert!(info.contains("file_0.rs"));
        assert!(info.contains("... and 5 more"));  // Should show truncation
    }
    
    #[test]
    fn test_formatter_status_messages() {
        let formatter = ChatFormatter::new();
        
        let thinking = formatter.format_thinking();
        assert!(thinking.contains("🤔"));
        assert!(thinking.contains("thinking"));
        
        let saved = formatter.format_session_saved("test_session");
        assert!(saved.contains("💾"));
        assert!(saved.contains("test_session"));
        
        let cleared = formatter.format_conversation_cleared();
        assert!(cleared.contains("🗑️"));
        assert!(cleared.contains("cleared"));
    }
}