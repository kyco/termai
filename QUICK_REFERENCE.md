# TermAI Quick Reference Card

> Essential commands at your fingertips

## 🚀 Getting Started

```bash
termai setup                        # Interactive setup wizard
termai ask "your question"          # Quick one-shot question
termai chat                         # Start interactive conversation
```

## 🧠 Smart Context Discovery

```bash
# Automatic file discovery
termai ask --smart-context "question" .
termai chat --smart-context --session work

# Preview what files would be selected  
termai ask --smart-context --preview-context "query" .

# Handle large projects with chunking
termai ask --smart-context --chunked-analysis "full review" .
```

## ⚙️ Configuration

```bash
termai config show                  # View current settings
termai config set-provider claude   # Set default provider
termai config set-openai KEY        # Set OpenAI API key
termai config set-claude KEY        # Set Claude API key
termai config env                   # Show environment variables
```

## 💬 Sessions

```bash
termai sessions list                # List all sessions
termai sessions show NAME           # View session details
termai sessions delete NAME         # Remove session
termai chat --session NAME          # Use/create named session
```

## 🔒 Privacy & Redaction

```bash
termai redact add "PATTERN"         # Add redaction pattern
termai redact list                  # Show active patterns
termai redact remove "PATTERN"      # Remove pattern
```

## 🐚 Shell Completion

```bash
# Basic completions
termai completion bash | sudo tee /etc/bash_completion.d/termai
termai completion zsh | sudo tee /usr/local/share/zsh/site-functions/_termai

# Enhanced completions (dynamic session/model names)
termai completion enhanced bash > ~/.termai-completion.bash
echo 'source ~/.termai-completion.bash' >> ~/.bashrc
```

## 🔧 Common Options

| Option | Description |
|--------|-------------|
| `--smart-context` | Enable intelligent file discovery |
| `--preview-context` | Preview selected files before processing |
| `--session NAME` | Use named session |
| `--system-prompt TEXT` | Custom system prompt |
| `--provider claude\|openai` | Choose AI provider |
| `--directory DIR` | Include specific directory |
| `--exclude PATTERNS` | Exclude file patterns |
| `--max-context-tokens NUM` | Limit context size |

## 🌍 Environment Variables

```bash
export OPENAI_API_KEY="your-key"
export CLAUDE_API_KEY="your-key" 
export TERMAI_PROVIDER="claude"
export TERMAI_SMART_CONTEXT="true"
export TERMAI_MAX_CONTEXT_TOKENS="4000"
```

## 🎯 Common Workflows

### Code Review
```bash
git diff | termai ask "Review this change"
termai ask --smart-context "Find security issues" .
```

### Documentation  
```bash
termai ask --smart-context "Generate README" .
termai ask --directory src "Create API docs" .
```

### Learning
```bash
termai chat --session learning --smart-context
termai ask --smart-context "Explain architecture" .
```

### Development
```bash
termai chat --session dev --smart-context "Let's refactor authentication" .
termai ask --smart-context --chunked-analysis "Full code analysis" .
```

## 🆘 Help & Troubleshooting

```bash
termai --help                       # General help
termai COMMAND --help               # Command-specific help
termai config show                  # Verify configuration
termai sessions list                # Check active sessions
```

## 🔍 File Patterns

Smart context respects `.termai.toml` configuration:

```toml
[context]
max_tokens = 4000
include = ["src/**/*.rs", "tests/**/*.rs"]
exclude = ["target/**", "*.log"]
priority_patterns = ["main.rs", "lib.rs"]

[project]
type = "rust" 
entry_points = ["src/main.rs"]
```

---

**💡 Pro Tips:**
- Use `--smart-context` for automatic file discovery
- Create focused sessions with `--session` for organized work
- Enable enhanced shell completion for faster command entry
- Use `--preview-context` to verify file selection before processing
- Set up redaction patterns once for privacy protection across all queries

**📚 More Help:** See [COMMANDS.md](COMMANDS.md) for complete reference