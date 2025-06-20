# TermAI

> A terminal AI assistant

TermAI is a command-line AI assistant built in Rust. It provides an interactive terminal interface for conversations with OpenAI and Anthropic Claude models, with support for session management, local file context, and privacy features.

![License: MIT](https://img.shields.io/badge/License-MIT-green.svg) ![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)

## ✨ Features

- **Terminal Interface**: Interactive chat interface with session management and navigation
- **Multiple LLM Providers**: Supports OpenAI and Anthropic Claude APIs
- **Session Management**: Save and restore conversations with auto-generated titles
- **Local File Context**: Include local files and directories in conversations
- **Privacy Controls**: Redact sensitive information before API calls
- **Dual Modes**: Terminal UI for interactive use, CLI mode for scripting
- **Configuration Management**: Store API keys and settings locally

## 🚀 Installation

### Prerequisites

- [Rust and Cargo](https://www.rust-lang.org/tools/install)
- API key from OpenAI or Anthropic Claude

### Install from Source

### Clone the repository

```                                                                                                                                                               
git clone https://github.com/kyco/termai.git
cd termai
```

### Build and install

```
cargo install --path .
```

## 🔧 Configuration

Set up TermAI with your API keys:

### Configure OpenAI API

```                                                                                                                                                               
termai --chat-gpt-api-key YOUR_OPENAI_API_KEY
```

### Or configure Claude API

```
termai --claude-api-key YOUR_CLAUDE_API_KEY
```

### Set your preferred provider

```                                                                                                                                                               
termai --provider claude  # or openapi
```

## 📖 Usage

### Terminal Interface

The default mode provides an interactive terminal interface:

```
termai
```

You can also explicitly specify the UI mode:

```
termai --ui
```

**Controls:**
- `Tab`: Cycle through areas (Sessions → Chat → Input)
- `↑↓←→`: Navigate within focused area
- `Enter`: Edit input (when focused) or send message
- `Esc`: Exit edit mode
- `Ctrl+N`: Create new session
- `Mouse`: Click to focus, scroll to navigate

### Command Line Mode

Provide input directly for CLI mode:

```
# Ask a simple question
termai "What is the capital of France?"

# Get coding advice
termai "How do I implement binary search in Rust?"

# Use with pipes for processing command output
git status | termai "Explain what these git changes mean"
```

### Using Local Context

```
# Create a README for your project
termai "Create a README for this project" .

# Generate tests for a specific file
termai "Write unit tests for this file" ./src/main.rs

# Provide explanations for complex code
termai "Explain what this function does" ./path/to/complex_code.rs
```

### Working with Git

### Generate a commit message from your changes

```
git diff | termai "Write a concise git commit message"
```

### Explain a complex git diff

```                                                                                                                                                               
git show | termai "Explain what changes were made in this commit"
```

### Session Management

### Start or continue a named session

```
termai --session my_project "Tell me about Rust's ownership model"
```

### List all saved sessions

```                                                                                                                                                               
termai --sessions-all
```

### Privacy Features

### Add text to be automatically redacted

```
termai --redact-add "supersecretpassword"
```

### List current redactions

```                                                                                                                                                               
termai --redact-list
```

### Remove a redaction

```
termai --redact-remove "supersecretpassword"
```

## 🏗️ Architecture

TermAI is built with a clean architecture focusing on:

- **Repository Pattern**: Data access through well-defined interfaces
- **Service Layer**: Business logic separated from presentation
- **Modular Design**: Support for multiple LLM providers
- **Local Storage**: SQLite for configuration and session persistence

## 🤝 Contributing

Contributions are welcome! Here's how to get started:

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request

Please ensure your code follows the project's coding style and includes appropriate tests.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔮 Future Plans

- Response streaming
- Shell completion plugins
- Additional LLM providers
- Customization options

---                                                                                                                                                                                                                

Made with ❤️ by [kyco](https://github.com/kyco)    
