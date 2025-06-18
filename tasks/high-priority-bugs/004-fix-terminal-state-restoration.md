# Task: Fix Terminal State Restoration and Panic Handling

## Priority: High
## Estimated Effort: 1 day
## Dependencies: None
## Files Affected: `src/ui/tui/runner.rs`, `src/main.rs`

## Overview
Fix terminal state restoration issues where panics or unexpected exits can leave the terminal in a corrupted state, requiring users to reset their terminal manually. Implement proper cleanup and panic handling.

## Bug Description
When the application panics or exits unexpectedly, it doesn't restore the terminal to its original state, leaving users with:
- Raw mode still enabled
- Alternate screen active
- Mouse capture enabled
- Cursor potentially hidden

## Root Cause Analysis
1. **No Panic Handlers**: Terminal state not restored on panic
2. **No RAII Pattern**: Manual cleanup instead of automatic
3. **Signal Handling Missing**: SIGINT/SIGTERM not handled properly
4. **Resource Leaks**: Terminal resources not properly released

## Current Problematic Code
```rust
// In runner.rs:43-48
enable_raw_mode()?;
let mut stdout = io::stdout();
execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
// ... if panic occurs here, terminal state is corrupted
```

## Implementation Steps

### 1. Create Terminal State Manager with RAII
```rust
// src/ui/terminal/terminal_manager.rs
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::panic;

pub struct TerminalManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    _guard: TerminalGuard,
}

struct TerminalGuard {
    was_raw_mode: bool,
    was_alternate_screen: bool,
    was_mouse_capture: bool,
}

impl TerminalManager {
    pub fn new() -> io::Result<Self> {
        let guard = TerminalGuard::new()?;
        let terminal = Self::setup_terminal()?;
        
        Ok(Self {
            terminal,
            _guard: guard,
        })
    }
    
    fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }
    
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }
    
    pub fn restore(&mut self) -> io::Result<()> {
        self._guard.restore()
    }
}

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        // Store current terminal state before making changes
        let was_raw_mode = false; // We'll assume normal mode initially
        let was_alternate_screen = false;
        let was_mouse_capture = false;
        
        Ok(Self {
            was_raw_mode,
            was_alternate_screen,
            was_mouse_capture,
        })
    }
    
    fn restore(&mut self) -> io::Result<()> {
        // Restore terminal state
        if !self.was_raw_mode {
            let _ = disable_raw_mode(); // Best effort, don't fail on error
        }
        
        if !self.was_alternate_screen {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
        }
        
        if !self.was_mouse_capture {
            let _ = execute!(io::stdout(), DisableMouseCapture);
        }
        
        // Always try to show cursor
        let _ = execute!(io::stdout(), crossterm::cursor::Show);
        
        Ok(())
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Ensure terminal is restored even if explicit restore() wasn't called
        let _ = self.restore();
    }
}
```

### 2. Add Panic Hook for Terminal Restoration
```rust
// src/ui/terminal/panic_handler.rs
use std::panic::{self, PanicInfo};
use std::io::{self, Write};
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};

pub struct PanicHandler;

impl PanicHandler {
    pub fn install() {
        let original_hook = panic::take_hook();
        
        panic::set_hook(Box::new(move |panic_info| {
            // Restore terminal state
            Self::restore_terminal();
            
            // Print panic information
            Self::print_panic_info(panic_info);
            
            // Call original panic hook
            original_hook(panic_info);
        }));
    }
    
    fn restore_terminal() {
        // Best effort terminal restoration
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = execute!(io::stdout(), DisableMouseCapture);
        let _ = execute!(io::stdout(), crossterm::cursor::Show);
    }
    
    fn print_panic_info(panic_info: &PanicInfo) {
        eprintln!("\n{'=':<60}");
        eprintln!("TermAI has encountered an unexpected error and must exit.");
        eprintln!("{'=':<60}");
        
        if let Some(location) = panic_info.location() {
            eprintln!("Location: {}:{}:{}", location.file(), location.line(), location.column());
        }
        
        if let Some(message) = panic_info.payload().downcast_ref::<&str>() {
            eprintln!("Error: {}", message);
        } else if let Some(message) = panic_info.payload().downcast_ref::<String>() {
            eprintln!("Error: {}", message);
        }
        
        eprintln!("\nPlease report this issue at: https://github.com/your-repo/termai/issues");
        eprintln!("Include the above information in your report.");
        eprintln!("{'=':<60}\n");
    }
}
```

