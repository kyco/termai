use anyhow::{anyhow, Result};
use chrono::Local;
use std::collections::VecDeque;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crossterm::event::{
    DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
    KeyModifiers,
};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::tty::IsTty;

use crate::branch::BranchService;
use crate::chat::anchor::{
    filter_palette, AnchorRenderer, AnchorState, PaletteItem, SpinnerInfo, StatusInfo,
    SPINNER_FRAMES,
};
use crate::chat::commands::{ChatCommand, InputType};
use crate::chat::editor::{EditorEvent, LineEditor};
use crate::chat::formatter::ChatFormatter;
use crate::chat::state::ChatState;
use crate::config::repository::ConfigRepository;
use crate::config::settings::{ResolvedSettings, SettingsOverrides, SettingsProvider, UserConfig};
use crate::llm::common::model::role::Role;
use crate::path::extract::extract_content;
use crate::path::model::Files;
use crate::repository::db::SqliteRepository;
use crate::session::model::session::Session;
use crate::session::repository::{MessageRepository, SessionRepository};
use crate::session::service::sessions_service;
use crate::ui::timer::ThinkingTimer;
use crate::ui::web_indicator::activity;

const HISTORY_FILE: &str = ".termai_history";

/// RAII guard for raw mode + bracketed paste. Always restores the terminal
/// on drop (including panics and error paths).
struct RawModeGuard {
    active: bool,
}

impl RawModeGuard {
    fn new() -> Result<Self> {
        enable_raw_mode()?;
        let _ = crossterm::execute!(std::io::stdout(), EnableBracketedPaste);
        Ok(Self { active: true })
    }

    /// Temporarily restore cooked mode (e.g. while printing formatted output).
    fn suspend(&mut self) {
        if self.active {
            let _ = crossterm::execute!(std::io::stdout(), DisableBracketedPaste);
            let _ = disable_raw_mode();
            self.active = false;
        }
    }

    fn resume(&mut self) {
        if !self.active {
            let _ = enable_raw_mode();
            let _ = crossterm::execute!(std::io::stdout(), EnableBracketedPaste);
            self.active = true;
        }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        self.suspend();
    }
}

/// Spawn a blocking thread that forwards crossterm events into a tokio
/// channel. The thread polls with a timeout so it can notice shutdown.
fn spawn_input_reader(shutdown: Arc<AtomicBool>) -> tokio::sync::mpsc::UnboundedReceiver<Event> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    std::thread::spawn(move || loop {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        match crossterm::event::poll(Duration::from_millis(100)) {
            Ok(true) => match crossterm::event::read() {
                Ok(ev) => {
                    if tx.send(ev).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            },
            Ok(false) => continue,
            Err(_) => break,
        }
    });
    rx
}

/// Terminal-facing state for the bottom-anchored UI: editor, anchor
/// renderer, palette state and the queue of messages submitted while a
/// response was streaming. Kept separate from `InteractiveSession` so the
/// AI-call future (which mutably borrows the session) never conflicts with
/// UI updates.
struct AnchorUi {
    editor: LineEditor,
    renderer: AnchorRenderer,
    guard: RawModeGuard,
    shutdown: Arc<AtomicBool>,
    queued: VecDeque<String>,
    /// Palette hidden via Esc until the buffer changes again.
    palette_hidden: bool,
    /// Entries pinned while Tab-cycling (so completing doesn't re-filter).
    palette_pin: Option<Vec<PaletteItem>>,
    palette_index: usize,
    /// First Ctrl+C on an empty buffer arms exit; the second exits.
    ctrl_c_armed: bool,
}

impl AnchorUi {
    fn new(guard: RawModeGuard, shutdown: Arc<AtomicBool>) -> Self {
        Self {
            editor: LineEditor::new(),
            renderer: AnchorRenderer::new(),
            guard,
            shutdown,
            queued: VecDeque::new(),
            palette_hidden: false,
            palette_pin: None,
            palette_index: 0,
            ctrl_c_armed: false,
        }
    }

    fn palette_items(&self) -> Vec<PaletteItem> {
        if self.palette_hidden {
            return Vec::new();
        }
        match &self.palette_pin {
            Some(pinned) => pinned.clone(),
            None => filter_palette(self.editor.buffer()),
        }
    }

