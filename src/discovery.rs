/// Command discovery aids with intelligent suggestions
use crate::args::{Args, Commands};
use colored::*;
use std::collections::HashMap;

/// Command discovery system that provides intelligent suggestions
pub struct CommandDiscovery;

impl CommandDiscovery {
    /// Analyze failed command and provide helpful suggestions
    pub fn suggest_for_error(args: &Args, error_context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Analyze the command structure to provide targeted suggestions
        if let Some(command) = &args.command {
            match command {
                Commands::Ask(_) => {
                    suggestions.extend(Self::ask_command_suggestions(error_context));
                }
                Commands::Chat(_) => {
                    suggestions.extend(Self::chat_command_suggestions(error_context));
                }
                Commands::Setup(_) => {
                    suggestions.extend(Self::setup_command_suggestions(error_context));
                }
                Commands::Config { action: _, args: _ } => {
                    suggestions.extend(Self::config_command_suggestions(error_context));
                }
                Commands::Sessions { action: _, args: _ } => {
                    suggestions.extend(Self::session_command_suggestions(error_context));
                }
                Commands::Redact { action: _, args: _ } => {
                    suggestions.extend(Self::redact_command_suggestions(error_context));
                }
                Commands::Completion { action: _, args: _ } => {
                    suggestions.extend(Self::completion_command_suggestions(error_context));
                }
                Commands::Complete { args: _ } => {
                    // Internal command, no suggestions needed
                }
                Commands::Help => {
                    // Internal help command, no suggestions needed
                }
                Commands::Man {
                    output: _,
                    install_help: _,
                } => {
                    // Internal man page command, no suggestions needed
                }
                Commands::Commit(_) => {
                    suggestions.extend(Self::commit_command_suggestions(error_context));
                }
                Commands::Review(_) => {
                    suggestions.extend(Self::review_command_suggestions(error_context));
                }
                Commands::BranchSummary(_) => {
                    suggestions.extend(Self::branch_command_suggestions(error_context));
                }
                Commands::Hooks(_) => {
                    suggestions.extend(Self::hooks_command_suggestions(error_context));
                }
                Commands::Stash(_) => {
                    suggestions.extend(Self::stash_command_suggestions(error_context));
                }
                Commands::Tag(_) => {
                    suggestions.extend(Self::tag_command_suggestions(error_context));
                }
                Commands::Rebase(_) => {
                    suggestions.extend(Self::rebase_command_suggestions(error_context));
                }
                Commands::Conflicts(_) => {
                    suggestions.extend(Self::conflicts_command_suggestions(error_context));
                }
                Commands::Preset(_) => {
                    suggestions.extend(Self::preset_command_suggestions(error_context));
                }
            }
        } else {
            // No command specified, suggest getting started
            suggestions.extend(Self::getting_started_suggestions());
        }

        suggestions
    }

    /// Get helpful suggestions for new users
    pub fn getting_started_suggestions() -> Vec<String> {
        vec![
            "Run 'termai setup' for interactive configuration".to_string(),
            "Try 'termai ask \"your question\"' for quick queries".to_string(),
            "Use 'termai chat' to start an interactive conversation".to_string(),
            "See 'termai --help' for all available commands".to_string(),
        ]
    }

    /// Smart context suggestions based on current directory
    pub fn smart_context_suggestions() -> Vec<String> {
        let mut suggestions = vec![
            "Add --smart-context to automatically discover relevant files".to_string(),
            "Use --preview-context to see what files would be selected".to_string(),
        ];

        // Check if we're in a recognizable project type
        if std::path::Path::new("Cargo.toml").exists() {
            suggestions.push(
                "Detected Rust project - smart context will prioritize .rs files".to_string(),
            );
        } else if std::path::Path::new("package.json").exists() {
            suggestions.push(
                "Detected JavaScript/Node project - smart context will prioritize .js/.ts files"
                    .to_string(),
            );
        } else if std::path::Path::new("pyproject.toml").exists()
            || std::path::Path::new("setup.py").exists()
        {
            suggestions.push(
                "Detected Python project - smart context will prioritize .py files".to_string(),
            );
        } else if std::path::Path::new("go.mod").exists() {
            suggestions
                .push("Detected Go project - smart context will prioritize .go files".to_string());
        }

        suggestions
    }

    /// Session management suggestions
    pub fn session_suggestions() -> Vec<String> {
        vec![
            "Use --session NAME to organize conversations by project or topic".to_string(),
            "Run 'termai sessions list' to see all your saved conversations".to_string(),
            "Sessions are automatically saved - resume anytime with the same name".to_string(),
        ]
    }

