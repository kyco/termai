use anyhow::{anyhow, Result};
use chrono::Local;
use std::io::Write;
use std::path::Path;

use crate::chat::commands::{ChatCommand, InputType};
use crate::chat::formatter::ChatFormatter;
use crate::chat::repl::ChatRepl;
use crate::chat::state::ChatState;
use crate::config::repository::ConfigRepository;
use crate::llm::common::model::role::Role;
use crate::path::extract::extract_content;
use crate::path::model::Files;
//use crate::session::model::message::Message;
use crate::session::model::session::Session;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::session::service::sessions_service;
use crate::ui::timer::ThinkingTimer;
use crate::repository::db::SqliteRepository;
use crate::branch::BranchService;

/// Manages an interactive chat session with REPL interface
pub struct InteractiveSession<'a, R, SR, MR>
where
    R: ConfigRepository,
    SR: SessionRepository,
    MR: MessageRepository,
{
    repl: ChatRepl,
    formatter: ChatFormatter,
    session: Session,
    config_repo: &'a R,
    session_repo: &'a SR,
    message_repo: &'a MR,
    #[allow(dead_code)]
    sqlite_repo: &'a SqliteRepository,
    context_files: Vec<Files>,
    should_exit: bool,
    ctrl_c_pressed: bool,
    chat_state: ChatState,
}