    fn build_state(&self, status: &StatusInfo, spinner: Option<SpinnerInfo>) -> AnchorState {
        let palette = self.palette_items();
        let selected = if palette.is_empty() {
            0
        } else {
            self.palette_index.min(palette.len() - 1)
        };
        AnchorState {
            input: self.editor.buffer().to_string(),
            cursor: self.editor.cursor(),
            status: status.clone(),
            spinner,
            queued: self.queued.len(),
            palette,
            palette_selected: selected,
        }
    }

    fn draw(&mut self, status: &StatusInfo, spinner: Option<SpinnerInfo>) -> std::io::Result<()> {
        let state = self.build_state(status, spinner);
        let mut out = std::io::stdout();
        self.renderer.draw(&mut out, &state)
    }

    /// Print content into the scrollback above the anchor, then repaint.
    fn print_above(&mut self, content: &str, status: &StatusInfo) -> std::io::Result<()> {
        let state = self.build_state(status, None);
        let mut out = std::io::stdout();
        self.renderer.print_above(&mut out, content, &state)
    }

    /// Erase the anchor and drop to cooked mode so ordinary `println!`-based
    /// output (formatter, command handlers) renders correctly above.
    fn begin_suspended(&mut self) -> std::io::Result<()> {
        let mut out = std::io::stdout();
        self.renderer.erase(&mut out)?;
        self.guard.suspend();
        Ok(())
    }

    fn end_suspended(&mut self) {
        self.guard.resume();
    }

    /// The buffer changed through typing/paste: unpin the palette.
    fn on_buffer_change(&mut self) {
        self.palette_pin = None;
        self.palette_index = 0;
        self.palette_hidden = false;
        self.ctrl_c_armed = false;
    }

    /// Tab: complete to the selected palette entry; further Tabs cycle.
    fn cycle_palette(&mut self) {
        if self.palette_hidden {
            return;
        }
        let items = match &self.palette_pin {
            Some(pinned) => pinned.clone(),
            None => filter_palette(self.editor.buffer()),
        };
        if items.is_empty() {
            return;
        }
        match self.palette_pin {
            None => {
                self.palette_pin = Some(items.clone());
                self.palette_index = 0;
            }
            Some(_) => {
                self.palette_index = (self.palette_index + 1) % items.len();
            }
        }
        self.editor.set_text(&items[self.palette_index].completion);
    }

    /// Tear down: erase the anchor, stop the reader thread, restore cooked
    /// mode. Consumes self so the terminal is clean afterwards.
    fn close(mut self) {
        let mut out = std::io::stdout();
        let _ = self.renderer.erase(&mut out);
        self.shutdown.store(true, Ordering::SeqCst);
        self.guard.suspend();
    }
}

/// Outcome of one anchored AI turn.
enum TurnOutcome {
    Done(Result<()>),
    Cancelled,
    InputClosed,
}

fn is_ctrl_c(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL)
}

