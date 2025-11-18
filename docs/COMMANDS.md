# TermAI Commands Reference Guide

> Complete reference for all TermAI commands and options

## 📋 Table of Contents

- [Basic Commands](#basic-commands)
- [Smart Context Discovery](#smart-context-discovery)
- [Configuration Management](#configuration-management)  
- [Session Management](#session-management)
- [Privacy & Redaction](#privacy--redaction)
- [Shell Completion](#shell-completion)
- [Environment Variables](#environment-variables)
- [Quick Reference](#quick-reference)

---

## Basic Commands

### `termai setup`
Interactive setup wizard for first-time configuration.

```bash
termai setup                    # Full interactive setup
termai setup --auto-accept     # Non-interactive with defaults  
termai setup --skip-validation # Skip API key validation
termai setup --force           # Overwrite existing configuration
```

**Options:**
- `--auto-accept` - Accept all defaults without prompting
- `--skip-validation` - Skip API key validation during setup
- `--force` - Overwrite existing configuration without confirmation

### `termai ask <question>`
Ask a one-shot question to the AI assistant.

```bash
termai ask "How do I implement binary search in Rust?"
termai ask "Explain this function" ./src/main.rs
termai ask --smart-context "Add error handling" .
```

**Required:**
- `<question>` - The question to ask (use quotes if it contains spaces)

**Options:**
- `--directory <dir>` - Include files from specific directory
- `--directories <dirs>` - Include files from multiple directories (comma-separated)
- `--exclude <patterns>` - Exclude files matching patterns (comma-separated)
- `--smart-context` - Enable intelligent file discovery
- `--context-query <query>` - Target specific context within smart discovery
- `--max-context-tokens <num>` - Limit context size (default: 4000)
- `--preview-context` - Show file selection before processing
- `--chunked-analysis` - Enable chunked analysis for large contexts
- `--chunk-strategy <strategy>` - Chunking strategy: module|functional|token|hierarchical
- `--session <name>` - Use or create named session
- `--system-prompt <prompt>` - Custom system prompt
- `--provider <provider>` - AI provider: claude|openai
- `--model <model>` - Specific model to use

### `termai chat [input]`
Start an interactive chat session.

```bash
termai chat                              # Interactive mode
termai chat "Let's discuss architecture" # Start with input
termai chat --session myproject          # Continue named session
```

**Optional:**
- `[input]` - Initial message to start the conversation

**Options:** Same as `termai ask` command

---

## Smart Context Discovery

Smart Context Discovery automatically analyzes your project and selects the most relevant files for your query.

### Basic Usage
```bash
termai ask --smart-context "Your question here" .
termai chat --smart-context --session work "Let's work on the project" .
```

### Advanced Features

#### Context Preview
```bash
termai ask --smart-context --preview-context "Add logging" .
```
Shows which files would be selected before processing.

#### Chunked Analysis
```bash
termai ask --smart-context --chunked-analysis "Full code review" .
termai ask --smart-context --chunk-strategy hierarchical "Analyze architecture" .
```

**Chunk Strategies:**
- `module` - Group by modules/packages
- `functional` - Group by functionality  
- `token` - Split by token limits
- `hierarchical` - Use project hierarchy

#### Targeted Context
```bash
termai ask --smart-context --context-query "authentication" "Fix login issues" .
termai ask --smart-context --context-query "database queries" "Optimize performance" .
```

#### Token Management
```bash
termai ask --smart-context --max-context-tokens 6000 "Explain codebase" .
```

---

## Configuration Management

### `termai config`
Manage TermAI configuration settings.

#### View Configuration
```bash
termai config show             # Display current settings
termai config env              # Show environment variables with current values
```

#### Set API Keys
```bash
termai config set-openai <key>    # Configure OpenAI API key
termai config set-claude <key>    # Configure Claude API key
termai config set-provider <name> # Set default provider (claude|openai)
```

#### Reset Configuration
```bash
termai config reset            # Clear all configuration settings
```

---

## Session Management

### `termai sessions`
Manage conversation sessions for organizing your work.

#### List Sessions
```bash
termai sessions list                    # Show all sessions
termai sessions list --limit 10         # Show first 10 sessions
termai sessions list --sort date        # Sort by date (name|date|messages)
```

#### Session Details
```bash
termai sessions show <session_name>     # View session details and history
```

#### Delete Sessions
```bash
termai sessions delete <session_name>   # Remove a specific session
```

#### Using Sessions
```bash
# Start or continue a named session
termai ask --session myproject "Continue our discussion"
termai chat --session coding_help
```

---

## Privacy & Redaction

### `termai redact`
Manage sensitive text redaction patterns.

#### Add Redaction Patterns
```bash
termai redact add "SECRET_KEY"              # Redact literal text
termai redact add "user_.*@company\.com"    # Redact using regex
termai redact add "password\s*=\s*.*"       # Redact password assignments
```

#### List Patterns
```bash
termai redact list             # Show all active redaction patterns
```

#### Remove Patterns
```bash
termai redact remove "SECRET_KEY"          # Remove specific pattern
termai redact reset                        # Clear all patterns
```

---

## Shell Completion

### `termai completion`
Generate shell completion scripts for enhanced productivity.

#### Basic Completions
```bash
termai completion bash                # Generate Bash completion
termai completion zsh                 # Generate Zsh completion
termai completion fish                # Generate Fish completion
termai completion powershell          # Generate PowerShell completion
```

#### Enhanced Completions
```bash
termai completion enhanced bash       # Enhanced Bash with dynamic features
termai completion enhanced zsh        # Enhanced Zsh with smart suggestions  
termai completion enhanced fish       # Enhanced Fish with context awareness
```

**Enhanced Features:**
- Dynamic session name completion
- Context-aware argument suggestions
- Model and provider name completion
- Smart directory completion

#### Installation Examples
```bash
# Bash
termai completion bash > ~/.termai-completion.bash
echo 'source ~/.termai-completion.bash' >> ~/.bashrc

# Zsh  
termai completion zsh > ~/.termai-completion.zsh
echo 'source ~/.termai-completion.zsh' >> ~/.zshrc

# Fish
termai completion fish > ~/.config/fish/completions/termai.fish
```

---

## Environment Variables

TermAI supports environment variables for flexible configuration:

### API Keys
```bash
OPENAI_API_KEY          # OpenAI API key
CLAUDE_API_KEY          # Claude API key
```

### Provider Settings
```bash
TERMAI_PROVIDER         # Default provider (claude|openai)
TERMAI_MODEL            # Default model name
```

### Context Settings
```bash
TERMAI_MAX_CONTEXT_TOKENS    # Default context token limit
TERMAI_SYSTEM_PROMPT         # Default system prompt
TERMAI_SMART_CONTEXT         # Enable smart context (true|false)
TERMAI_PREVIEW_CONTEXT       # Enable context preview (true|false)
```

### Session Settings  
```bash
TERMAI_DEFAULT_SESSION       # Default session name
TERMAI_AUTO_SAVE_SESSIONS    # Auto-save sessions (true|false)
```

### Advanced Settings
```bash
TERMAI_CHUNK_STRATEGY        # Default chunking strategy
TERMAI_CHUNKED_ANALYSIS      # Enable chunked analysis (true|false)
TERMAI_EXCLUDE_PATTERNS      # Default exclude patterns (comma-separated)
```

### View Environment Variables
```bash
termai config env            # Show all environment variables with values
```

---

## Quick Reference

### 🚀 Getting Started
```bash
termai setup                                    # Interactive setup
termai ask "How do I center a div in CSS?"     # Quick question
termai chat                                     # Interactive conversation
```

### 🧠 Smart Context
```bash
termai ask --smart-context "Fix bugs" .                    # Auto-discover relevant files
termai ask --smart-context --preview-context "Review" .    # Preview file selection
termai ask --smart-context --chunked-analysis "Analyze" .  # Handle large codebases
```

### ⚙️ Configuration
```bash
termai config show                     # View settings
termai config set-provider claude      # Set default provider
termai config env                      # View environment variables
```

### 💬 Sessions
```bash
termai sessions list                   # List all sessions
termai chat --session mywork          # Use named session
termai sessions delete old_session     # Clean up sessions
```

### 🔒 Privacy
```bash
termai redact add "API_KEY_.*"         # Add redaction pattern
termai redact list                     # View active patterns
```

### 🐚 Completion
```bash
termai completion enhanced bash        # Enhanced shell completion
# Save output and source in your shell config
```

### 🔍 Help
```bash
termai --help                          # General help
termai ask --help                      # Command-specific help
termai config --help                   # Subcommand help
```

---

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Configuration error
- `3` - Network/API error  
- `4` - Validation error
- `5` - File system error

## Configuration Files

- **Config Database**: `~/.config/termai/app.db`
- **Project Config**: `.termai.toml` (optional, in project root)

## Troubleshooting

1. **API Key Issues**: Run `termai config show` to verify configuration
2. **Smart Context Not Working**: Ensure you're in a recognized project directory
3. **Completion Issues**: Verify TermAI is in your PATH and completion script is sourced
4. **Permission Errors**: Check file permissions on config directory

For more detailed troubleshooting, run commands with increased verbosity or check the documentation.