impl<'a, R, SR, MR> InteractiveSession<'a, R, SR, MR>
where
    R: ConfigRepository,
    SR: SessionRepository,
    MR: MessageRepository,
{
    /// Create a new interactive session
    pub fn new(
        config_repo: &'a R,
        session_repo: &'a SR,
        message_repo: &'a MR,
        sqlite_repo: &'a SqliteRepository,
        session: Session,
        context_files: Vec<Files>,
    ) -> Result<Self> {
        let repl = ChatRepl::new()?;
        let formatter = ChatFormatter::new();
        
        // Initialize chat state with current provider and model from config
        let chat_state = Self::initialize_chat_state(config_repo)?;

        Ok(Self {
            repl,
            formatter,
            session,
            config_repo,
            session_repo,
            message_repo,
            sqlite_repo,
            context_files,
            should_exit: false,
            ctrl_c_pressed: false,
            chat_state,
        })
    }

    /// Start the interactive chat session
    pub async fn run(&mut self) -> Result<()> {
        // Show welcome message
        self.display_welcome();

        // Show initial context info if any
        if !self.context_files.is_empty() {
            self.display_context_info();
        }

        // Main chat loop
        loop {
            if self.should_exit {
                break;
            }

            // Continue with next input

            match self.repl.read_line() {
                Ok(input) => {
                    // Reset Ctrl+C flag when user provides input
                    self.ctrl_c_pressed = false;

                    if let Err(e) = self.process_input(&input).await {
                        self.repl
                            .print_message(&self.formatter.format_error(&e.to_string()));
                    }
                    // Processing complete, loop will continue to next read_line()
                }
                Err(e) => {
                    if e.to_string().contains("Interrupted") {
                        // Ctrl+C pressed - handle double press for exit
                        if self.ctrl_c_pressed {
                            // Second Ctrl+C - exit immediately
                            break;
                        } else {
                            // First Ctrl+C - show exit message and set flag
                            self.ctrl_c_pressed = true;
                            self.repl.print_message(&self.formatter.format_warning(
                                "Press Ctrl+C again to exit, or type /exit to quit gracefully",
                            ));
                        }
                    } else if e.to_string().contains("EOF") {
                        // Ctrl+D pressed - exit gracefully
                        break;
                    } else {
                        self.repl
                            .print_message(&self.formatter.format_error(&e.to_string()));
                    }
                }
            }
        }

        // Save session and history on exit
        self.save_on_exit().await?;
        self.repl.print_message(
            &self
                .formatter
                .format_success("Chat session ended. Goodbye! 👋"),
        );

        Ok(())
    }

    /// Process user input (command or message)
    async fn process_input(&mut self, input: &str) -> Result<()> {
        let input = input.trim();
        if input.is_empty() {
            return Ok(());
        }

        match InputType::classify(input) {
            InputType::Command(command) => self.handle_command(command).await,
            InputType::Message(message) => self.handle_message(message).await,
        }
    }

    /// Handle slash commands
    async fn handle_command(&mut self, command: ChatCommand) -> Result<()> {
        match command {
            ChatCommand::Help => {
                let help_text = self.formatter.format_help(&ChatCommand::all_commands());
                self.repl.print_message(&help_text);
            }
            ChatCommand::Save(name) => {
                let session_name = name
                    .unwrap_or_else(|| format!("chat_{}", Local::now().format("%Y%m%d_%H%M%S")));
                self.session.name = session_name.clone();
                sessions_service::session_add_messages(
                    self.session_repo,
                    self.message_repo,
                    &mut self.session,
                )?;
                self.repl
                    .print_message(&self.formatter.format_session_saved(&session_name));
            }
            ChatCommand::Context => {
                self.display_context_info();
            }
            ChatCommand::Clear => {
                self.session.messages.clear();
                self.repl.clear_screen();
                self.display_welcome();
                self.repl
                    .print_message(&self.formatter.format_conversation_cleared());
            }
            ChatCommand::Exit | ChatCommand::Quit => {
                self.should_exit = true;
            }
            ChatCommand::Retry => {
                if let Some(last_message) = self.session.messages.last() {
                    if last_message.role == Role::Assistant {
                        // Remove the last AI response and regenerate
                        self.session.messages.pop();
                        if let Some(user_message) = self.session.messages.last() {
                            if user_message.role == Role::User {
                                let content = user_message.content.clone();
                                self.generate_ai_response(&content).await?;
                            }
                        }
                    } else {
                        self.repl.print_message(
                            &self.formatter.format_warning("No AI response to retry"),
                        );
                    }
                } else {
                    self.repl.print_message(
                        &self
                            .formatter
                            .format_warning("No previous message to retry"),
                    );
                }
            }
            ChatCommand::Branch(name) => {
                self.handle_branch_command(name).await?;
            }
            ChatCommand::AddContext(path) => {
                self.add_context_path(&path)?;
            }
            ChatCommand::RemoveContext(path) => {
                self.remove_context_path(&path);
            }
            ChatCommand::Model(model_name) => {
                self.handle_model_command(model_name).await?;
            }
            ChatCommand::Provider(provider_name) => {
                self.handle_provider_command(provider_name).await?;
            }
        }
        Ok(())
    }

    /// Handle regular chat messages
    async fn handle_message(&mut self, message: String) -> Result<()> {
        // Add user message to session
        self.session.add_raw_message(message.clone(), Role::User);

        // Don't display user message again - rustyline already showed it
        // Just generate AI response
        self.generate_ai_response(&message).await?;

        Ok(())
    }

    /// Generate AI response for the given user input
    async fn generate_ai_response(&mut self, user_input: &str) -> Result<()> {
        // Start thinking timer (no separate message needed)
        let mut timer = ThinkingTimer::new();
        timer.start();

        // Create input with context
        let input_with_context = self.create_contextual_input(user_input);

        // Add context to session
        if !self.context_files.is_empty() {
            // Update the last user message to include context
            if let Some(last_msg) = self.session.messages.last_mut() {
                if last_msg.role == Role::User {
                    last_msg.content = input_with_context;
                }
            }
        }

        // Redact sensitive information
        self.session.redact(self.config_repo);

        // Call AI service based on configured provider
        let result = self.call_ai_service().await;

        timer.stop();

        // Ensure thinking indicator is completely cleared before showing response
        print!("\r\x1b[2K");
        std::io::stdout().flush().unwrap();

        match result {
            Ok(_) => {
                // Display AI response with enhanced formatting
                if let Some(last_message) = self.session.messages.last() {
                    if last_message.role == Role::Assistant {
                        // Use the new async formatter for enhanced markdown and syntax highlighting
                        if let Err(e) = self.formatter.format_message_async(
                            &Role::Assistant,
                            &last_message.content,
                            Some(Local::now()),
                        ).await {
                            eprintln!("Error formatting AI response: {}", e);
                            // Fallback to basic formatting
                            let formatted_ai = self.formatter.format_message(
                                &Role::Assistant,
                                &last_message.content,
                                Some(Local::now()),
                            );
                            println!("{}", formatted_ai);
                        }
                        std::io::stdout().flush().unwrap();
                    }
                }

                // Save session automatically
                sessions_service::session_add_messages(
                    self.session_repo,
                    self.message_repo,
                    &mut self.session,
                )?;
            }
            Err(e) => {
                self.repl
                    .print_message(&self.formatter.format_error(&format!("AI Error: {}", e)));

                // Remove the failed user message to keep session clean
                if let Some(last_msg) = self.session.messages.last() {
                    if last_msg.role == Role::User {
                        self.session.messages.pop();
                    }
                }
            }
        }

        // Unredact for display
        self.session.unredact();

        // Ensure we return control properly
        std::io::stdout().flush().unwrap();

        Ok(())
    }

    /// Call the AI service based on current chat state provider
    async fn call_ai_service(&mut self) -> Result<()> {
        use crate::config::model::keys::ConfigKeys;
        use crate::config::service::config_service;
        use crate::llm::{claude, openai};

        // Use provider from chat state instead of config
        match self.chat_state.provider.as_str() {
            "claude" => {
                let api_key = config_service::fetch_by_key(
                    self.config_repo,
                    &ConfigKeys::ClaudeApiKey.to_key(),
                )?;
                claude::service::chat::chat(&api_key.value, &mut self.session).await?;
            }
            "openai" => {
                let api_key = config_service::fetch_by_key(
                    self.config_repo,
                    &ConfigKeys::ChatGptApiKey.to_key(),
                )?;
                openai::service::chat::chat(&api_key.value, &mut self.session).await?;
            }
            _ => {
                return Err(anyhow!("Unsupported provider: {}", self.chat_state.provider));
            }
        }

        Ok(())
    }

    /// Create input with local context
    fn create_contextual_input(&self, user_input: &str) -> String {
        if self.context_files.is_empty() {
            return user_input.to_string();
        }

        let local_context: Vec<String> = self
            .context_files
            .iter()
            .map(|file| format!("{}\n```\n{}```", file.path, file.content))
            .collect();

        format!("{}\n{}", user_input, local_context.join("\n"))
    }

    /// Add a path to the context
    fn add_context_path(&mut self, path: &str) -> Result<()> {
        if !Path::new(path).exists() {
            return Err(anyhow!("Path does not exist: {}", path));
        }

        // Extract content from the path
        let new_context = extract_content(&Some(path.to_string()), &vec![], &vec![]);

        if let Some(mut files) = new_context {
            // Remove duplicates and add new files
            for file in files.drain(..) {
                if !self.context_files.iter().any(|f| f.path == file.path) {
                    self.context_files.push(file);
                }
            }
            self.repl.print_message(
                &self
                    .formatter
                    .format_success(&format!("Added '{}' to context", path)),
            );
            self.display_context_info();
        }

        Ok(())
    }

    /// Remove a path from the context
    fn remove_context_path(&mut self, path: &str) {
        let initial_count = self.context_files.len();
        self.context_files.retain(|file| !file.path.contains(path));

        if self.context_files.len() < initial_count {
            self.repl.print_message(
                &self
                    .formatter
                    .format_success(&format!("Removed files matching '{}' from context", path)),
            );
            self.display_context_info();
        } else {
            self.repl.print_message(
                &self
                    .formatter
                    .format_warning(&format!("No files matching '{}' found in context", path)),
            );
        }
    }

    /// Display welcome message
    fn display_welcome(&self) {
        println!(); // Add spacing before welcome
        self.repl.print_message(&self.formatter.format_welcome());
        println!(); // Add spacing after welcome
    }

    /// Display current context information
    fn display_context_info(&self) {
        let file_paths: Vec<String> = self.context_files.iter().map(|f| f.path.clone()).collect();
        let context_info = self
            .formatter
            .format_context_info(file_paths.len(), &file_paths);
        self.repl.print_message(&context_info);
    }

    /// Save session and history when exiting
    async fn save_on_exit(&mut self) -> Result<()> {
        // Save command history
        self.repl.save_history()?;

        // Auto-save session if it has messages and no name
        if !self.session.messages.is_empty() && self.session.name == "temporary" {
            let auto_name = format!("auto_save_{}", Local::now().format("%Y%m%d_%H%M%S"));
            self.session.name = auto_name.clone();
            sessions_service::session_add_messages(
                self.session_repo,
                self.message_repo,
                &mut self.session,
            )?;
            self.repl.print_message(
                &self
                    .formatter
                    .format_success(&format!("Auto-saved session as '{}'", auto_name)),
            );
        }

        Ok(())
    }

    /// Handle the /branch command
    async fn handle_branch_command(&mut self, name: Option<String>) -> Result<()> {
        // Generate branch name with context hint
        let branch_name = if let Some(name) = name.clone() {
            name
        } else {
            BranchService::generate_branch_name(&self.session.id, None)
        };

        // Create branch from current session state
        // Note: Need &mut SqliteRepository but we only have &SqliteRepository
        // This is a limitation of the current design. For now, show what the command would do:
        let message = if name.is_some() {
            format!("🌿 Would create branch '{}' from current conversation state", branch_name)
        } else {
            format!("🌿 Would create auto-named branch '{}' from current conversation state", branch_name)
        };

        // Display the branch creation message
        self.repl.print_message(&self.formatter.format_success(&message));

        // Show branch creation info
        let info_lines = vec![
            "📋 Branch would include:".to_string(),
            format!("   • {} messages from current conversation", self.session.messages.len()),
            "   • Full conversation context preserved".to_string(),
            "   • Ready for exploring alternative approaches".to_string(),
        ];

        for line in info_lines {
            println!("  {}", line); // Simple formatting for info lines
        }

        // TODO: Actually create the branch when we have mutable access to repo
        // For now, this demonstrates the UI and command structure
        self.repl.print_message(&self.formatter.format_warning(
            "⚠️  Branch creation temporarily disabled - requires mutable database access"
        ));

        Ok(())
    }

    /// Initialize chat state from current configuration
    fn initialize_chat_state(config_repo: &R) -> Result<ChatState> {
        use crate::args::Provider;
        use crate::config::model::keys::ConfigKeys;
        use crate::config::service::config_service;

        // Get current provider from config
        let provider_config = config_service::fetch_by_key(
            config_repo, 
            &ConfigKeys::ProviderKey.to_key()
        )?;
        let provider = Provider::new(&provider_config.value);

        let provider_str = match provider {
            Provider::Claude => "claude",
            Provider::Openai => "openai",
        };

        // Get current model - use default for the provider
        let chat_state = ChatState::new(provider_str.to_string(),
            match provider {
                Provider::Claude => "claude-sonnet-4-20250514".to_string(),
                Provider::Openai => "gpt-5.1".to_string(),
            }
        );

        Ok(chat_state)
    }

    /// Handle model switching command
    async fn handle_model_command(&mut self, model_name: Option<String>) -> Result<()> {
        match model_name {
            Some(model) => {
                // Switch to specified model
                match self.chat_state.switch_model(model) {
                    Ok(message) => {
                        self.repl.print_message(&self.formatter.format_success(&message));
                        
                        // Update the configuration to reflect the new provider/model
                        self.update_config_from_state().await?;
                    }
                    Err(error) => {
                        self.repl.print_message(&self.formatter.format_error(&error));
                    }
                }
            }
            None => {
                // Show current model and available models
                self.repl.print_message(&self.chat_state.status());
                println!();
                self.repl.print_message(&self.chat_state.list_models());
            }
        }
        Ok(())
    }

    /// Handle provider switching command
    async fn handle_provider_command(&mut self, provider_name: Option<String>) -> Result<()> {
        match provider_name {
            Some(provider) => {
                // Switch to specified provider
                match self.chat_state.switch_provider(provider) {
                    Ok(message) => {
                        self.repl.print_message(&self.formatter.format_success(&message));
                        
                        // Update the configuration to reflect the new provider/model
                        self.update_config_from_state().await?;
                    }
                    Err(error) => {
                        self.repl.print_message(&self.formatter.format_error(&error));
                    }
                }
            }
            None => {
                // Show current provider and status
                self.repl.print_message(&self.chat_state.status());
            }
        }
        Ok(())
    }

    /// Update configuration to reflect current chat state
    async fn update_config_from_state(&self) -> Result<()> {
        use crate::args::Provider;
        use crate::config::model::keys::ConfigKeys;
        use crate::config::service::config_service;

        let provider = match self.chat_state.provider.as_str() {
            "claude" => Provider::Claude,
            "openai" => Provider::Openai,
            _ => return Err(anyhow!("Unknown provider: {}", self.chat_state.provider)),
        };

        // Update provider in config
        config_service::write_config(
            self.config_repo,
            &ConfigKeys::ProviderKey.to_key(),
            provider.to_str(),
        )?;

        Ok(())
    }
}
