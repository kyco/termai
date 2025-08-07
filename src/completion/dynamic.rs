/// Dynamic completion system for advanced shell integration
use crate::completion::values::CompletionValues;
use crate::repository::db::SqliteRepository;
use anyhow::Result;
#[allow(unused_imports)]
use colored::*;

/// Dynamic completer that provides context-aware completions
pub struct DynamicCompleter;

impl DynamicCompleter {
    /// Generate completion for a given command and position
    pub fn complete(repo: &SqliteRepository, args: &[String]) -> Result<Vec<String>> {
        if args.is_empty() {
            return Ok(Self::top_level_commands());
        }

        match args[0].as_str() {
            "session" => Self::complete_session_command(repo, &args[1..]),
            "config" => Self::complete_config_command(&args[1..]),
            "ask" | "chat" => Self::complete_context_command(&args[1..]),
            "completion" => Ok(CompletionValues::shell_types()),
            _ => Ok(Vec::new()),
        }
    }

    /// Get top-level commands
    fn top_level_commands() -> Vec<String> {
        vec![
            "setup".to_string(),
            "chat".to_string(),
            "ask".to_string(),
            "session".to_string(),
            "config".to_string(),
            "redact".to_string(),
            "completion".to_string(),
        ]
    }

    /// Complete session subcommands
    fn complete_session_command(repo: &SqliteRepository, args: &[String]) -> Result<Vec<String>> {
        if args.is_empty() {
            return Ok(vec![
                "list".to_string(),
                "delete".to_string(),
                "show".to_string(),
            ]);
        }

        match args[0].as_str() {
            "delete" | "show" => {
                // Suggest session names
                CompletionValues::session_names(repo)
            }
            "list" => Ok(vec![
                "--verbose".to_string(),
                "--limit".to_string(),
                "--sort".to_string(),
            ]),
            _ => Ok(Vec::new()),
        }
    }

    /// Complete config subcommands
    fn complete_config_command(args: &[String]) -> Result<Vec<String>> {
        if args.is_empty() {
            return Ok(vec![
                "show".to_string(),
                "set-openai".to_string(),
                "set-claude".to_string(),
                "set-provider".to_string(),
                "reset".to_string(),
                "env".to_string(),
            ]);
        }

        match args[0].as_str() {
            "set-provider" => Ok(CompletionValues::provider_names()),
            _ => Ok(Vec::new()),
        }
    }

    /// Complete context-related commands (ask, chat)
    fn complete_context_command(args: &[String]) -> Result<Vec<String>> {
        if args.is_empty() {
            return Ok(vec![
                "--directory".to_string(),
                "--directories".to_string(),
                "--exclude".to_string(),
                "--smart-context".to_string(),
                "--session".to_string(),
            ]);
        }

        // Look for the last flag to provide context-specific completions
        if let Some(last_arg) = args.last() {
            match last_arg.as_str() {
                "--directories" | "--directory" => {
                    Ok(CompletionValues::common_context_directories())
                }
                "--exclude" => Ok(CompletionValues::common_exclude_patterns()),
                "--chunk-strategy" => Ok(CompletionValues::chunk_strategies()),
                _ => Ok(Vec::new()),
            }
        } else {
            Ok(Vec::new())
        }
    }

    /// Print completions to stdout (for shell integration)
    pub fn print_completions(repo: &SqliteRepository, args: &[String]) -> Result<()> {
        let completions = Self::complete(repo, args)?;
        for completion in completions {
            println!("{}", completion);
        }
        Ok(())
    }

    /// Generate enhanced completion script for shells
    pub fn generate_enhanced_completion_script(shell: &str) -> String {
        match shell {
            "bash" => Self::bash_enhanced_script(),
            "zsh" => Self::zsh_enhanced_script(),
            "fish" => Self::fish_enhanced_script(),
            _ => String::new(),
        }
    }

    fn bash_enhanced_script() -> String {
        r#"
# Enhanced TermAI completion for Bash
# Source this file or add to ~/.bashrc

_termai_complete() {
    local cur prev words cword
    _init_completion || return

    # Get dynamic completions from TermAI
    local completions=$(termai _complete "${COMP_WORDS[@]:1}" 2>/dev/null)
    
    if [[ -n "$completions" ]]; then
        COMPREPLY=($(compgen -W "$completions" -- "$cur"))
        return 0
    fi

    # Fallback to basic completion
    case "$prev" in
        --session)
            local sessions=$(termai session list --quiet 2>/dev/null | grep -o '^[^[:space:]]*')
            COMPREPLY=($(compgen -W "$sessions" -- "$cur"))
            return 0
            ;;
        --provider)
            COMPREPLY=($(compgen -W "claude openai" -- "$cur"))
            return 0
            ;;
        --directories|--directory)
            COMPREPLY=($(compgen -d -- "$cur"))
            return 0
            ;;
    esac

    # Default file completion
    _filedir
}

