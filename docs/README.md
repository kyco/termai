# TermAI

> A powerful, privacy-focused AI assistant for your terminal

TermAI is a versatile command-line AI assistant built in Rust that brings the power of modern large language models directly to your terminal. It supports Anthropic Claude, OpenAI GPT-5.x, and OpenAI Codex (via ChatGPT subscription OAuth) with a focus on privacy, speed, and developer productivity.

![Terminal AI Assistant](https://img.shields.io/badge/Terminal-AI_Assistant-blueviolet) ![Smart Context](https://img.shields.io/badge/🧠_Smart-Context_Discovery-brightgreen) ![License: MIT](https://img.shields.io/badge/License-MIT-green.svg) ![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)

## ✨ Features

- **🎯 Interactive Setup Wizard**: Get started in under 2 minutes with guided configuration
- **🤖 Multi-Provider Support**: Works with Claude, OpenAI GPT-5.x, and Codex (ChatGPT OAuth)
- **🚀 Modern Models**: Claude Sonnet 4 (`claude-sonnet-4-20250514`) and GPT-5.x family (`gpt-5.4` for Codex) by default
- **🧠 Smart Context Discovery**: Revolutionary intelligent project analysis that automatically selects the most relevant files
- **🔄 AI-Powered Git Integration**: Complete Git workflow automation with intelligent commit messages, code reviews, and conflict resolution
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

- **🎯 Automatic Discovery**: Finds relevant files using relevance scoring, so you spend less time hunting for them
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

## 📋 Commands Reference

TermAI offers intuitive subcommands for all your AI assistant needs:

### 🚀 Quick Start Commands

```bash
# Interactive setup (recommended first step)
termai setup

# Ask a quick question
termai ask "How do I implement binary search in Rust?"

# Start a conversation
termai chat
termai chat "Let's discuss software architecture"

# Smart context analysis
termai chat --smart-context "Refactor this authentication system" .
```

### 🧠 Smart Context Discovery

```bash
# Automatic file discovery and analysis
termai ask --smart-context "Add error handling" .
termai chat --smart-context "Review security issues" .

# Preview what files would be selected
termai ask --smart-context --preview-context "Optimize performance" .

# Customize token limits and chunking
termai ask --smart-context --max-context-tokens 6000 "Explain the codebase" .
termai chat --smart-context --chunked-analysis --chunk-strategy hierarchical "Full code review" .

# Target specific context
termai ask --smart-context --context-query "database connections" "Fix connection pooling" .
```

### ⚙️ Configuration Management

```bash
# Interactive setup wizard
termai setup                           # Complete guided setup
termai setup --auto-accept            # Non-interactive with defaults
termai setup --skip-validation        # Skip API key validation

# View and manage configuration
termai config show                     # Display current settings

# Authenticate providers (recommended)
termai auth login claude               # Configure Claude API key
termai auth login openai               # Configure OpenAI API key
termai auth login codex                # Log in to Codex via ChatGPT OAuth
termai auth status claude              # Check authentication status
termai auth logout codex               # Remove stored credentials

# Choose models
termai config set-model                # Interactive model selector
termai config set-model gpt-5.4       # Set model directly
termai config list-models              # List available models
```

> The older `termai config set-claude/set-openai/set-provider`, `config env`, and `config reset` commands still work but are deprecated and hidden; prefer `termai auth` and `termai config set-model`.

### 🔑 Provider Authentication

```bash
termai auth login <provider>           # Log in (claude | openai | codex)
termai auth status <provider>          # Show authentication status
termai auth logout <provider>          # Remove stored credentials
```

- **claude** / **openai**: prompts for an API key and stores it locally.
- **codex**: authenticates via OAuth using your ChatGPT Plus/Pro subscription — no API key needed. See [CODEX_OAUTH.md](CODEX_OAUTH.md) for details.

You can also switch provider and model mid-conversation with the `/provider` and `/model` commands inside `termai chat`.

### 📝 Presets & Templates

Reusable prompt templates with variables, categories, and Git integration:

```bash
termai preset list                     # List available presets
termai preset use code-review .        # Run a preset with project context
termai preset create my-preset         # Create your own preset
termai preset show code-review         # Inspect a preset
```

See the [Preset Guide](preset-guide.md) and [Preset Quick Reference](preset-quick-reference.md) for the full workflow.

### 💬 Session Management

```bash
# List and manage sessions
termai sessions list                   # Show all sessions
termai sessions --limit 5 --sort date list # Show recent 5 sessions
termai sessions show session_name      # View session details
termai sessions delete session_name    # Remove a session

# Use sessions in conversations
termai chat --session my_project       # Continue specific session
termai ask --session code_review "Explain this function" ./src/lib.rs
```

#### 🌿 Conversation Branching

Sessions support git-style conversation branches, so you can explore alternative directions without losing history:

```bash
termai sessions branch my_session --name experiment   # Branch from a session
termai sessions tree my_session                       # Visualize the branch tree
termai sessions branches my_session                   # List branches
termai sessions switch my_session experiment          # Switch branches
termai sessions compare my_session main experiment    # Compare branches
termai sessions merge my_session experiment --into main # Merge branches
```

Additional branch tools: `bookmark`, `search`, `stats`, `selective-merge`, `archive`, `cleanup`, and `export`. Run `termai sessions --help` for the full list.

### 🔒 Privacy & Redaction

```bash
# Manage redaction patterns
termai redact add "SECRET_KEY"          # Add sensitive text pattern
termai redact add "user_.*@company.com" # Add regex pattern
termai redact list                      # Show all patterns
termai redact remove "SECRET_KEY"       # Remove a pattern
termai redact reset                     # Clear all patterns
```

### 🔧 Advanced Features

```bash
# Custom context directories
termai ask --directories src,tests,docs "Document this project" .
termai chat --directory src --exclude "*.test.js,node_modules" "Review the code" .

# Provider and model selection (set once, applies to ask/chat)
termai auth login openai               # Authenticate a provider
termai config set-model                # Pick the default model interactively
# ...or switch on the fly inside `termai chat` with /provider and /model

# System prompts and customization
termai ask --system-prompt "You are a Rust expert" "Optimize this code" ./main.rs
termai chat --session rust_tutor --system-prompt "Act as a patient teacher"
```

### 🐚 Shell Completion

```bash
# Generate completion scripts
termai completion bash                  # Basic Bash completion
termai completion zsh                   # Basic Zsh completion  
termai completion fish                  # Basic Fish completion
termai completion powershell            # PowerShell completion

# Enhanced completions with dynamic features
termai completion enhanced bash         # Bash with session name completion
termai completion enhanced zsh          # Zsh with smart suggestions
termai completion enhanced fish         # Fish with context-aware completion

# Installation examples
termai completion bash > ~/.termai-completion.bash
echo 'source ~/.termai-completion.bash' >> ~/.bashrc
```

### 🔍 Environment Variables

TermAI supports environment variables for API keys:

```bash
export OPENAI_API_KEY="your-key"       # OpenAI API key
export CLAUDE_API_KEY="your-key"       # Claude API key
export ANTHROPIC_API_KEY="your-key"    # Alternative to CLAUDE_API_KEY
```

These are the only supported environment variables. Provider, model, and behavior settings are managed with `termai auth`, `termai config set-model`, and `.termai.toml`.

## 📖 Usage Examples

### 🎯 Common Workflows

#### Development Assistant
```bash
# Quick coding questions
termai ask "How do I implement a thread pool in Rust?"
termai ask "Best practices for error handling in REST APIs"

# Code analysis with smart context
termai ask --smart-context "Find potential memory leaks" .
termai ask --smart-context "Suggest performance optimizations" .

# Interactive development session
termai chat --session dev_work --smart-context
# Then continue the conversation naturally
```

#### Code Review & Documentation
```bash
# Review recent changes
git diff | termai ask "Review this code change for potential issues"

# Generate commit messages
git diff --staged | termai ask "Write a clear commit message for these changes"

# Document your code
termai ask --smart-context "Generate comprehensive README documentation" .
termai ask --directory src "Create API documentation for these modules" .
```

#### Learning & Exploration  
```bash
# Understand complex codebases
termai ask --smart-context --preview-context "Explain the architecture" .
termai chat --session learning --smart-context "How does the authentication system work?" .

# Deep dive into specific topics
termai chat --session rust_patterns --system-prompt "You are a Rust expert"
```

#### Project Management
```bash
# Analyze project health
termai ask --smart-context "Identify technical debt and areas for refactoring" .
termai ask --smart-context "Suggest project structure improvements" .

# Generate project assets
termai ask --smart-context "Create a comprehensive test plan" .
termai ask --directories src,docs "Generate a contributing guide" .
```

## 🔄 AI-Powered Git Integration

TermAI revolutionizes Git workflows with intelligent automation that understands your code and context. Say goodbye to repetitive Git tasks and hello to AI-powered development workflows.

### ✨ Complete Git Workflow Automation

**🤖 Smart Commit Generation**
```bash
# AI analyzes your staged changes and generates perfect commit messages
termai commit
> 📝 Analyzing staged changes...
> 
> Suggested commit message:
> feat(auth): add OAuth2 integration with token refresh
> 
> - Add OAuth2Provider trait implementation
> - Implement token refresh mechanism  
> - Add comprehensive error handling
> - Update tests for new auth flow
> 
> [e]dit, [a]ccept, [r]egenerate, [c]ancel?

# Quick auto-commit mode
termai commit --auto
```

**🔍 Intelligent Code Review**
```bash
# Get AI-powered code review before committing
termai review
> 🔍 Reviewing staged changes...
> 
> ⚠️  Security Issues:
> src/auth.rs:42 - Use SecretString instead of String for passwords
> 
> 🚀 Performance:
> src/auth.rs:58 - Consider caching OAuth tokens to reduce API calls
> 
> ✅ Positive Findings:
> - Excellent test coverage for new functionality
> - Follows consistent error handling patterns

# Focus on specific areas
termai review --security --performance
```

**🌿 Smart Branch Management**
```bash
# Analyze your current branch with AI insights
termai branch-summary
> 📊 Branch: feature/oauth (5 commits ahead of main)
> 
> AI Analysis:
> - No breaking changes detected
> - Well-structured commits with clear progression
> - Good test coverage maintained
> 
> Suggested PR Description:
> ## OAuth2 Integration
> This PR adds comprehensive OAuth2 authentication support...

# Generate PR/MR descriptions automatically
termai branch-summary --pr-description
```

**⚔️ Conflict Resolution Assistant**
```bash
# AI-powered merge conflict analysis and resolution
termai conflicts detect
> ⚔️  Found 3 merge conflicts in 2 files
> 
> src/auth.rs:
> - Complexity: Medium (structural changes to auth module)
> - Strategy: Manual review recommended (business logic conflict)
> - Confidence: 85% - can suggest resolution approach
> 
> tests/auth_test.rs:
> - Complexity: Low (test assertion updates)  
> - Strategy: Auto-resolution possible
> - Confidence: 95% - safe to auto-resolve

# Interactive conflict resolution
termai conflicts resolve --interactive
```

**🔄 Interactive Rebase Guidance**
```bash
# AI-assisted interactive rebase with smart suggestions
termai rebase start main --count 5
> 🔄 Planning interactive rebase of last 5 commits
> 
> AI Recommendations:
> ✅ Squash commits #2 and #3 (both fix typos in same function)
> ✅ Reword commit #4 (improve message clarity)
> ⚠️  Keep commit #1 separate (substantial feature addition)
> 
> [a]pply suggestions, [m]anual edit, [c]ancel?

# Continue interrupted rebase with AI guidance
termai rebase continue
```

**🏷️ Release Management**
```bash
# AI-powered tag creation with semantic versioning
termai tag create --from-tag v1.2.0
> 🏷️  Analyzing changes since v1.2.0...
> 
> Detected Changes:
> - 3 new features (OAuth, rate limiting, caching)
> - 2 bug fixes (auth timeout, memory leak)
> - No breaking changes
> 
> Suggested Version: v1.3.0 (minor)
> 
> Generated Release Notes:
> ## v1.3.0 - Enhanced Authentication & Performance
> 
> ### 🚀 New Features
> - OAuth2 integration with multiple providers
> - Advanced rate limiting system
> - Intelligent response caching
```

**🪝 Git Hooks Automation**
```bash
# Install AI-powered Git hooks for automated quality assurance
termai hooks install-all
> 🪝 Installing TermAI Git hooks...
> ✅ Pre-commit: Code quality analysis
> ✅ Commit-msg: Message validation
> ✅ Pre-push: Final review check
> ✅ Post-commit: Success insights

# Check hooks status
termai hooks status
```

### 🎯 Smart Git Commands Reference

| Command | Purpose | AI Features |
|---------|---------|-------------|
| `termai commit` | Generate commit messages | Analyzes diffs, follows conventions, suggests scope |
| `termai review` | Code review assistance | Security scan, performance check, style validation |
| `termai branch-summary` | Branch analysis | Change summary, PR descriptions, impact analysis |
| `termai conflicts` | Merge conflict help | Strategy suggestions, complexity analysis, auto-resolution |
| `termai rebase` | Interactive rebase guide | Commit squashing, message improvements, conflict prediction |
| `termai tag` | Release management | Semantic versioning, release notes, change categorization |
| `termai hooks` | Git hooks management | Quality gates, automation, integration with existing tools |
| `termai stash` | Stash operations | Smart naming, conflict detection, restoration guidance |

### 🚀 Workflow Examples

**Daily Development Flow**
```bash
# Make changes
git add .

# AI-powered commit
termai commit
> feat(api): add user profile endpoints with validation

# AI code review
termai review
> ✅ No issues found, ready to push!

# Push with confidence
git push
```

**Feature Branch Workflow**
```bash
# Create and work on feature branch
git checkout -b feature/user-profiles

# Multiple commits...
git add . && termai commit --auto
git add . && termai commit --auto

# Clean up commits before merging
termai rebase start main --interactive
> Squashed 3 commits into coherent feature story

# Generate PR description
termai branch-summary --pr-description
> ## User Profile Management
> Comprehensive user profile system with validation...

# Create PR with generated description
```

**Release Preparation**
```bash
# Analyze changes for release
termai tag create
> Suggested version: v2.1.0
> 
> Breaking Changes: None
> New Features: 4
> Bug Fixes: 2

# Create release with AI-generated notes
git tag v2.1.0 -m "$(termai tag release-notes --from v2.0.0)"
```

### 🎨 Beautiful Terminal Experience

All Git commands feature:
- **🎨 Rich Colors**: Visual distinction for different types of information
- **📊 Progress Indicators**: Real-time feedback during analysis
- **🎯 Interactive Prompts**: Smart defaults with easy customization
- **💡 Contextual Tips**: Learning opportunities built into the workflow
- **⚡ Fast Performance**: Efficient analysis even for large repositories

### 🔧 Configuration & Customization

Create `.termai.toml` in your project root:

```toml
[git]
# Commit message preferences
commit_template = "conventional"  # conventional, minimal, detailed
auto_stage = false               # Auto-stage files before commit
require_scope = true            # Require scope in commit messages

# Code review settings  
review_depth = "standard"       # quick, standard, thorough
security_focus = true          # Enable security analysis
performance_focus = true       # Enable performance analysis

# Hook configuration
hooks_enabled = ["pre-commit", "commit-msg"]
hook_strictness = "warn"       # warn, error, off
```

---

### 🎯 Session-Based Workflows

```bash
# Start focused development sessions
termai chat --session auth_refactor --smart-context "Let's refactor the authentication system" .

# Continue multi-part conversations
termai chat --session auth_refactor  # Resumes previous conversation

# Organize work by project
termai sessions list                  # See all active sessions
termai sessions show auth_refactor    # Review session history
termai sessions delete old_session    # Clean up completed work
```

### 🔒 Privacy-First Development

```bash
# Set up redaction patterns for your organization
termai redact add "ACME_API_KEY_.*"           # Redact API keys
termai redact add "user_\d+@company\.com"     # Redact user emails  
termai redact add "password.*=.*"             # Redact password assignments

# Verify redaction is working
termai redact list                            # Check active patterns

# Safe analysis of sensitive codebases
termai ask --smart-context "Review security practices" .  # Redaction applied automatically
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

- Voice input/output support
- Additional LLM providers (Gemini, Cohere, etc.)
- Custom fine-tuned models
- Enhanced smart context with semantic code analysis
- Team collaboration features for shared context templates

---                                                                                                                                                                                                                

Made with ❤️ by [kyco](https://github.com/kyco)    