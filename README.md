# TermAI

> A powerful, privacy-focused AI assistant for your terminal

TermAI is a versatile command-line AI assistant built in Rust that brings the power of modern large language models directly to your terminal. It supports both OpenAI and Anthropic
Claude APIs (now with Claude Opus 4 support) with a focus on privacy, speed, and developer productivity.

![Terminal AI Assistant](https://img.shields.io/badge/Terminal-AI_Assistant-blueviolet) ![License: MIT](https://img.shields.io/badge/License-MIT-green.svg) ![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)

## ✨ Features

- **🎯 Interactive Setup Wizard**: Get started in under 2 minutes with guided configuration
- **🤖 Multi-Provider Support**: Works with both OpenAI and Claude APIs
- **🚀 Claude Opus 4**: Now powered by Anthropic's most capable model with superior intelligence
- **📁 Local Context Understanding**: Analyze your code and files for more relevant responses
- **💬 Session Management**: Save and restore conversations for later reference
- **🔒 Privacy-Focused**: Redact sensitive information before sending to APIs
- **⚡ Developer-Optimized**: Perfect for generating code, explaining concepts, and assisting with daily dev tasks
- **🖥️ Fully Terminal-Based**: No web interfaces or external dependencies needed
- **🎨 Beautiful Interface**: Progress indicators, colors, and intuitive subcommands

## 🚀 Quick Start

### 1. Install TermAI

```bash
# Prerequisites: Rust and Cargo (https://www.rust-lang.org/tools/install)
git clone https://github.com/kyco/termai.git
cd termai
cargo install --path .
```

### 2. Interactive Setup (⭐ Recommended)

Get started instantly with our **interactive setup wizard**:

```bash
termai setup
```

The setup wizard will guide you through:
- **Provider Selection**: Choose Claude, OpenAI, or both
- **API Key Configuration**: Enter your keys with live validation
- **Default Provider**: Set your preferred AI assistant

**✨ Takes less than 2 minutes and validates your API keys automatically!**

### 3. Start Chatting

```bash
# Ask anything!
termai "What is the capital of France?"

# Use with local context
termai "Create a README for this project" .

# Work with your code
termai "Explain this function" ./src/main.rs
```

## 📋 Commands

### Setup & Configuration

```bash
termai setup                    # 🌟 Interactive setup wizard
termai config show             # View current configuration
termai config reset            # Clear all settings
termai config set-claude KEY   # Set Claude API key
termai config set-openai KEY   # Set OpenAI API key
```

### Advanced Configuration (Manual)

If you prefer manual configuration over the setup wizard:

```bash
termai --claude-api-key YOUR_CLAUDE_API_KEY    # Configure Claude
termai --chat-gpt-api-key YOUR_OPENAI_API_KEY  # Configure OpenAI  
termai --provider claude                       # Set default provider
```

## 📖 Usage

### Basic Queries

```
# Ask a simple question
termai "What is the capital of France?"

# Get coding advice
termai "How do I implement binary search in Rust?"
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

```bash
# Start or continue a named session
termai --session my_project "Tell me about Rust's ownership model"

# List all saved sessions
termai sessions list
```

### Privacy & Redaction

```bash
termai redact add "sensitive_data"     # Add text to redact
termai redact list                     # List redaction patterns
termai redact remove "sensitive_data"  # Remove a redaction pattern
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

- Stream responses for faster feedback
- Auto-completion plugins for common shells
- Voice input/output support
- Additional LLM providers
- Custom fine-tuned models

---                                                                                                                                                                                                                

Made with ❤️ by [kyco](https://github.com/kyco)    