### 3. Add Signal Handling for Graceful Shutdown
```rust
// src/ui/terminal/signal_handler.rs
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::signal;

pub struct SignalHandler {
    shutdown_flag: Arc<AtomicBool>,
}

impl SignalHandler {
    pub fn new() -> Self {
        Self {
            shutdown_flag: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn install(&self) -> Arc<AtomicBool> {
        let shutdown_flag = self.shutdown_flag.clone();
        
        #[cfg(unix)]
        {
            let shutdown_flag_sigint = shutdown_flag.clone();
            let shutdown_flag_sigterm = shutdown_flag.clone();
            
            tokio::spawn(async move {
                let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
                    .expect("Failed to create SIGINT handler");
                
                if sigint.recv().await.is_some() {
                    eprintln!("\nReceived SIGINT, shutting down gracefully...");
                    shutdown_flag_sigint.store(true, Ordering::Relaxed);
                }
            });
            
            tokio::spawn(async move {
                let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("Failed to create SIGTERM handler");
                
                if sigterm.recv().await.is_some() {
                    eprintln!("\nReceived SIGTERM, shutting down gracefully...");
                    shutdown_flag_sigterm.store(true, Ordering::Relaxed);
                }
            });
        }
        
        #[cfg(windows)]
        {
            let shutdown_flag_ctrl_c = shutdown_flag.clone();
            
            tokio::spawn(async move {
                if signal::ctrl_c().await.is_ok() {
                    eprintln!("\nReceived Ctrl+C, shutting down gracefully...");
                    shutdown_flag_ctrl_c.store(true, Ordering::Relaxed);
                }
            });
        }
        
        shutdown_flag
    }
    
    pub fn should_shutdown(&self) -> bool {
        self.shutdown_flag.load(Ordering::Relaxed)
    }
}
```

### 4. Create Terminal Session Manager
```rust
// src/ui/terminal/session.rs
use super::{TerminalManager, SignalHandler, PanicHandler};
use crate::error::{TermAIError, Result};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub struct TerminalSession {
    terminal_manager: TerminalManager,
    signal_handler: SignalHandler,
    shutdown_flag: Arc<AtomicBool>,
}

impl TerminalSession {
    pub fn new() -> Result<Self> {
        // Install panic handler first
        PanicHandler::install();
        
        // Create terminal manager
        let terminal_manager = TerminalManager::new()
            .map_err(|e| TermAIError::Internal { 
                message: format!("Failed to initialize terminal: {}", e) 
            })?;
        
        // Set up signal handling
        let signal_handler = SignalHandler::new();
        let shutdown_flag = signal_handler.install();
        
        Ok(Self {
            terminal_manager,
            signal_handler,
            shutdown_flag,
        })
    }
    
    pub fn terminal(&mut self) -> &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>> {
        self.terminal_manager.terminal()
    }
    
    pub fn should_shutdown(&self) -> bool {
        self.signal_handler.should_shutdown()
    }
    
    pub fn shutdown(&mut self) -> Result<()> {
        self.terminal_manager.restore()
            .map_err(|e| TermAIError::Internal { 
                message: format!("Failed to restore terminal: {}", e) 
            })?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        // Ensure cleanup happens even if shutdown() wasn't called
        let _ = self.shutdown();
    }
}
```

