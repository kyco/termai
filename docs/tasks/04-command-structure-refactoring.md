# Task: Command Structure Refactoring

## Overview
Replace the current flat command structure with intuitive subcommands, reducing cognitive load and improving discoverability.

## Success Criteria
- [ ] Intuitive command hierarchy with logical grouping
- [ ] Reduced help text complexity and better navigation
- [ ] Clear command purpose and reduced ambiguity

**Note**: Backwards compatibility is explicitly not a concern - existing command patterns will be replaced.

## Implementation Tasks

### 1. Design New Command Hierarchy
- [ ] Design primary subcommands:
  - [ ] `termai setup` - Initial configuration and setup
  - [ ] `termai chat` - Interactive conversation mode
  - [ ] `termai ask` - One-shot questions (current default behavior)
  - [ ] `termai session` - Session management operations
  - [ ] `termai config` - Configuration management
- [ ] Design secondary subcommands:
  - [ ] `termai session list` - List all sessions
  - [ ] `termai session delete <name>` - Delete specific session
  - [ ] `termai config show` - Display current configuration
  - [ ] `termai config set <key> <value>` - Set configuration values

### 2. Restructure Arguments and Options
- [ ] Create dedicated Args structs for each subcommand:
  - [ ] `SetupArgs` for setup wizard
  - [ ] `ChatArgs` for interactive mode
  - [ ] `AskArgs` for one-shot queries
  - [ ] `SessionArgs` for session operations
  - [ ] `ConfigArgs` for configuration management
- [ ] Move global options to appropriate subcommands
- [ ] Consolidate related options into logical groups
- [ ] Add meaningful defaults and validation

### 3. Implement Subcommand Routing
- [ ] Create `Commands` enum with all subcommands
- [ ] Implement command dispatch logic in main.rs
- [ ] Add proper error handling for unknown commands
- [ ] Add command aliases for common operations

### 4. Update Help and Documentation
- [ ] Implement contextual help for each subcommand
- [ ] Add usage examples for common workflows
- [ ] Create command discovery aids (suggestions)
- [ ] Improve error messages with actionable guidance
- [ ] Add man page generation support

### 5. Enhanced Argument Parsing
- [ ] Add intelligent argument validation
- [ ] Implement conflicting argument detection
- [ ] Add required argument validation with clear errors
- [ ] Support environment variable fallbacks
- [ ] Add configuration file integration

### 6. Command Auto-completion
- [ ] Generate shell completion scripts
- [ ] Support Bash, Zsh, Fish, and PowerShell
- [ ] Add dynamic completion for session names
- [ ] Support file path completion for context arguments
- [ ] Add provider and model name completion

### 7. Testing
- [ ] Unit tests for argument parsing logic
- [ ] Integration tests for each subcommand
- [ ] Shell completion tests
- [ ] Help text validation tests

### 8. Documentation Updates
- [ ] Update README with new command examples
- [ ] Create command reference guide
- [ ] Update CLAUDE.md with new command patterns
- [ ] Create quick reference card

## File Changes Required

### New Files
- `src/commands/mod.rs` - Command structure definitions
- `src/commands/setup.rs` - Setup command implementation
- `src/commands/chat.rs` - Chat command implementation  
- `src/commands/ask.rs` - Ask command implementation
- `src/commands/session.rs` - Session command implementation
- `src/commands/config.rs` - Config command implementation

### Modified Files
- `src/args.rs` - Restructure to use subcommands
- `src/main.rs` - Implement command routing
- `Cargo.toml` - Add clap derive features

## Command Structure Design

### Current Structure (Problems)
```bash
termai --chat-gpt-api-key "key" "question"
termai --sessions-all
termai --session "name" "question" 
termai --redact-add "pattern"
```

### New Structure (Solution)
```bash
termai setup                    # Interactive setup
termai chat                     # Start interactive session
termai ask "question"           # One-shot question
termai ask "question" src/      # With context
termai session list             # List sessions
termai session delete "name"    # Delete session
termai config show             # Show configuration
termai config set provider claude
```

### Backwards Compatibility
```bash
# Still supported with deprecation warning
termai "question" --provider claude
termai --sessions-all  # Maps to: termai session list
```

## Implementation Details

### Command Enum Structure
```rust
#[derive(Subcommand)]
pub enum Commands {
    /// Interactive setup wizard
    Setup(SetupArgs),
    /// Start interactive chat session
    Chat(ChatArgs),
    /// Ask a one-shot question
    Ask(AskArgs),
    /// Manage conversation sessions
    Session(SessionArgs),
    /// Manage configuration
    Config(ConfigArgs),
}

#[derive(Subcommand)]
pub enum SessionCommands {
    /// List all sessions
    List,
    /// Delete a session
    Delete { name: String },
    /// Show session details
    Show { name: String },
}
```

### Help Text Example
```
termai 0.2.0
A powerful, privacy-focused AI assistant for your terminal

USAGE:
    termai <SUBCOMMAND>

SUBCOMMANDS:
    setup     Interactive setup wizard
    chat      Start interactive conversation
    ask       Ask a one-shot question  
    session   Manage conversation sessions
    config    Manage configuration
    help      Print this message or the help of the given subcommand(s)

Use 'termai <subcommand> --help' for more information about a specific command.
```

## Success Metrics
- Help text comprehension: >90% users understand command structure
- Command discovery: >80% users find needed commands without documentation
- Error clarity: <10% support requests about command usage

## Risk Mitigation
- **Risk**: Increased complexity for simple usage
  - **Mitigation**: Keep `ask` as default for simple questions

**Note**: Backwards compatibility is explicitly not a concern - existing command patterns will be replaced with the new structure.