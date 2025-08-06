use anyhow::{anyhow, Result};
use rustyline::{Config, Editor, Helper, error::ReadlineError, history::FileHistory};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::validate::{Validator, MatchingBracketValidator};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashSet;

/// Custom helper for rustyline that provides completion and validation
pub struct ChatHelper {
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    commands: HashSet<String>,
}

impl ChatHelper {
    pub fn new() -> Self {
        let mut commands = HashSet::new();
        // Add all slash commands for tab completion
        let command_list = vec![
            "/help", "/h", "/save", "/s", "/context", "/ctx", 
            "/clear", "/c", "/exit", "/quit", "/q", "/retry", "/r",
            "/branch", "/b", "/add", "/remove", "/rm"
        ];
        
        for cmd in command_list {
            commands.insert(cmd.to_string());
        }
        
        Self {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            validator: MatchingBracketValidator::new(),
            hinter: HistoryHinter::new(),
            commands,
        }
    }
}

impl Completer for ChatHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        // If line starts with '/', complete slash commands
        if line.starts_with('/') {
            let input = &line[1..pos];
            let mut matches = Vec::new();
            
            for command in &self.commands {
                if command[1..].starts_with(input) {  // Skip the '/' prefix
                    matches.push(Pair {
                        display: command.clone(),
                        replacement: command.clone(),
                    });
                }
            }
            
            Ok((1, matches))  // Start replacement from position 1 (after '/')
        } else {
            // Fall back to filename completion for regular messages
            self.completer.complete(line, pos, _ctx)
        }
    }
}

impl Hinter for ChatHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ChatHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(prompt)
        } else {
            Owned(format!("\x1b[1;32m{}\x1b[0m", prompt))  // Green bold
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("\x1b[2m{}\x1b[0m", hint))  // Dimmed
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        // Highlight slash commands
        if line.starts_with('/') {
            if let Some(space_pos) = line.find(' ') {
                let command = &line[..space_pos];
                let rest = &line[space_pos..];
                if self.commands.contains(command) {
                    return Owned(format!("\x1b[1;36m{}\x1b[0m{}", command, rest));  // Cyan bold
                }
            } else if self.commands.contains(line) {
                return Owned(format!("\x1b[1;36m{}\x1b[0m", line));  // Cyan bold
            }
        }
        
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

impl Validator for ChatHelper {
    fn validate(
        &self,
        ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
        self.validator.validate(ctx)
    }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

impl Helper for ChatHelper {}

/// Interactive REPL for chat mode
pub struct ChatRepl {
    editor: Editor<ChatHelper, FileHistory>,
    prompt: String,
}

impl ChatRepl {
    pub fn new() -> Result<Self> {
        let config = Config::builder()
            .history_ignore_space(true)
            .completion_type(rustyline::CompletionType::List)
            .edit_mode(rustyline::EditMode::Emacs)
            .build();
        
        let helper = ChatHelper::new();
        let mut editor = Editor::with_config(config)?;
        editor.set_helper(Some(helper));
        
        // Load history if it exists
        let _ = editor.load_history(".termai_history");
        
        Ok(Self {
            editor,
            prompt: "❯ ".to_string(),  // Restore the prompt
        })
    }
    
    /// Read a line of input from the user
    pub fn read_line(&mut self) -> Result<String> {
        match self.editor.readline(&self.prompt) {
            Ok(line) => {
                // Add to history if not empty and not just whitespace
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let _ = self.editor.add_history_entry(&line);
                }
                Ok(line)
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C was pressed
                Err(anyhow!("Interrupted"))
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D was pressed
                Err(anyhow!("EOF"))
            }
            Err(err) => Err(anyhow!("Readline error: {}", err)),
        }
    }
    
    /// Update the prompt (e.g., to show context or status)
    #[allow(dead_code)]
    pub fn set_prompt(&mut self, prompt: String) {
        self.prompt = prompt;
    }
    
    /// Get the current prompt
    #[allow(dead_code)]
    pub fn get_prompt(&self) -> &str {
        &self.prompt
    }
    
    /// Save command history
    pub fn save_history(&mut self) -> Result<()> {
        self.editor.save_history(".termai_history")
            .map_err(|e| anyhow!("Failed to save history: {}", e))?;
        Ok(())
    }
    
    /// Clear the screen
    pub fn clear_screen(&mut self) {
        print!("\x1B[2J\x1B[1;1H");  // ANSI escape codes to clear screen
    }
    
    /// Print a message without interfering with input
    pub fn print_message(&self, message: &str) {
        println!("{}", message);
    }
}

impl Default for ChatRepl {
    fn default() -> Self {
        Self::new().expect("Failed to create ChatRepl")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_helper_creation() {
        let helper = ChatHelper::new();
        assert!(!helper.commands.is_empty());
        assert!(helper.commands.contains("/help"));
        assert!(helper.commands.contains("/exit"));
    }

    #[test]
    fn test_command_completion() {
        let helper = ChatHelper::new();
        let history = rustyline::history::FileHistory::new();
        let ctx = rustyline::Context::new(&history);
        
        // Test slash command completion
        let result = helper.complete("/he", 3, &ctx);
        assert!(result.is_ok());
        let (start, matches) = result.unwrap();
        assert_eq!(start, 1);
        assert!(!matches.is_empty());
        
        // Find /help in matches
        let help_match = matches.iter().find(|pair| pair.replacement == "/help");
        assert!(help_match.is_some());
    }

    #[test]
    fn test_repl_creation() {
        let repl = ChatRepl::new();
        assert!(repl.is_ok());
        
        let repl = repl.unwrap();
        assert_eq!(repl.prompt, "❯ ");
    }
    
    #[test]
    fn test_prompt_modification() {
        let mut repl = ChatRepl::new().unwrap();
        repl.set_prompt("test> ".to_string());
        assert_eq!(repl.prompt, "test> ");
    }
}