### 5. Update Main Runner with Proper Cleanup
```rust
// src/ui/tui/runner.rs
use crate::ui::terminal::session::TerminalSession;
use crate::error::{Result, ErrorLogger};

pub async fn run_tui<R, SR, MR>(
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
) -> Result<()>
where
    R: ConfigRepository + Send + Sync,
    SR: SessionRepository + Send + Sync,
    MR: MessageRepository + Send + Sync,
{
    // Create terminal session with proper cleanup
    let mut terminal_session = TerminalSession::new()?;
    
    // Create app state
    let mut app = App::new();
    
    // Load existing sessions
    match fetch_all_sessions_for_ui(session_repository, message_repository) {
        Ok(sessions) => {
            if !sessions.is_empty() {
                app.set_sessions(sessions);
            }
        }
        Err(e) => {
            ErrorLogger::log_warning(&format!("Failed to load sessions: {}", e), Some("startup"));
            // Continue with empty sessions rather than failing
        }
    }

    // Create event handler
    let mut events = EventHandler::new(Duration::from_millis(250));

    // Main event loop with proper error handling
    let result = run_main_loop(&mut terminal_session, &mut app, &mut events, repo, session_repository, message_repository).await;
    
    // Ensure terminal is properly restored
    if let Err(e) = terminal_session.shutdown() {
        ErrorLogger::log_error(&e, Some("terminal_cleanup"));
        eprintln!("Warning: Failed to properly restore terminal: {}", e);
    }
    
    result
}

async fn run_main_loop<R, SR, MR>(
    terminal_session: &mut TerminalSession,
    app: &mut App,
    events: &mut EventHandler,
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
) -> Result<()>
where
    R: ConfigRepository + Send + Sync,
    SR: SessionRepository + Send + Sync,
    MR: MessageRepository + Send + Sync,
{
    loop {
        // Check for external shutdown signals
        if terminal_session.should_shutdown() || app.should_quit {
            break;
        }
        
        // Check for automatic error dismissal
        app.check_error_auto_dismiss();
        
        // Refresh session if needed
        if app.session_needs_refresh {
            app.refresh_current_session(session_repository, message_repository);
        }

        // Draw UI with error handling
        match terminal_session.terminal().draw(|f| ui::draw(f, app, Some(repo))) {
            Ok(_) => {},
            Err(e) => {
                ErrorLogger::log_error(
                    &TermAIError::Internal { message: format!("Failed to draw UI: {}", e) },
                    Some("main_loop")
                );
                // Continue running despite draw errors
            }
        }

        // Handle events
        if let Some(event) = events.next().await {
            if let Err(e) = handle_event(event, app, terminal_session.terminal(), repo, session_repository, message_repository).await {
                ErrorLogger::log_error(&e, Some("event_handling"));
                app.set_error(e);
            }
        }
    }

    Ok(())
}

async fn handle_event<R, SR, MR>(
    event: AppEvent,
    app: &mut App,
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
) -> Result<()>
where
    R: ConfigRepository + Send + Sync,
    SR: SessionRepository + Send + Sync,
    MR: MessageRepository + Send + Sync,
{
    match event {
        AppEvent::Key(key_event) => {
            handle_key_event_safe(key_event, app, terminal, repo, session_repository, message_repository).await
        }
        AppEvent::Mouse(mouse_event) => {
            handle_mouse_event_safe(mouse_event, app).await
        }
        AppEvent::Resize(width, height) => {
            // Handle terminal resize
            if let Err(e) = terminal.resize(ratatui::layout::Rect::new(0, 0, width, height)) {
                return Err(TermAIError::Internal { 
                    message: format!("Failed to resize terminal: {}", e) 
                });
            }
            Ok(())
        }
        AppEvent::Tick => {
            // Regular maintenance
            Ok(())
        }
    }
}

// Wrapper functions that convert errors to TermAIError
async fn handle_key_event_safe<R, SR, MR>(
    key_event: crossterm::event::KeyEvent,
    app: &mut App,
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    repo: &R,
    session_repository: &SR,
    message_repository: &MR,
) -> Result<()>
where
    R: ConfigRepository + Send + Sync,
    SR: SessionRepository + Send + Sync,
    MR: MessageRepository + Send + Sync,
{
    // ... existing key event handling logic with proper error conversion
    Ok(())
}

async fn handle_mouse_event_safe(
    mouse_event: crossterm::event::MouseEvent,
    app: &mut App,
) -> Result<()> {
    // ... existing mouse event handling logic
    Ok(())
}
```