    /// Common workflow suggestions
    pub fn workflow_suggestions() -> HashMap<&'static str, Vec<String>> {
        let mut workflows = HashMap::new();

        workflows.insert(
            "code_review",
            vec![
                "git diff | termai ask \"Review this code change\"".to_string(),
                "termai ask --smart-context \"Find potential security issues\" .".to_string(),
                "termai ask --smart-context --preview-context \"Suggest improvements\" ."
                    .to_string(),
            ],
        );

        workflows.insert(
            "documentation",
            vec![
                "termai ask --smart-context \"Generate comprehensive README\" .".to_string(),
                "termai ask --directory src \"Create API documentation\" .".to_string(),
                "termai ask \"Write unit tests for this function\" ./path/to/file.rs".to_string(),
            ],
        );

        workflows.insert(
            "learning",
            vec![
                "termai chat --session learning --smart-context".to_string(),
                "termai ask --smart-context \"Explain the project architecture\" .".to_string(),
                "termai ask \"How does X work in this codebase?\" --smart-context .".to_string(),
            ],
        );

        workflows.insert("development", vec![
            "termai chat --session dev_work --smart-context \"Let's refactor the authentication\"".to_string(),
            "termai ask --smart-context \"Add error handling to this module\" .".to_string(),
            "termai ask --smart-context --chunked-analysis \"Full code audit\" .".to_string(),
        ]);

