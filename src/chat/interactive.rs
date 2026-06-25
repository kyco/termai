use anyhow::{anyhow, Result};
use chrono::Local;
use std::io::Write;
use std::path::Path;

use crate::chat::commands::{ChatCommand, InputType};
use crate::chat::formatter::ChatFormatter;
use crate::chat::repl::ChatRepl;
use crate::chat::state::ChatState;
use crate::config::repository::ConfigRepository;
use crate::config::settings::{ResolvedSettings, SettingsOverrides, SettingsProvider, UserConfig};
use crate::llm::common::model::role::Role;
use crate::path::extract::extract_content;
use crate::path::model::Files;
//use crate::session::model::message::Message;
use crate::repository::db::SqliteRepository;
use crate::session::model::session::Session;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::session::service::sessions_service;
use crate::ui::timer::ThinkingTimer;

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
    /// A message/command supplied on the command line to run as the first turn.
    initial_input: Option<String>,
}

/// The result of attempting to generate one AI turn.
enum TurnOutcome {
    Completed,
    Cancelled,
    Failed(anyhow::Error),
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
        initial_input: Option<String>,
    ) -> Result<Self> {
        let repl = ChatRepl::new()?;
        let formatter = ChatFormatter::new();

        // Initialize chat state with current provider and model from config
        let chat_state = Self::initialize_chat_state(sqlite_repo)?;

        let mut session = Self {
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
            initial_input,
        };
        session.update_prompt();
        Ok(session)
    }

    /// Rebuild the REPL prompt so it always reflects the active session and a
    /// `*` marker when there are in-memory messages not yet on disk.
    fn update_prompt(&mut self) {
        let label = if self.session.temporary {
            "temporary".to_string()
        } else {
            self.session.name.clone()
        };
        let unsaved = self.has_unsaved_messages();
        let marker = if unsaved { "*" } else { "" };
        self.repl
            .set_prompt(format!("\x1b[36m{}\x1b[0m{} ❯ ", label, marker));
    }

    /// True when the conversation holds messages that are not yet persisted:
    /// either the session is temporary (nothing is on disk) or a message still
    /// has an unassigned id.
    fn has_unsaved_messages(&self) -> bool {
        (self.session.temporary && !self.session.messages.is_empty())
            || self.session.messages.iter().any(|m| m.id.is_empty())
    }

    /// Start the interactive chat session
    pub async fn run(&mut self) -> Result<()> {
        // Show welcome message
        self.display_welcome();

        // Show initial context info if any
        if !self.context_files.is_empty() {
            self.display_context_info();
        }

        // Run a command-line-supplied first turn (e.g. `termai chat "question"`).
        if let Some(initial) = self.initial_input.take() {
            if let Err(e) = self.process_input(&initial).await {
                self.repl
                    .print_message(&self.formatter.format_error(&e.to_string()));
            }
        }

        // Main chat loop
        loop {
            if self.should_exit {
                break;
            }

            match self.repl.read_line() {
                Ok(input) => {
                    // Only a real (non-empty) line disarms the exit prompt, so a
                    // stray Enter between two Ctrl+C presses doesn't silently
                    // cancel "press again to exit".
                    if !input.trim().is_empty() {
                        self.ctrl_c_pressed = false;
                    }

                    if let Err(e) = self.process_input(&input).await {
                        self.repl
                            .print_message(&self.formatter.format_error(&e.to_string()));
                    }
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
                        // Unrelated readline error: clear the exit arm so it
                        // can't survive across error frames and trigger a
                        // surprise single-Ctrl+C exit later.
                        self.ctrl_c_pressed = false;
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
            InputType::UnknownCommand(verb) => {
                self.repl
                    .print_message(&self.formatter.format_warning(&format!(
                        "Unknown command: {}. Type /help or ? to see available commands.",
                        verb
                    )));
                Ok(())
            }
        }
    }

    /// Handle slash commands
    async fn handle_command(&mut self, command: ChatCommand) -> Result<()> {
        match command {
            ChatCommand::Help => {
                let help_text = self.formatter.format_help(&ChatCommand::all_commands());
                self.repl.print_message(&help_text);
            }
            ChatCommand::Commands => {
                let palette = ChatCommand::command_palette();
                let palette_text = self.formatter.format_command_palette(&palette);
                self.repl.print_message(&palette_text);
            }
            ChatCommand::Save(name) => {
                self.handle_save_command(name)?;
            }
            ChatCommand::NewSession(name) => {
                self.handle_new_session_command(name)?;
            }
            ChatCommand::ListSessions => {
                self.handle_list_sessions_command()?;
            }
            ChatCommand::LoadSession(name) => {
                self.handle_load_session_command(name)?;
            }
            ChatCommand::RenameSession(name) => {
                self.handle_rename_session_command(name)?;
            }
            ChatCommand::Context => {
                self.display_context_info();
            }
            ChatCommand::Clear => {
                self.handle_clear_command()?;
            }
            ChatCommand::Exit | ChatCommand::Quit => {
                self.should_exit = true;
            }
            ChatCommand::Retry => {
                if let Some(last_message) = self.session.messages.last() {
                    if last_message.role == Role::Assistant {
                        // Drop the last AI response and regenerate from the
                        // preceding (raw) user message.
                        self.session.messages.pop();
                        if let Some(user_message) = self.session.messages.last() {
                            if user_message.role == Role::User {
                                let content = user_message.content.clone();
                                let ok = self.generate_ai_response(&content).await?;
                                if ok {
                                    // Re-sync the stored copy so the discarded
                                    // assistant reply doesn't linger in the DB.
                                    self.resync_persisted_history()?;
                                }
                                self.update_prompt();
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
            ChatCommand::Tools(setting) => {
                self.handle_tools_command(setting);
            }
            ChatCommand::Status => {
                self.repl.print_message(&self.chat_state.status());
            }
            ChatCommand::Theme(theme_name) => {
                self.handle_theme_command(theme_name);
            }
            ChatCommand::Streaming(setting) => {
                self.handle_streaming_command(setting);
            }
            ChatCommand::Settings => {
                self.display_settings_overview();
            }
        }
        Ok(())
    }

    /// A user-facing label for the active session.
    fn session_label(&self) -> String {
        if self.session.temporary {
            "temporary (unsaved)".to_string()
        } else {
            self.session.name.clone()
        }
    }

    /// `/save [name]` — persist the conversation, promoting a temporary session
    /// to a real one on first save (the fix for "I can't save an ad-hoc chat").
    fn handle_save_command(&mut self, name: Option<String>) -> Result<()> {
        if self.session.messages.is_empty() {
            self.repl.print_message(
                &self
                    .formatter
                    .format_warning("Nothing to save yet — send a message first."),
            );
            return Ok(());
        }

        if self.session.temporary {
            let session_name =
                name.unwrap_or_else(|| format!("chat_{}", Local::now().format("%Y%m%d_%H%M%S")));
            if sessions_service::session_exists(self.session_repo, &session_name) {
                self.repl.print_message(&self.formatter.format_warning(&format!(
                    "A session named '{}' already exists. Pick another name, or /load {} to continue it.",
                    session_name, session_name
                )));
                return Ok(());
            }
            sessions_service::promote_session(
                self.session_repo,
                self.message_repo,
                &mut self.session,
                &session_name,
            )?;
            self.repl
                .print_message(&self.formatter.format_session_saved(&session_name));
        } else {
            // Already persistent: a new name renames; otherwise just flush.
            if let Some(new_name) = name {
                if new_name != self.session.name {
                    if let Err(e) = sessions_service::rename_session(
                        self.session_repo,
                        &mut self.session,
                        &new_name,
                    ) {
                        self.repl
                            .print_message(&self.formatter.format_warning(&e.to_string()));
                        return Ok(());
                    }
                }
            }
            sessions_service::session_add_messages(
                self.session_repo,
                self.message_repo,
                &mut self.session,
            )?;
            self.repl
                .print_message(&self.formatter.format_session_saved(&self.session.name));
        }
        self.update_prompt();
        Ok(())
    }

    /// `/new [name]` — auto-save the current conversation, then swap to a blank
    /// slate (provider/model and context selections are retained).
    fn handle_new_session_command(&mut self, name: Option<String>) -> Result<()> {
        if let Some(n) = &name {
            if sessions_service::session_exists(self.session_repo, n) {
                self.repl
                    .print_message(&self.formatter.format_warning(&format!(
                        "A session named '{}' already exists. Use /load {} to continue it.",
                        n, n
                    )));
                return Ok(());
            }
        }

        self.autosave_current()?;

        self.session = match name {
            Some(n) => sessions_service::session(self.session_repo, self.message_repo, &n)?,
            None => Session::new_temporary(),
        };

        self.repl.clear_screen();
        self.display_welcome();
        self.update_prompt();
        self.repl.print_message(
            &self
                .formatter
                .format_success(&format!("Started new session: {}", self.session_label())),
        );
        Ok(())
    }

    /// `/sessions` — list saved sessions with message counts and active marker.
    fn handle_list_sessions_command(&mut self) -> Result<()> {
        let sessions = sessions_service::list_sessions(self.session_repo, self.message_repo)?;
        let listing =
            self.formatter
                .format_session_list(&sessions, &self.session.id, self.session.temporary);
        self.repl.print_message(&listing);
        Ok(())
    }

    /// `/load <name>` (aka `/switch`) — swap another saved session's history in.
    fn handle_load_session_command(&mut self, name: String) -> Result<()> {
        let name = name.trim().to_string();
        if name.is_empty() {
            self.repl.print_message(
                &self
                    .formatter
                    .format_warning("Usage: /load <name>   (run /sessions to see saved sessions)"),
            );
            return Ok(());
        }
        if !self.session.temporary && name == self.session.name {
            self.repl.print_message(
                &self
                    .formatter
                    .format_warning(&format!("Already in session '{}'", name)),
            );
            return Ok(());
        }
        if !sessions_service::session_exists(self.session_repo, &name) {
            self.repl
                .print_message(&self.formatter.format_error(&format!(
                    "Session '{}' not found. Run /sessions to see saved sessions.",
                    name
                )));
            return Ok(());
        }

        self.autosave_current()?;

        let loaded = sessions_service::load_session(self.session_repo, self.message_repo, &name)?;
        self.session = loaded;
        self.session.redaction_mapping = None;

        self.repl.clear_screen();
        self.display_welcome();
        self.replay_history();
        self.update_prompt();
        self.repl.print_message(
            &self
                .formatter
                .format_success(&format!("Switched to session '{}'", name)),
        );
        Ok(())
    }

    /// `/rename <name>` — rename the active session (saves it first if it was a
    /// temporary/unsaved session).
    fn handle_rename_session_command(&mut self, name: String) -> Result<()> {
        let name = name.trim().to_string();
        if name.is_empty() {
            self.repl
                .print_message(&self.formatter.format_warning("Usage: /rename <new-name>"));
            return Ok(());
        }

        if self.session.temporary {
            if self.session.messages.is_empty() {
                self.repl.print_message(
                    &self
                        .formatter
                        .format_warning("Nothing to save yet — send a message first."),
                );
                return Ok(());
            }
            if sessions_service::session_exists(self.session_repo, &name) {
                self.repl.print_message(
                    &self
                        .formatter
                        .format_warning(&format!("A session named '{}' already exists.", name)),
                );
                return Ok(());
            }
            sessions_service::promote_session(
                self.session_repo,
                self.message_repo,
                &mut self.session,
                &name,
            )?;
            self.repl
                .print_message(&self.formatter.format_session_saved(&name));
        } else {
            match sessions_service::rename_session(self.session_repo, &mut self.session, &name) {
                Ok(()) => self.repl.print_message(
                    &self
                        .formatter
                        .format_success(&format!("Renamed session to '{}'", name)),
                ),
                Err(e) => self
                    .repl
                    .print_message(&self.formatter.format_warning(&e.to_string())),
            }
        }
        self.update_prompt();
        Ok(())
    }

    /// `/clear` — wipe the conversation. For a persisted session this also
    /// deletes the stored messages so cleared history does not reappear on
    /// reload.
    fn handle_clear_command(&mut self) -> Result<()> {
        if !self.session.temporary {
            self.message_repo
                .delete_messages_for_session(&self.session.id)
                .map_err(|e| anyhow!("Failed to clear stored messages: {:?}", e))?;
        }
        self.session.messages.clear();
        self.session.redaction_mapping = None;
        self.repl.clear_screen();
        self.display_welcome();
        self.update_prompt();
        self.repl
            .print_message(&self.formatter.format_conversation_cleared());
        Ok(())
    }

    /// Auto-save the current conversation so /new, /load, and /exit never lose
    /// work. Temporary sessions are promoted under an auto_save_<timestamp> name.
    fn autosave_current(&mut self) -> Result<()> {
        if self.session.messages.is_empty() {
            return Ok(());
        }
        if self.session.temporary {
            let base = format!("auto_save_{}", Local::now().format("%Y%m%d_%H%M%S"));
            let name = if sessions_service::session_exists(self.session_repo, &base) {
                format!(
                    "{}_{}",
                    base,
                    &self.session.id[..8.min(self.session.id.len())]
                )
            } else {
                base
            };
            sessions_service::promote_session(
                self.session_repo,
                self.message_repo,
                &mut self.session,
                &name,
            )?;
            self.repl.print_message(
                &self
                    .formatter
                    .format_success(&format!("Auto-saved current conversation as '{}'", name)),
            );
        } else {
            sessions_service::session_add_messages(
                self.session_repo,
                self.message_repo,
                &mut self.session,
            )?;
        }
        Ok(())
    }

    /// Rewrite the persisted message rows to exactly match the in-memory
    /// conversation. Used after `/retry` so a discarded assistant reply doesn't
    /// remain in the database. No-op for unsaved temporary sessions.
    fn resync_persisted_history(&mut self) -> Result<()> {
        if self.session.temporary {
            return Ok(());
        }
        self.message_repo
            .delete_messages_for_session(&self.session.id)
            .map_err(|e| anyhow!("Failed to resync stored messages: {:?}", e))?;
        for message in self.session.messages.iter_mut() {
            message.id = String::new();
        }
        sessions_service::session_add_messages(
            self.session_repo,
            self.message_repo,
            &mut self.session,
        )?;
        Ok(())
    }

    /// Replay a loaded session's prior messages (user/assistant turns only).
    fn replay_history(&self) {
        for message in &self.session.messages {
            if message.role == Role::System {
                continue;
            }
            let formatted = self
                .formatter
                .format_message(&message.role, &message.content, None);
            self.repl.print_message(&formatted);
        }
    }

    /// Handle /tools command - toggle or set tool usage
    fn handle_tools_command(&mut self, setting: Option<bool>) {
        let wants_enable = match setting {
            Some(v) => v,
            None => !self.chat_state.tools_enabled,
        };

        // Tools are only wired for the OpenAI provider; enabling them under
        // claude/codex would be a silent no-op, so refuse rather than report a
        // misleading success.
        if wants_enable && self.chat_state.provider != "openai" {
            self.repl.print_message(&self.formatter.format_warning(&format!(
                "Tools are only supported with the OpenAI provider (current: {}). Run /provider openai first.",
                self.chat_state.provider
            )));
            return;
        }

        self.chat_state.set_tools_enabled(wants_enable);

        let detail = if self.chat_state.tools_enabled {
            " The AI can execute bash commands, read/write files, and list directories."
        } else {
            ""
        };
        let status = if self.chat_state.tools_enabled {
            "enabled"
        } else {
            "disabled"
        };
        self.repl.print_message(
            &self
                .formatter
                .format_success(&format!("Tools are now {}.{}", status, detail)),
        );
    }

    /// Handle regular chat messages
    async fn handle_message(&mut self, message: String) -> Result<()> {
        // Add user message to session (rustyline already echoed the input).
        self.session.add_raw_message(message.clone(), Role::User);

        let succeeded = self.generate_ai_response(&message).await?;
        if !succeeded {
            // The turn failed or was cancelled — drop the user message we just
            // added so the conversation (and any later save) stays consistent.
            if let Some(last) = self.session.messages.last() {
                if last.role == Role::User {
                    self.session.messages.pop();
                }
            }
        }
        self.update_prompt();
        Ok(())
    }

    /// Generate an AI response for the given (raw) user input. Returns whether a
    /// response was successfully produced. Never pops the user message itself —
    /// that ownership belongs to the caller.
    async fn generate_ai_response(&mut self, user_input: &str) -> Result<bool> {
        let mut timer = ThinkingTimer::new();
        timer.start();

        // Inject file context only into the wire payload — never into the stored
        // conversation — so context isn't baked in and re-appended every turn.
        let mut raw_backup: Option<String> = None;
        if !self.context_files.is_empty() {
            let augmented = self.create_contextual_input(user_input);
            if let Some(last_msg) = self.session.messages.last_mut() {
                if last_msg.role == Role::User {
                    raw_backup = Some(last_msg.content.clone());
                    last_msg.content = augmented;
                }
            }
        }

        // Redact sensitive information before it goes over the wire.
        self.session.redact(self.config_repo);

        let outcome = self.call_ai_service_cancellable().await;

        timer.stop();

        // Ensure the thinking indicator is fully cleared before output.
        print!("\r\x1b[2K");
        std::io::stdout().flush().unwrap();

        // Restore real content, then revert the augmented user message back to
        // the raw text the user typed — so persistence stores real, un-bloated
        // history (not redaction placeholders or duplicated context).
        self.session.unredact();
        if let Some(raw) = raw_backup {
            if let Some(idx) = self
                .session
                .messages
                .iter()
                .rposition(|m| m.role == Role::User)
            {
                self.session.messages[idx].content = raw;
            }
        }

        match outcome {
            TurnOutcome::Completed => {
                if let Some(last_message) = self.session.messages.last() {
                    if last_message.role == Role::Assistant {
                        if let Err(e) = self
                            .formatter
                            .format_message_async(
                                &Role::Assistant,
                                &last_message.content,
                                Some(Local::now()),
                            )
                            .await
                        {
                            eprintln!("Error formatting AI response: {}", e);
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

                // Incrementally persist (no-op for an unsaved temporary session
                // until the user /saves it; flush for a persisted session).
                sessions_service::session_add_messages(
                    self.session_repo,
                    self.message_repo,
                    &mut self.session,
                )?;
                std::io::stdout().flush().unwrap();
                Ok(true)
            }
            TurnOutcome::Cancelled => {
                self.repl
                    .print_message(&self.formatter.format_warning("Generation cancelled."));
                std::io::stdout().flush().unwrap();
                Ok(false)
            }
            TurnOutcome::Failed(e) => {
                self.repl
                    .print_message(&self.formatter.format_error(&format!("AI Error: {}", e)));
                std::io::stdout().flush().unwrap();
                Ok(false)
            }
        }
    }

    /// Run the AI request, but let Ctrl+C cancel an in-flight generation and
    /// drop the user back to the prompt instead of waiting (or killing the
    /// whole process).
    async fn call_ai_service_cancellable(&mut self) -> TurnOutcome {
        tokio::select! {
            result = self.call_ai_service() => match result {
                Ok(()) => TurnOutcome::Completed,
                Err(e) => TurnOutcome::Failed(e),
            },
            _ = tokio::signal::ctrl_c() => TurnOutcome::Cancelled,
        }
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
                claude::service::chat::chat_with_model(
                    &api_key.value,
                    &mut self.session,
                    Some(&self.chat_state.model),
                )
                .await?;
            }
            "openai" => {
                let api_key = config_service::fetch_by_key(
                    self.config_repo,
                    &ConfigKeys::ChatGptApiKey.to_key(),
                )?;
                if self.chat_state.tools_enabled {
                    openai::service::chat::chat_with_tools(&api_key.value, &mut self.session)
                        .await?;
                } else {
                    openai::service::chat::chat_with_model(
                        &api_key.value,
                        &mut self.session,
                        Some(&self.chat_state.model),
                    )
                    .await?;
                }
            }
            "openai-codex" | "openai_codex" | "codex" => {
                use crate::auth::token_manager::TokenManager;

                // Get valid access token (auto-refreshes if needed)
                let token_manager = TokenManager::new(self.config_repo);
                let access_token = token_manager
                    .get_valid_token()
                    .await?
                    .ok_or_else(|| anyhow!(
                        "Not authenticated with Codex. Run 'termai auth login codex' to authenticate."
                    ))?;

                openai::service::codex::chat(
                    &access_token,
                    &mut self.session,
                    Some(&self.chat_state.model),
                )
                .await?;
            }
            _ => {
                return Err(anyhow!(
                    "Unsupported provider: {}",
                    self.chat_state.provider
                ));
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

    /// Display welcome message + a one-line banner showing the active session.
    fn display_welcome(&self) {
        println!(); // Add spacing before welcome
        self.repl.print_message(&self.formatter.format_welcome());
        self.repl
            .print_message(&self.formatter.format_session_banner(
                &self.session_label(),
                self.session.messages.len(),
                &self.chat_state.provider,
                &self.chat_state.model,
            ));
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

    /// Save session and history when exiting. Auto-saves an unsaved conversation
    /// so nothing is ever lost on exit.
    async fn save_on_exit(&mut self) -> Result<()> {
        self.repl.save_history()?;
        self.autosave_current()?;
        Ok(())
    }

    /// Handle the /branch command. In-chat branching isn't implemented yet;
    /// be honest about it and point at the working CLI path rather than
    /// pretending to create a branch.
    async fn handle_branch_command(&mut self, _name: Option<String>) -> Result<()> {
        let save_hint = if self.session.temporary {
            " Save it first with /save <name>."
        } else {
            ""
        };
        self.repl.print_message(&self.formatter.format_warning(&format!(
            "Branching isn't available inside chat yet. Use `termai session branch {}` from the command line.{}",
            if self.session.temporary { "<name>" } else { &self.session.name },
            save_hint
        )));
        Ok(())
    }

    /// Handle /theme command
    fn handle_theme_command(&mut self, theme_name: Option<String>) {
        match theme_name {
            Some(name) => match self.formatter.set_theme(&name) {
                Ok(()) => {
                    self.repl.print_message(
                        &self
                            .formatter
                            .format_success(&format!("Switched to '{}' theme", name)),
                    );
                }
                Err(e) => {
                    self.repl.print_message(&self.formatter.format_error(&e));
                    let themes = self.formatter.available_themes();
                    self.repl
                        .print_message(&format!("Available themes: {}", themes.join(", ")));
                }
            },
            None => {
                let themes = self.formatter.available_themes();
                self.repl.print_message(&format!(
                    "Available themes: {}\nUse '/theme <name>' to switch",
                    themes.join(", ")
                ));
            }
        }
    }

    /// Handle /streaming command — explicit on/off, or toggle when no argument.
    fn handle_streaming_command(&mut self, setting: Option<bool>) {
        let enabled = match setting {
            Some(v) => v,
            None => !self.formatter.is_streaming(),
        };
        self.formatter.set_streaming(enabled);
        let status = if enabled { "enabled" } else { "disabled" };
        self.repl.print_message(
            &self
                .formatter
                .format_success(&format!("Streaming output {}", status)),
        );
    }

    /// Display a settings overview panel
    fn display_settings_overview(&self) {
        let overview = self.formatter.format_settings_overview(
            &self.chat_state.provider,
            &self.chat_state.model,
            self.chat_state.tools_enabled,
            self.formatter.is_streaming(),
            self.context_files.len(),
            &self.session_label(),
        );
        self.repl.print_message(&overview);
    }

    /// Initialize chat state from current configuration
    fn initialize_chat_state(sqlite_repo: &SqliteRepository) -> Result<ChatState> {
        let settings = ResolvedSettings::load_for_current_dir_with_repo(
            sqlite_repo,
            SettingsOverrides::default(),
        )?;
        let chat_state = ChatState::new(
            settings.default_provider.as_str().to_string(),
            settings.selected_model(),
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
                        self.repl
                            .print_message(&self.formatter.format_success(&message));

                        // Update the configuration to reflect the new provider/model
                        self.update_config_from_state().await?;
                    }
                    Err(error) => {
                        self.repl
                            .print_message(&self.formatter.format_error(&error));
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
                        self.repl
                            .print_message(&self.formatter.format_success(&message));

                        // Update the configuration to reflect the new provider/model
                        self.update_config_from_state().await?;
                    }
                    Err(error) => {
                        self.repl
                            .print_message(&self.formatter.format_error(&error));
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
        let mut user_config = UserConfig::load()?;
        user_config.default.provider = match self.chat_state.provider.as_str() {
            "claude" => SettingsProvider::Claude,
            "openai" => SettingsProvider::Openai,
            "codex" | "openai-codex" | "openai_codex" => SettingsProvider::Codex,
            _ => return Err(anyhow!("Unknown provider: {}", self.chat_state.provider)),
        };
        user_config.default.model = Some(self.chat_state.model.clone());
        user_config.save()?;

        Ok(())
    }
}