/// Manages an interactive chat session.
///
/// On a real TTY this runs the bottom-anchored UI (input pinned at the
/// bottom, conversation flowing into native scrollback, typing allowed while
/// a response is in flight). When stdin or stdout is piped it falls back to
/// a plain line-based loop so scripted/e2e usage keeps working.
pub struct InteractiveSession<'a, R, SR, MR>
where
    R: ConfigRepository,
    SR: SessionRepository,
    MR: MessageRepository,
{
    formatter: ChatFormatter,
    session: Session,
    config_repo: &'a R,
    session_repo: &'a SR,
    message_repo: &'a MR,
    #[allow(dead_code)]
    sqlite_repo: &'a SqliteRepository,
    context_files: Vec<Files>,
    should_exit: bool,
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
        let formatter = ChatFormatter::new();

        // Initialize chat state with current provider and model from config
        let chat_state = Self::initialize_chat_state(sqlite_repo)?;

        Ok(Self {
            formatter,
            session,
            config_repo,
            session_repo,
            message_repo,
            sqlite_repo,
            context_files,
            should_exit: false,
            chat_state,
        })
    }

    /// Start the interactive chat session
    pub async fn run(&mut self) -> Result<()> {
        let interactive_tty = std::io::stdin().is_tty() && std::io::stdout().is_tty();
        if interactive_tty {
            self.run_anchored().await
        } else {
            self.run_plain().await
        }
    }

    /// Print a message to the transcript (cooked-mode paths).
    fn say(&self, message: &str) {
        println!("{}", message);
    }

    // ------------------------------------------------------------------
    // Plain (non-TTY) fallback: read lines from stdin, print responses.
    // ------------------------------------------------------------------

    async fn run_plain(&mut self) -> Result<()> {
        self.display_welcome();
        if !self.context_files.is_empty() {
            self.display_context_info();
        }

        use std::io::BufRead;
        loop {
            if self.should_exit {
                break;
            }

            let mut line = String::new();
            let bytes = std::io::stdin().lock().read_line(&mut line)?;
            if bytes == 0 {
                // EOF
                break;
            }
            let input = line.trim_end_matches(['\n', '\r']).to_string();
            if let Err(e) = self.process_input(&input).await {
                self.say(&self.formatter.format_error(&e.to_string()));
            }
        }

        self.finish().await
    }

    /// Save session and print the goodbye message (shared by both modes).
    async fn finish(&mut self) -> Result<()> {
        self.save_on_exit().await?;
        self.say(
            &self
                .formatter
                .format_success("Chat session ended. Goodbye! 👋"),
        );
        Ok(())
    }

    // ------------------------------------------------------------------
    // Bottom-anchored TTY mode
    // ------------------------------------------------------------------

    async fn run_anchored(&mut self) -> Result<()> {
        // In anchored mode the response is painted instantly into scrollback:
        // the typewriter animation would hold the terminal in cooked mode for
        // seconds while the user may be typing their next message.
        self.formatter.set_streaming(false);
        self.formatter.set_show_role_labels(false);

        self.display_welcome();
        if !self.context_files.is_empty() {
            self.display_context_info();
        }

        let guard = RawModeGuard::new()?;
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut rx = spawn_input_reader(shutdown.clone());
        let mut ui = AnchorUi::new(guard, shutdown);

        if let Ok(content) = std::fs::read_to_string(HISTORY_FILE) {
            ui.editor.load_history(
                content
                    .lines()
                    .filter(|l| !l.starts_with('#'))
                    .map(String::from),
            );
        }

        activity::set_anchored(true);
        let result = self.anchored_loop(&mut ui, &mut rx).await;
        activity::set_anchored(false);

        let history = ui.editor.history().to_vec();
        ui.close();
        drop(rx);
        if !history.is_empty() {
            let _ = std::fs::write(HISTORY_FILE, history.join("\n") + "\n");
        }

        result?;
        self.finish().await
    }

    async fn anchored_loop(
        &mut self,
        ui: &mut AnchorUi,
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<Event>,
    ) -> Result<()> {
        loop {
            if self.should_exit {
                break;
            }

            let status = self.status_info();
            ui.draw(&status, None)?;

            let Some(event) = rx.recv().await else {
                break;
            };

            match event {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }
                    if key.code == KeyCode::Tab {
                        ui.cycle_palette();
                        continue;
                    }
                    match ui.editor.handle_key(key) {
                        EditorEvent::Submit(text) => {
                            ui.on_buffer_change();
                            self.process_submission(ui, rx, text).await?;
                            // Drain anything queued while the response streamed
                            while !self.should_exit {
                                match ui.queued.pop_front() {
                                    Some(next) => self.process_submission(ui, rx, next).await?,
                                    None => break,
                                }
                            }
                        }
                        EditorEvent::Cancel => {
                            if !ui.palette_items().is_empty() {
                                // Esc closes the palette until the buffer changes
                                ui.palette_hidden = true;
                                ui.palette_pin = None;
                            } else if is_ctrl_c(&key) {
                                if ui.ctrl_c_armed {
                                    self.should_exit = true;
                                } else {
                                    ui.ctrl_c_armed = true;
                                    let status = self.status_info();
                                    ui.print_above(
                                        &self.formatter.format_warning(
                                            "Press Ctrl+C again to exit, or type /exit to quit gracefully",
                                        ),
                                        &status,
                                    )?;
                                }
                            }
                        }
                        EditorEvent::Exit => {
                            self.should_exit = true;
                        }
                        EditorEvent::Redraw => {
                            ui.on_buffer_change();
                        }
                        EditorEvent::None => {}
                    }
                }
                Event::Paste(text) => {
                    ui.editor.insert_str(&text);
                    ui.on_buffer_change();
                }
                Event::Resize(_, _) => {
                    // Repainted at the top of the loop with the new width
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Handle one submitted line in anchored mode: echo it into scrollback,
    /// then run it as a slash command or an AI turn.
    async fn process_submission(
        &mut self,
        ui: &mut AnchorUi,
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<Event>,
        text: String,
    ) -> Result<()> {
        let text = text.trim().to_string();
        if text.is_empty() {
            return Ok(());
        }

        let status = self.status_info();
        ui.print_above(&format!("\x1b[1;32m  you ›\x1b[0m {}", text), &status)?;

        match InputType::classify(&text) {
            InputType::Command(ChatCommand::Retry) => {
                let mut retry_input: Option<String> = None;
                if self
                    .session
                    .messages
                    .last()
                    .map(|m| m.role == Role::Assistant)
                    .unwrap_or(false)
                {
                    self.session.messages.pop();
                    if let Some(user_msg) = self.session.messages.last() {
                        if user_msg.role == Role::User {
                            retry_input = Some(user_msg.content.clone());
                        }
                    }
                }
                match retry_input {
                    Some(content) => self.anchored_ai_turn(ui, rx, &content).await?,
                    None => {
                        let status = self.status_info();
                        ui.print_above(
                            &self.formatter.format_warning("No AI response to retry"),
                            &status,
                        )?;
                    }
                }
            }
            InputType::Command(command) => {
                // Command handlers print with `println!`: run them in cooked
                // mode with the anchor erased, then repaint.
                ui.begin_suspended()?;
                let result = self.handle_command(command).await;
                ui.end_suspended();
                if let Err(e) = result {
                    let status = self.status_info();
                    ui.print_above(&self.formatter.format_error(&e.to_string()), &status)?;
                }
            }
            InputType::Message(message) => {
                self.session.add_raw_message(message.clone(), Role::User);
                self.anchored_ai_turn(ui, rx, &message).await?;
            }
        }
        Ok(())
    }

    /// Run one AI turn while keeping the anchor alive: the user can keep
    /// typing (their input persists in the anchor), submissions are queued,
    /// and Esc cancels the in-flight request by dropping its future.
    async fn anchored_ai_turn(
        &mut self,
        ui: &mut AnchorUi,
        rx: &mut tokio::sync::mpsc::UnboundedReceiver<Event>,
        user_input: &str,
    ) -> Result<()> {
        // Fold context files into the outgoing message (mirrors plain mode)
        let input_with_context = self.create_contextual_input(user_input);
        if !self.context_files.is_empty() {
            if let Some(last_msg) = self.session.messages.last_mut() {
                if last_msg.role == Role::User {
                    last_msg.content = input_with_context;
                }
            }
        }
        self.session.redact(self.config_repo);

        // Snapshot status segments: the session is mutably borrowed by the
        // request future below, so the anchor repaints from this copy.
        let status = self.status_info();
        let started = Instant::now();
        let mut ticker = tokio::time::interval(Duration::from_millis(100));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        let outcome = {
            let fut = Self::call_ai(self.config_repo, &self.chat_state, &mut self.session);
            tokio::pin!(fut);
            loop {
                tokio::select! {
                    result = &mut fut => break TurnOutcome::Done(result),
                    _ = ticker.tick() => {
                        let _ = ui.draw(&status, Some(Self::spinner_info(started)));
                    }
                    event = rx.recv() => match event {
                        None => break TurnOutcome::InputClosed,
                        Some(Event::Key(key)) => {
                            if key.kind == KeyEventKind::Release {
                                continue;
                            }
                            match ui.editor.handle_key(key) {
                                EditorEvent::Submit(text) => {
                                    let text = text.trim().to_string();
                                    if !text.is_empty() {
                                        ui.queued.push_back(text);
                                    }
                                }
                                EditorEvent::Cancel => break TurnOutcome::Cancelled,
                                EditorEvent::Exit => {
                                    self.should_exit = true;
                                    break TurnOutcome::Cancelled;
                                }
                                _ => {
                                    let _ = ui.draw(&status, Some(Self::spinner_info(started)));
                                }
                            }
                        }
                        Some(Event::Paste(text)) => {
                            ui.editor.insert_str(&text);
                            let _ = ui.draw(&status, Some(Self::spinner_info(started)));
                        }
                        Some(_) => {}
                    },
                }
            }
        };

        match outcome {
            TurnOutcome::Done(Ok(())) => {
                // Paint the response into scrollback above the anchor.
                ui.begin_suspended()?;
                if let Some(last_message) = self.session.messages.last() {
                    if last_message.role == Role::Assistant {
                        println!("\x1b[1;35m  ai  ›\x1b[0m");
                        let content = last_message.content.clone();
                        if let Err(e) = self
                            .formatter
                            .format_message_async(&Role::Assistant, &content, Some(Local::now()))
                            .await
                        {
                            eprintln!("Error formatting AI response: {}", e);
                            println!("{}", content);
                        }
                        std::io::stdout().flush().ok();
                    }
                }
                ui.end_suspended();

                sessions_service::session_add_messages(
                    self.session_repo,
                    self.message_repo,
                    &self.session,
                )?;
            }
            TurnOutcome::Done(Err(e)) => {
                self.pop_trailing_user_message();
                let status = self.status_info();
                ui.print_above(
                    &self.formatter.format_error(&format!("AI Error: {}", e)),
                    &status,
                )?;
            }
            TurnOutcome::Cancelled => {
                // The request future was dropped above, aborting the HTTP
                // call. Remove the un-answered user message.
                self.pop_trailing_user_message();
                let status = self.status_info();
                ui.print_above("\x1b[2m  ✋ response cancelled\x1b[0m", &status)?;
            }
            TurnOutcome::InputClosed => {
                self.should_exit = true;
            }
        }

        self.session.unredact();
        Ok(())
    }

    fn pop_trailing_user_message(&mut self) {
        if self
            .session
            .messages
            .last()
            .map(|m| m.role == Role::User)
            .unwrap_or(false)
        {
            self.session.messages.pop();
        }
    }

    fn spinner_info(started: Instant) -> SpinnerInfo {
        let frame = (started.elapsed().as_millis() / 100) as usize % SPINNER_FRAMES.len();
        match activity::current() {
            Some((label, secs)) => SpinnerInfo {
                frame,
                elapsed_secs: secs,
                label: format!("🌐 {}", label),
            },
            None => SpinnerInfo {
                frame,
                elapsed_secs: started.elapsed().as_secs_f32(),
                label: "thinking".to_string(),
            },
        }
    }

    fn status_info(&self) -> StatusInfo {
        StatusInfo {
            model: self.chat_state.model.clone(),
            session: self.session.name.clone(),
            token_estimate: Self::estimate_tokens(&self.session),
            tools_enabled: self.chat_state.tools_enabled,
        }
    }

    /// Cheap token estimate (~4 chars per token) for the status line.
    fn estimate_tokens(session: &Session) -> usize {
        session
            .messages
            .iter()
            .map(|m| m.content.chars().count())
            .sum::<usize>()
            / 4
    }

    // ------------------------------------------------------------------
    // Shared input processing (plain mode + command handling)
    // ------------------------------------------------------------------

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
                self.say(&help_text);
            }
            ChatCommand::Commands => {
                let palette = ChatCommand::command_palette();
                let palette_text = self.formatter.format_command_palette(&palette);
                self.say(&palette_text);
            }
            ChatCommand::Save(name) => {
                let session_name = name
                    .unwrap_or_else(|| format!("chat_{}", Local::now().format("%Y%m%d_%H%M%S")));
                self.session.name = session_name.clone();
                sessions_service::session_add_messages(
                    self.session_repo,
                    self.message_repo,
                    &self.session,
                )?;
                self.say(&self.formatter.format_session_saved(&session_name));
            }
            ChatCommand::Context => {
                self.display_context_info();
            }
            ChatCommand::Clear => {
                self.session.messages.clear();
                print!("\x1B[2J\x1B[1;1H"); // Clear screen, home cursor
                std::io::stdout().flush().ok();
                self.display_welcome();
                self.say(&self.formatter.format_conversation_cleared());
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
                        self.say(&self.formatter.format_warning("No AI response to retry"));
                    }
                } else {
                    self.say(
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
                self.say(&self.chat_state.status());
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

    /// Handle /tools command - toggle or set tool usage
    fn handle_tools_command(&mut self, setting: Option<bool>) {
        match setting {
            Some(enabled) => {
                self.chat_state.set_tools_enabled(enabled);
            }
            None => {
                self.chat_state.toggle_tools();
            }
        }

        let status = if self.chat_state.tools_enabled {
            "enabled"
        } else {
            "disabled"
        };

        let provider_note = if self.chat_state.provider != "openai" {
            format!("\n⚠️  Note: Tools are only supported with the OpenAI provider. Current provider: {}", self.chat_state.provider)
        } else {
            String::new()
        };

        self.say(&self.formatter.format_success(&format!(
            "Tools are now {}. The AI can execute bash commands, read/write files, and list directories.{}",
            status,
            provider_note
        )));
    }

    /// Handle regular chat messages (plain mode)
    async fn handle_message(&mut self, message: String) -> Result<()> {
        // Add user message to session
        self.session.add_raw_message(message.clone(), Role::User);

        // Generate AI response
        self.generate_ai_response(&message).await?;

        Ok(())
    }

    /// Generate AI response for the given user input (plain mode)
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
        let result = Self::call_ai(self.config_repo, &self.chat_state, &mut self.session).await;

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
                        let content = last_message.content.clone();
                        if let Err(e) = self
                            .formatter
                            .format_message_async(&Role::Assistant, &content, Some(Local::now()))
                            .await
                        {
                            eprintln!("Error formatting AI response: {}", e);
                            // Fallback to basic formatting
                            let formatted_ai = self.formatter.format_message(
                                &Role::Assistant,
                                &content,
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
                    &self.session,
                )?;
            }
            Err(e) => {
                self.say(&self.formatter.format_error(&format!("AI Error: {}", e)));

                // Remove the failed user message to keep session clean
                self.pop_trailing_user_message();
            }
        }

        // Unredact for display
        self.session.unredact();

        // Ensure we return control properly
        std::io::stdout().flush().unwrap();

        Ok(())
    }

    /// Call the AI service for the configured provider.
    ///
    /// An associated function (not a method) so the anchored UI can poll this
    /// future while separately updating the input line from key events.
    async fn call_ai(config_repo: &R, chat_state: &ChatState, session: &mut Session) -> Result<()> {
        use crate::config::model::keys::ConfigKeys;
        use crate::config::service::config_service;
        use crate::llm::{claude, openai};

        match chat_state.provider.as_str() {
            "claude" => {
                let api_key =
                    config_service::fetch_by_key(config_repo, &ConfigKeys::ClaudeApiKey.to_key())?;
                claude::service::chat::chat_with_model(
                    &api_key.value,
                    session,
                    Some(&chat_state.model),
                )
                .await?;
            }
            "openai" => {
                let api_key =
                    config_service::fetch_by_key(config_repo, &ConfigKeys::ChatGptApiKey.to_key())?;
                if chat_state.tools_enabled {
                    openai::service::chat::chat_with_tools(&api_key.value, session).await?;
                } else {
                    openai::service::chat::chat_with_model(
                        &api_key.value,
                        session,
                        Some(&chat_state.model),
                    )
                    .await?;
                }
            }
            "openai-codex" | "openai_codex" | "codex" => {
                use crate::auth::token_manager::TokenManager;

                // Get valid access token (auto-refreshes if needed)
                let token_manager = TokenManager::new(config_repo);
                let access_token = token_manager
                    .get_valid_token()
                    .await?
                    .ok_or_else(|| anyhow!(
                        "Not authenticated with Codex. Run 'termai auth login codex' to authenticate."
                    ))?;

                openai::service::codex::chat(&access_token, session, Some(&chat_state.model), None)
                    .await?;
            }
            _ => {
                return Err(anyhow!("Unsupported provider: {}", chat_state.provider));
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
        let new_context = extract_content(&Some(path.to_string()), &[], &[]);

        if let Some(mut files) = new_context {
            // Remove duplicates and add new files
            for file in files.drain(..) {
                if !self.context_files.iter().any(|f| f.path == file.path) {
                    self.context_files.push(file);
                }
            }
            self.say(
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
            self.say(
                &self
                    .formatter
                    .format_success(&format!("Removed files matching '{}' from context", path)),
            );
            self.display_context_info();
        } else {
            self.say(
                &self
                    .formatter
                    .format_warning(&format!("No files matching '{}' found in context", path)),
            );
        }
    }

    /// Display welcome message
    fn display_welcome(&self) {
        println!(); // Add spacing before welcome
        self.say(&self.formatter.format_welcome());
        println!(); // Add spacing after welcome
    }

    /// Display current context information
    fn display_context_info(&self) {
        let file_paths: Vec<String> = self.context_files.iter().map(|f| f.path.clone()).collect();
        let context_info = self
            .formatter
            .format_context_info(file_paths.len(), &file_paths);
        self.say(&context_info);
    }

    /// Save session when exiting
    async fn save_on_exit(&mut self) -> Result<()> {
        // Auto-save session if it has messages and no name
        if !self.session.messages.is_empty() && self.session.name == "temporary" {
            let auto_name = format!("auto_save_{}", Local::now().format("%Y%m%d_%H%M%S"));
            self.session.name = auto_name.clone();
            sessions_service::session_add_messages(
                self.session_repo,
                self.message_repo,
                &self.session,
            )?;
            self.say(
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
            format!(
                "🌿 Would create branch '{}' from current conversation state",
                branch_name
            )
        } else {
            format!(
                "🌿 Would create auto-named branch '{}' from current conversation state",
                branch_name
            )
        };

        // Display the branch creation message
        self.say(&self.formatter.format_success(&message));

        // Show branch creation info
        let info_lines = vec![
            "📋 Branch would include:".to_string(),
            format!(
                "   • {} messages from current conversation",
                self.session.messages.len()
            ),
            "   • Full conversation context preserved".to_string(),
            "   • Ready for exploring alternative approaches".to_string(),
        ];

        for line in info_lines {
            println!("  {}", line); // Simple formatting for info lines
        }

        // TODO: Actually create the branch when we have mutable access to repo
        // For now, this demonstrates the UI and command structure
        self.say(&self.formatter.format_warning(
            "⚠️  Branch creation temporarily disabled - requires mutable database access",
        ));

        Ok(())
    }

    /// Handle /theme command
    fn handle_theme_command(&mut self, theme_name: Option<String>) {
        match theme_name {
            Some(name) => match self.formatter.set_theme(&name) {
                Ok(()) => {
                    self.say(
                        &self
                            .formatter
                            .format_success(&format!("Switched to '{}' theme", name)),
                    );
                }
                Err(e) => {
                    self.say(&self.formatter.format_error(&e));
                    let themes = self.formatter.available_themes();
                    self.say(&format!("Available themes: {}", themes.join(", ")));
                }
            },
            None => {
                let themes = self.formatter.available_themes();
                self.say(&format!(
                    "Available themes: {}\nUse '/theme <name>' to switch",
                    themes.join(", ")
                ));
            }
        }
    }

    /// Handle /streaming command
    fn handle_streaming_command(&mut self, setting: Option<bool>) {
        match setting {
            Some(enabled) => {
                self.formatter.set_streaming(enabled);
                let status = if enabled { "enabled" } else { "disabled" };
                self.say(
                    &self
                        .formatter
                        .format_success(&format!("Streaming output {}", status)),
                );
            }
            None => {
                // Toggle: we don't track the current state externally, so just
                // tell the user how to use the command
                self.say(
                    "Usage: /streaming on  - enable streaming output\n       /streaming off - disable streaming output",
                );
            }
        }
    }

    /// Display a settings overview panel
    fn display_settings_overview(&self) {
        let overview = self.formatter.format_settings_overview(
            &self.chat_state.provider,
            &self.chat_state.model,
            self.chat_state.tools_enabled,
            true, // streaming default
            self.context_files.len(),
            &self.session.name,
        );
        self.say(&overview);
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
                        self.say(&self.formatter.format_success(&message));

                        // Update the configuration to reflect the new provider/model
                        self.update_config_from_state().await?;
                    }
                    Err(error) => {
                        self.say(&self.formatter.format_error(&error));
                    }
                }
            }
            None => {
                // Show current model and available models
                self.say(&self.chat_state.status());
                println!();
                self.say(&self.chat_state.list_models());
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
                        self.say(&self.formatter.format_success(&message));

                        // Update the configuration to reflect the new provider/model
                        self.update_config_from_state().await?;
                    }
                    Err(error) => {
                        self.say(&self.formatter.format_error(&error));
                    }
                }
            }
            None => {
                // Show current provider and status
                self.say(&self.chat_state.status());
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