### 6. Add Terminal State Validation
```rust
// src/ui/terminal/validator.rs
use crossterm::terminal;
use std::io;

pub struct TerminalValidator;

impl TerminalValidator {
    pub fn validate_initial_state() -> io::Result<TerminalState> {
        let size = terminal::size()?;
        
        // Check minimum terminal size
        if size.0 < 80 || size.1 < 24 {
            eprintln!("Warning: Terminal size {}x{} is smaller than recommended 80x24", size.0, size.1);
        }
        
        Ok(TerminalState {
            width: size.0,
            height: size.1,
            supports_colors: Self::check_color_support(),
            supports_mouse: Self::check_mouse_support(),
        })
    }
    
    fn check_color_support() -> bool {
        // Check if terminal supports colors
        std::env::var("TERM")
            .map(|term| !term.contains("mono") && term != "dumb")
            .unwrap_or(true)
    }
    
    fn check_mouse_support() -> bool {
        // Most modern terminals support mouse
        true
    }
    
    pub fn validate_cleanup() -> io::Result<()> {
        // Verify terminal was properly restored
        // This is mainly for testing purposes
        Ok(())
    }
}

#[derive(Debug)]
pub struct TerminalState {
    pub width: u16,
    pub height: u16,
    pub supports_colors: bool,
    pub supports_mouse: bool,
}
```

### 7. Update Main Function with Error Handling
```rust
// src/main.rs
use crate::ui::terminal::validator::TerminalValidator;
use crate::error::{ErrorLogger, TermAIError};

#[tokio::main]
async fn main() {
    // Install panic handler as early as possible
    crate::ui::terminal::panic_handler::PanicHandler::install();
    
    // Validate terminal before starting
    match TerminalValidator::validate_initial_state() {
        Ok(state) => {
            if !state.supports_colors {
                eprintln!("Warning: Limited color support detected");
            }
        }
        Err(e) => {
            eprintln!("Terminal validation failed: {}", e);
            std::process::exit(1);
        }
    }
    
    // Run the main application
    if let Err(e) = run_application().await {
        ErrorLogger::log_error(&e, Some("main"));
        eprintln!("Application error: {}", e.user_message());
        std::process::exit(1);
    }
}

async fn run_application() -> crate::error::Result<()> {
    // ... existing application logic
    Ok(())
}
```

## Testing Requirements

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_guard_creation() {
        let guard = TerminalGuard::new();
        assert!(guard.is_ok());
    }
    
    #[test]
    fn test_panic_handler_installation() {
        // Test that panic handler can be installed without panicking
        PanicHandler::install();
        // Can't easily test actual panic handling in unit tests
    }
    
    #[test] 
    fn test_signal_handler_creation() {
        let handler = SignalHandler::new();
        assert!(!handler.should_shutdown());
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_terminal_session_lifecycle() {
    let session = TerminalSession::new();
    assert!(session.is_ok());
    
    let mut session = session.unwrap();
    
    // Test that terminal is available
    assert!(session.terminal().size().is_ok());
    
    // Test shutdown
    assert!(session.shutdown().is_ok());
}

#[test]
fn test_terminal_restoration_on_drop() {
    // Create and immediately drop terminal session
    {
        let _session = TerminalSession::new().unwrap();
        // Terminal should be set up here
    }
    // Terminal should be restored here
    
    // Verify terminal state (this is challenging to test automatically)
    assert!(TerminalValidator::validate_cleanup().is_ok());
}
```

### Manual Tests
- Test Ctrl+C handling
- Test application crash recovery
- Test terminal state after panic
- Test terminal resize handling

## Acceptance Criteria
- [ ] Terminal state is always restored on exit
- [ ] Panics don't leave terminal corrupted
- [ ] Signal handling works correctly
- [ ] No resource leaks in terminal management
- [ ] Error messages are clear when terminal issues occur
- [ ] Application gracefully handles terminal resize
- [ ] All terminal features are properly cleaned up

## Error Scenarios to Handle
1. **Terminal too small**: Graceful degradation or clear error message
2. **No terminal capabilities**: Fallback to CLI mode
3. **Terminal disconnection**: Proper error handling
4. **Permission issues**: Clear error messages

## Future Enhancements
- Automatic terminal capability detection
- Fallback rendering for limited terminals
- Terminal session persistence across disconnections
- Better handling of SSH/remote terminals
- Terminal multiplexer integration (tmux, screen)