        workflows
    }

    /// Display workflow suggestions with beautiful formatting
    pub fn display_workflow_suggestions() {
        println!("\n{}", "🤖 Common TermAI Workflows".bright_blue().bold());
        println!("{}", "═══════════════════════════".white().dimmed());

        let workflows = Self::workflow_suggestions();

        // Code Review
        println!("\n{}", "📝 Code Review".bright_green().bold());
        for suggestion in workflows.get("code_review").unwrap_or(&vec![]) {
            println!("   • {}", suggestion.cyan());
        }

        // Documentation
        println!("\n{}", "📚 Documentation".bright_green().bold());
        for suggestion in workflows.get("documentation").unwrap_or(&vec![]) {
            println!("   • {}", suggestion.cyan());
        }

        // Learning
        println!("\n{}", "🎓 Learning & Exploration".bright_green().bold());
        for suggestion in workflows.get("learning").unwrap_or(&vec![]) {
            println!("   • {}", suggestion.cyan());
        }

        // Development
        println!("\n{}", "🔧 Development".bright_green().bold());
        for suggestion in workflows.get("development").unwrap_or(&vec![]) {
            println!("   • {}", suggestion.cyan());
        }

        println!("\n{}", "💡 Pro Tips:".bright_yellow().bold());
        println!(
            "   • Use {} for automatic file discovery",
            "--smart-context".bright_white()
        );
        println!(
            "   • Create focused sessions with {} for organized work",
            "--session NAME".bright_white()
        );
        println!("   • Enable enhanced shell completion for faster command entry");
        println!(
            "   • Use {} to preview file selection",
            "--preview-context".bright_white()
        );
    }

    // Command-specific suggestion methods
    fn ask_command_suggestions(error_context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if error_context.contains("empty") || error_context.contains("question") {
            suggestions.push(
                "Wrap your question in quotes if it contains spaces: termai ask \"your question\""
                    .to_string(),
            );
            suggestions.push(
                "Example: termai ask \"How do I implement error handling in Rust?\"".to_string(),
            );
        }

        if error_context.contains("context") || error_context.contains("smart") {
            suggestions.extend(Self::smart_context_suggestions());
        }

        if error_context.contains("session") {
            suggestions.extend(Self::session_suggestions());
        }

        suggestions
    }

    fn chat_command_suggestions(error_context: &str) -> Vec<String> {
        let mut suggestions = Vec::new();

        if error_context.contains("context") || error_context.contains("smart") {
            suggestions.extend(Self::smart_context_suggestions());
        }

        if error_context.contains("session") {
            suggestions.extend(Self::session_suggestions());
        }

        suggestions.push(
            "Start with initial input: termai chat \"Let's discuss architecture\"".to_string(),
        );
        suggestions.push("Use interactive mode: termai chat (then type naturally)".to_string());

        suggestions
    }

    fn setup_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Run setup without flags for guided interactive configuration".to_string(),
            "Use --auto-accept for non-interactive setup with defaults".to_string(),
            "Use --skip-validation to bypass API key validation during setup".to_string(),
            "Setup creates configuration in ~/.config/termai/".to_string(),
        ]
    }

    fn config_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "View current settings: termai config show".to_string(),
            "Set API keys: termai config set-claude KEY or termai config set-openai KEY"
                .to_string(),
            "Set default provider: termai config set-provider claude".to_string(),
            "View environment variables: termai config env".to_string(),
            "Reset all settings: termai config reset".to_string(),
        ]
    }

    fn session_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "List all sessions: termai sessions list".to_string(),
            "View session details: termai sessions show SESSION_NAME".to_string(),
            "Delete old sessions: termai sessions delete SESSION_NAME".to_string(),
            "Sessions are automatically created when you use --session flag".to_string(),
        ]
    }

    fn redact_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Add sensitive patterns: termai redact add \"API_KEY_.*\"".to_string(),
            "Use regex patterns: termai redact add \"user_\\\\d+@company\\\\.com\"".to_string(),
            "List active patterns: termai redact list".to_string(),
            "Remove patterns: termai redact remove \"PATTERN\"".to_string(),
            "Redaction is applied automatically to all queries".to_string(),
        ]
    }

    fn completion_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Generate enhanced completion: termai completion enhanced bash".to_string(),
            "Save to file: termai completion bash > ~/.termai-completion.bash".to_string(),
            "Add to shell config: echo 'source ~/.termai-completion.bash' >> ~/.bashrc".to_string(),
            "Enhanced completion includes session names and dynamic suggestions".to_string(),
        ]
    }

    fn commit_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Ensure you're in a Git repository: check 'git status'".to_string(),
            "Stage changes first: 'git add <files>' or use --add-all".to_string(),
            "Generate commit message: termai commit".to_string(),
            "Auto-commit with generated message: termai commit --auto".to_string(),
            "Use --force to generate messages without staged changes".to_string(),
        ]
    }

    fn review_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Ensure you're in a Git repository with staged changes".to_string(),
            "Review staged changes: termai review".to_string(),
            "Focus on specific files: termai review --files \"*.rs\"".to_string(),
            "Enable security analysis: termai review --security".to_string(),
            "Enable performance analysis: termai review --performance".to_string(),
            "Output to file: termai review --output review.md --format markdown".to_string(),
        ]
    }

    fn branch_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Analyze current branch: termai branch-summary".to_string(),
            "Analyze specific branch: termai branch-summary feature/new-auth".to_string(),
            "Generate release notes: termai branch-summary --release-notes --from-tag v1.0.0"
                .to_string(),
            "Compare with main branch: termai branch-summary main".to_string(),
            "Get PR description: termai branch-summary feature/branch".to_string(),
        ]
    }

    fn hooks_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Check hook status: termai hooks status".to_string(),
            "Install all hooks: termai hooks install-all".to_string(),
            "Install specific hook: termai hooks install pre-commit".to_string(),
            "Interactive installation: termai hooks install".to_string(),
            "Uninstall hook: termai hooks uninstall pre-commit".to_string(),
        ]
    }

    fn stash_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "List all stashes: termai stash list".to_string(),
            "Create new stash: termai stash push".to_string(),
            "Apply latest stash: termai stash pop".to_string(),
            "Show stash details: termai stash show 0".to_string(),
            "Apply without removing: termai stash apply 0".to_string(),
        ]
    }

    fn tag_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "List all tags: termai tag list".to_string(),
            "Create new tag: termai tag create v1.2.3".to_string(),
            "Get tag suggestions: termai tag suggest".to_string(),
            "Show tag details: termai tag show v1.2.0".to_string(),
            "Generate release notes: termai tag release-notes --from-tag v1.1.0".to_string(),
        ]
    }

    fn rebase_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Check rebase status: termai rebase status".to_string(),
            "Start interactive rebase: termai rebase start --interactive".to_string(),
            "Plan rebase with AI: termai rebase plan --count 5".to_string(),
            "Analyze commits: termai rebase analyze".to_string(),
            "Continue interrupted rebase: termai rebase continue".to_string(),
            "Abort current rebase: termai rebase abort".to_string(),
        ]
    }

    fn conflicts_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "Detect all conflicts: termai conflicts detect".to_string(),
            "Get AI conflict analysis: termai conflicts analyze".to_string(),
            "Get resolution strategies: termai conflicts suggest".to_string(),
            "Interactive resolution wizard: termai conflicts resolve".to_string(),
            "Check conflict status: termai conflicts status".to_string(),
            "Show resolution guide: termai conflicts guide".to_string(),
        ]
    }

    fn preset_command_suggestions(_error_context: &str) -> Vec<String> {
        vec![
            "List available presets: termai preset list".to_string(),
            "Use a preset interactively: termai preset use <name>".to_string(),
            "Create a custom preset: termai preset create <name>".to_string(),
            "Show preset details: termai preset show <name>".to_string(),
            "Search presets: termai preset search <query>".to_string(),
            "Export preset: termai preset export <name> --file preset.yaml".to_string(),
            "Import preset: termai preset import preset.yaml".to_string(),
        ]
    }

    /// Display contextual help based on current directory and git status
    pub fn display_contextual_help() {
        println!("\n{}", "🔍 Context-Aware Suggestions".bright_blue().bold());
        println!("{}", "═══════════════════════════════".white().dimmed());

        // Check git status
        if let Ok(output) = std::process::Command::new("git")
            .args(["status", "--porcelain"])
            .output()
        {
            if !output.stdout.is_empty() {
                println!("\n{}", "📝 Git Changes Detected".bright_green().bold());
                println!(
                    "   • {} - Review your changes",
                    "git diff | termai ask \"Review this code\"".cyan()
                );
                println!(
                    "   • {} - Generate commit message",
                    "git diff --staged | termai ask \"Write commit message\"".cyan()
                );
            }
        }

        // Check project type and suggest relevant workflows
        if std::path::Path::new("Cargo.toml").exists() {
            println!("\n{}", "🦀 Rust Project Detected".bright_green().bold());
            println!(
                "   • {} - Analyze Rust-specific patterns",
                "termai ask --smart-context \"Review Rust best practices\" .".cyan()
            );
            println!(
                "   • {} - Optimize performance",
                "termai ask --smart-context \"Find performance bottlenecks\" .".cyan()
            );
        } else if std::path::Path::new("package.json").exists() {
            println!("\n{}", "📦 Node.js Project Detected".bright_green().bold());
            println!(
                "   • {} - Analyze JavaScript/TypeScript code",
                "termai ask --smart-context \"Review JS/TS patterns\" .".cyan()
            );
            println!(
                "   • {} - Check dependencies",
                "termai ask --smart-context \"Analyze package dependencies\" .".cyan()
            );
        } else if std::path::Path::new("pyproject.toml").exists() {
            println!("\n{}", "🐍 Python Project Detected".bright_green().bold());
            println!(
                "   • {} - Review Python code quality",
                "termai ask --smart-context \"Analyze Python best practices\" .".cyan()
            );
            println!(
                "   • {} - Check for security issues",
                "termai ask --smart-context \"Find security vulnerabilities\" .".cyan()
            );
        }

        // Check if there are many files (suggest chunked analysis)
        if let Ok(entries) = std::fs::read_dir(".") {
            let file_count = entries.count();
            if file_count > 50 {
                println!("\n{}", "📊 Large Project Detected".bright_yellow().bold());
                println!(
                    "   • {} - Handle large codebase efficiently",
                    "termai ask --smart-context --chunked-analysis \"Full analysis\" .".cyan()
                );
                println!(
                    "   • {} - Preview before processing",
                    "termai ask --smart-context --preview-context \"Check selection\" .".cyan()
                );
            }
        }
    }
}

