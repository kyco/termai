# TermAI

> A powerful, privacy-focused AI assistant for your terminal

TermAI is a versatile command-line AI assistant built in Rust that brings the power of modern large language models directly to your terminal. It supports both OpenAI and Anthropic
Claude APIs (now with Claude Opus 4 support) with a focus on privacy, speed, and developer productivity.

![Terminal AI Assistant](https://img.shields.io/badge/Terminal-AI_Assistant-blueviolet) ![Smart Context](https://img.shields.io/badge/🧠_Smart-Context_Discovery-brightgreen) ![License: MIT](https://img.shields.io/badge/License-MIT-green.svg) ![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)

## ✨ Features

- **🎯 Interactive Setup Wizard**: Get started in under 2 minutes with guided configuration
- **🤖 Multi-Provider Support**: Works with both OpenAI and Claude APIs
- **🚀 Claude Opus 4**: Now powered by Anthropic's most capable model with superior intelligence
- **🧠 Smart Context Discovery**: Revolutionary intelligent project analysis that automatically selects the most relevant files
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

# 🧠 Smart Context Discovery (auto-selects relevant files!)
termai --smart-context "Refactor this to use async/await" .

# Traditional local context
termai "Create a README for this project" .

# Work with specific files
termai "Explain this function" ./src/main.rs
```

## 🧠 Smart Context Discovery

**The game-changer for developer productivity.** Smart Context Discovery automatically analyzes your project and intelligently selects the most relevant files for your AI assistant, eliminating the need to manually specify files and ensuring optimal token usage.

### 🎯 Before vs After

**❌ Before (Manual Context):**
```bash
# You had to guess which files were relevant
termai "Add error handling to the user authentication" \
  ./src/auth/login.rs \
  ./src/auth/middleware.rs \
  ./src/auth/tokens.rs \
  ./src/errors.rs \
  ./src/models/user.rs
# Often missing important files or including irrelevant ones
```

**✅ After (Smart Context Discovery):**
```bash
# AI automatically finds ALL relevant files
termai --smart-context "Add error handling to the user authentication" .

# 🧠 Smart Context automatically discovered:
# ✓ auth/login.rs (entry point, 95% relevance)  
# ✓ auth/middleware.rs (dependency, 87% relevance)
# ✓ auth/tokens.rs (dependency, 82% relevance) 
# ✓ errors.rs (related functionality, 78% relevance)
# ✓ models/user.rs (data model, 71% relevance)
# ✗ Excluded: tests, docs, config files (not relevant to request)
```

### ⚡ Key Benefits

- **🎯 90%+ Accuracy**: Finds the right files automatically using advanced relevance scoring
- **🚀 10x Faster**: No more manually hunting for relevant files
- **💡 Token Optimized**: Stays within API limits while including maximum relevant context
- **🔍 Multi-Language**: Supports Rust, JavaScript/TypeScript, Python, Go, Java, Kotlin projects
- **📊 Intelligent Ranking**: Prioritizes entry points, recent changes, and dependency relationships
- **⚙️ Configurable**: Use `.termai.toml` to customize patterns and preferences

### 🛠️ How It Works

Smart Context Discovery uses sophisticated algorithms to analyze your project:

1. **🔍 Project Detection**: Automatically identifies project type (Rust, JS, Python, etc.)
2. **📁 File Discovery**: Scans project structure and identifies important files
3. **🧮 Relevance Scoring**: Analyzes file importance using multiple factors:
   - **Recent Changes**: Git history and staging status
   - **Entry Points**: main.rs, index.js, __init__.py, etc.
   - **Dependencies**: Cross-file import/reference analysis  
   - **File Types**: Prioritizes source code over config/docs
   - **Query Matching**: Keywords in your request vs file content
4. **🎯 Smart Selection**: Optimally selects files within token budget
5. **⚖️ Token Management**: Automatically chunks large projects across multiple requests

### 🚀 Usage Examples

#### Basic Smart Context
```bash
# Automatically find relevant files for any task
termai --smart-context "Optimize database queries" .
termai --smart-context "Add logging to error handlers" .
termai --smart-context "Implement user permissions" .
```

#### Large Projects (Automatic Chunking)
```bash
# Works even with massive codebases - chunks intelligently
termai --smart-context "Review security vulnerabilities" /path/to/large-project

# 📊 Smart Context found 847 files (12,450 tokens)
# ⚖️ Exceeds budget (4,000 tokens) - using chunking strategy
# 🎯 Chunk 1/3: Core auth & security modules (3,892 tokens)
# 🎯 Chunk 2/3: API endpoints & middleware (3,745 tokens)  
# 🎯 Chunk 3/3: Database & validation logic (3,234 tokens)
```

#### Custom Configuration
```bash
# Create .termai.toml in your project root
cat > .termai.toml << EOF
[context]
max_tokens = 6000
include = ["src/**/*.rs", "tests/**/*.rs"]
exclude = ["target/**", "*.log"]
priority_patterns = ["main.rs", "lib.rs", "mod.rs"]

[project]  
type = "rust"
entry_points = ["src/main.rs", "src/lib.rs"]
EOF

# Smart context will respect your configuration
termai --smart-context "Add comprehensive error handling" .
```

### 🎛️ Advanced Features

#### Context Preview
```bash
# See what files would be selected before processing
termai --smart-context --preview "Add authentication" .

# 📊 Smart Context Selection Summary
# ═══════════════════════════════════
# 🎯 Selected 8 files
# 📝 Estimated tokens: ~3,247
# 💾 Token budget: 4,000
# 
# 📁 Selected Files (by relevance):
#  1. ████████████ (89.2%) main.rs
#     💬 /src/main.rs
#     🏷️ EntryPoint, RecentlyModified
#  2. ██████████ (76.5%) auth.rs  
#     💬 /src/auth.rs
#     🏷️ QueryRelevant, Dependency
```

#### Multi-Session for Large Projects
```bash
# Automatically manages conversation across multiple sessions
termai --smart-context --session auth_refactor "Modernize authentication system" .

# Creates linked sessions for comprehensive project analysis:
# Session 1: auth_refactor_core (authentication logic)
# Session 2: auth_refactor_api (API endpoints) 
# Session 3: auth_refactor_db (database integration)
```

#### Supported Project Types

| Language | Project Files | Entry Points | Dependencies |
|----------|---------------|--------------|-------------|
| **Rust** | Cargo.toml | main.rs, lib.rs | mod declarations, use statements |
| **JavaScript/TypeScript** | package.json | index.js/ts, main.js/ts | import/require statements |
| **Python** | pyproject.toml, setup.py | main.py, __init__.py | import statements |
| **Go** | go.mod | main.go | import declarations |
| **Java** | pom.xml, build.gradle | Main.java, Application.java | import statements |
| **Kotlin** | build.gradle.kts | Main.kt, Application.kt | import declarations |

### ⚙️ Configuration Options

Create a `.termai.toml` file in your project root:

```toml
[context]
max_tokens = 4000              # Token budget per request
include = ["src/**/*.rs"]      # Files to include (glob patterns)  
exclude = ["target/**"]        # Files to exclude (glob patterns)
priority_patterns = ["main.rs"] # High-priority file patterns
enable_cache = true            # Cache analysis results

[project]
type = "rust"                  # Override project type detection
entry_points = ["src/main.rs"] # Override entry point detection
```

---

## 📋 Commands

### Smart Context Discovery

```bash
termai --smart-context "your query" .              # 🧠 Auto-select relevant files
termai --smart-context --preview "query" .         # Preview file selection
termai --smart-context --max-tokens 6000 "query" . # Custom token limit
termai --smart-context --session name "query" .    # Use with session management
```

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

```bash
# Ask a simple question
termai "What is the capital of France?"

# Get coding advice
termai "How do I implement binary search in Rust?"
```

### 🧠 Smart Context Discovery (Recommended)

```bash
# Let AI automatically find relevant files
termai --smart-context "Add authentication to the user service" .

# Optimize large codebases with automatic chunking
termai --smart-context "Review and improve error handling" .

# Preview what files would be selected
termai --smart-context --preview "Refactor database queries" .
```

### Traditional Local Context

```bash
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
- Additional LLM providers (Gemini, Cohere, etc.)
- Custom fine-tuned models
- Enhanced smart context with semantic code analysis
- Team collaboration features for shared context templates

---                                                                                                                                                                                                                

Made with ❤️ by [kyco](https://github.com/kyco)    