complete -F _termai_complete termai
"#
        .to_string()
    }

    fn zsh_enhanced_script() -> String {
        r#"
# Enhanced TermAI completion for Zsh
# Source this file or add to ~/.zshrc

_termai_complete() {
    local context state state_descr line
    typeset -A opt_args

    # Get dynamic completions from TermAI
    local completions=(${(f)"$(termai _complete "${words[@]:1}" 2>/dev/null)"})
    
    if [[ ${#completions[@]} -gt 0 ]]; then
        _describe 'completions' completions
        return 0
    fi

    _arguments \
        '1: :->command' \
        '*: :->args' && return 0

    case $state in
        command)
            local commands=(
                'setup:Interactive setup wizard'
                'chat:Start interactive conversation'
                'ask:Ask a one-shot question'
                'session:Manage conversation sessions'
                'config:Manage configuration'
                'redact:Manage redaction patterns'
                'completion:Generate shell completions'
            )
            _describe 'command' commands
            ;;
        args)
            case $words[2] in
                session)
                    _arguments \
                        '1: :(list delete show)' \
                        '--session: :($(termai session list --quiet 2>/dev/null))'
                    ;;
                config)
                    _arguments \
                        '1: :(show set-openai set-claude set-provider reset env)'
                    ;;
                ask|chat)
                    _arguments \
                        '--session: :($(termai session list --quiet 2>/dev/null))' \
                        '--provider: :(claude openai)' \
                        '--directory:directory:_files -/' \
                        '--directories:directories:_files -/'
                    ;;
            esac
            ;;
    esac
}

compdef _termai_complete termai
"#
        .to_string()
    }

    fn fish_enhanced_script() -> String {
        r#"
# Enhanced TermAI completion for Fish
# Source this file or add to ~/.config/fish/config.fish

# Dynamic completion function
function __termai_complete
    set -l completions (termai _complete (commandline -opc)[2..-1] 2>/dev/null)
    if test -n "$completions"
        printf '%s\n' $completions
        return 0
    end
    return 1
end

# Session name completion
function __termai_sessions
    termai session list --quiet 2>/dev/null
end

# Main completions
complete -f -c termai -n '__fish_use_subcommand' -a 'setup chat ask session config redact completion'
complete -f -c termai -n '__fish_use_subcommand' -s h -l help -d 'Show help information'

# Session subcommands
complete -f -c termai -n '__fish_seen_subcommand_from session' -a 'list delete show'
complete -f -c termai -n '__fish_seen_subcommand_from session; and __fish_seen_subcommand_from delete show' -a '(__termai_sessions)'

# Config subcommands
complete -f -c termai -n '__fish_seen_subcommand_from config' -a 'show set-openai set-claude set-provider reset env'
complete -f -c termai -n '__fish_seen_subcommand_from config; and __fish_seen_subcommand_from set-provider' -a 'claude openai'

# Common flags
complete -c termai -n '__fish_seen_subcommand_from ask chat' -l session -a '(__termai_sessions)' -d 'Session name'
complete -c termai -n '__fish_seen_subcommand_from ask chat' -l provider -a 'claude openai' -d 'AI provider'
complete -c termai -n '__fish_seen_subcommand_from ask chat' -l directory -d 'Context directory' -x -a '(__fish_complete_directories)'
complete -c termai -n '__fish_seen_subcommand_from ask chat' -l smart-context -d 'Enable smart context'

# Dynamic completions as fallback
complete -c termai -a '(__termai_complete)'
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_level_commands() {
        let commands = DynamicCompleter::top_level_commands();
        assert!(commands.contains(&"setup".to_string()));
        assert!(commands.contains(&"chat".to_string()));
        assert!(commands.contains(&"session".to_string()));
    }

    #[test]
    fn test_complete_empty_args() {
        // This test would need a mock repository
        let _args: Vec<String> = vec![];
        // Can't easily test without a real repo, but we can test the structure
        assert_eq!(DynamicCompleter::top_level_commands().len(), 7);
    }

    #[test]
    fn test_enhanced_scripts_not_empty() {
        assert!(!DynamicCompleter::bash_enhanced_script().is_empty());
        assert!(!DynamicCompleter::zsh_enhanced_script().is_empty());
        assert!(!DynamicCompleter::fish_enhanced_script().is_empty());
    }
}