/// Display intelligent suggestions when user needs help
pub fn display_discovery_help() {
    println!("{}", "🤖 TermAI Command Discovery".bright_blue().bold());
    println!("{}", "═════════════════════════════".white().dimmed());

    // Show getting started suggestions
    println!("\n{}", "🚀 Getting Started".bright_green().bold());
    for suggestion in CommandDiscovery::getting_started_suggestions() {
        println!("   • {}", suggestion.cyan());
    }

    // Show smart context suggestions
    println!("\n{}", "🧠 Smart Context Discovery".bright_green().bold());
    for suggestion in CommandDiscovery::smart_context_suggestions() {
        println!("   • {}", suggestion.cyan());
    }

    // Show session suggestions
    println!("\n{}", "💬 Session Management".bright_green().bold());
    for suggestion in CommandDiscovery::session_suggestions() {
        println!("   • {}", suggestion.cyan());
    }

    // Display workflow suggestions
    CommandDiscovery::display_workflow_suggestions();

    // Display contextual help based on current directory
    CommandDiscovery::display_contextual_help();

    println!("\n{}", "📚 More Help".bright_yellow().bold());
    println!(
        "   • {} - Complete command reference",
        "See COMMANDS.md".bright_white()
    );
    println!(
        "   • {} - Quick reference card",
        "See QUICK_REFERENCE.md".bright_white()
    );
    println!(
        "   • {} - Command-specific help",
        "termai COMMAND --help".bright_white()
    );
}
