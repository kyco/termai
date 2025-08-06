# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

TermAI is a terminal-based AI assistant built in Rust that supports both OpenAI and Claude APIs. It provides privacy-focused chat capabilities with local context awareness, session management, and redaction features.

## Development Commands

### Build and Testing
- `cargo build --release` - Build the project in release mode
- `cargo test` - Run the test suite
- `cargo run --release -- [args]` - Run the application with arguments
- `just build` - Alternative build command using justfile
- `just test` - Run tests via justfile
- `just run [args]` - Run with arguments via justfile

### Code Quality
- `cargo fmt` - Format code
- `cargo clippy -- -D warnings` - Run clippy linter with warnings as errors
- `just fmt` - Format code via justfile  
- `just clippy` - Run clippy via justfile

### Dependencies
- `cargo update` - Update dependencies
- `just update-deps` - Update dependencies via justfile
- `just outdated` - Check for outdated dependencies

### Documentation
- `cargo doc --open` - Generate and open documentation
- `just doc` - Generate documentation via justfile

## User Commands

### Setup and Configuration
- `cargo run --release -- setup` - Interactive setup wizard (recommended for first-time setup)
- `cargo run --release -- config show` - Display current configuration
- `cargo run --release -- config reset` - Clear all configuration
- `cargo run --release -- config set-claude KEY` - Set Claude API key
- `cargo run --release -- config set-openai KEY` - Set OpenAI API key
- `cargo run --release -- config set-provider claude` - Set default provider

### Chat and Usage
- `cargo run --release -- "your question"` - Basic chat query
- `cargo run --release -- chat "your question"` - Explicit chat command
- `cargo run --release -- "question" ./path/to/file` - Chat with local file context

### Session Management  
- `cargo run --release -- sessions list` - List all saved sessions
- `cargo run --release -- --session name "question"` - Use named session

### Privacy and Redaction
- `cargo run --release -- redact add "pattern"` - Add redaction pattern
- `cargo run --release -- redact list` - List redaction patterns
- `cargo run --release -- redact remove "pattern"` - Remove redaction pattern

## Architecture

TermAI follows a layered architecture with clear separation of concerns:

### Core Layers
- **main.rs**: Entry point, argument parsing, and orchestration
- **Repository Layer**: Data access abstraction using SQLite (repository/db.rs)
- **Service Layer**: Business logic for config, sessions, and LLM interactions
- **Adapter Layer**: API integrations for OpenAI and Claude

### Key Modules

#### LLM Integration (`src/llm/`)
- **claude/**: Claude API adapter, models, and chat service
- **openai/**: OpenAI API adapter, models, and chat service  
- **common/**: Shared models and constants for both providers

#### Configuration Management (`src/config/`)
- **service/**: Provider configuration, API keys, redaction settings
- **repository/**: Config data access layer
- **entity/**: Config data models

#### Session Management (`src/session/`)
- **service/**: Session and message management
- **repository/**: Session persistence layer
- **entity/**: Session and message entities

#### Supporting Modules
- **path/**: File content extraction for local context
- **redactions/**: Privacy protection through text redaction
- **output/**: Message formatting and display
- **ui/**: User interface components (thinking timer)

### Data Flow
1. Arguments parsed via clap (args.rs)
2. SQLite repository initialized for persistence
3. Configuration loaded (API keys, provider, redactions)
4. Local context extracted from specified files/directories
5. Session created/loaded with message history
6. User input processed and redacted
7. LLM provider called (Claude or OpenAI) via adapter pattern
8. Response processed and displayed
9. Session and messages persisted

### Key Design Patterns
- **Repository Pattern**: Data access through traits (ConfigRepository, SessionRepository, MessageRepository)
- **Adapter Pattern**: LLM provider abstraction in claude/ and openai/ modules
- **Service Layer**: Business logic separation from data and presentation layers

## Database
- SQLite database stored in `~/.config/termai/app.db`
- Handles configuration, sessions, messages, and redaction patterns
- Repository pattern provides abstraction over direct database access