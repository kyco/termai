/// Git hooks management and installation
use crate::git::repository::GitRepository;
use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

/// Git hook manager for TermAI integration
pub struct HookManager {
    repo: GitRepository,
}

/// Types of Git hooks supported by TermAI
#[derive(Debug, Clone, PartialEq)]
pub enum HookType {
    PreCommit,
    CommitMsg,
    PrePush,
    PostCommit,
}

/// Hook installation status
#[derive(Debug, Clone)]
pub struct HookStatus {
    pub hook_type: HookType,
    pub installed: bool,
    pub termai_managed: bool,
    pub existing_hook: bool,
    pub path: PathBuf,
}

impl HookManager {
    /// Create a new hook manager for a Git repository
    pub fn new(repo: GitRepository) -> Self {
        Self { repo }
    }

    /// Install a TermAI hook with backup of existing hooks
    pub fn install_hook(&self, hook_type: HookType) -> Result<()> {
        let hook_path = self.get_hook_path(&hook_type);
        let hook_name = self.hook_type_to_name(&hook_type);

        println!(
            "{}",
            format!("📦 Installing {} hook...", hook_name).bright_blue()
        );

        // Check if hook already exists
        if hook_path.exists() {
            if self.is_termai_hook(&hook_path)? {
                println!(
                    "{}",
                    format!("✅ TermAI {} hook already installed", hook_name).green()
                );
                return Ok(());
            }

            // Backup existing hook
            let backup_path = hook_path.with_extension(&format!("{}.backup", hook_name));
            fs::copy(&hook_path, &backup_path).context("Failed to backup existing hook")?;
            println!(
                "{}",
                format!("📋 Backed up existing hook to {}", backup_path.display()).yellow()
            );
        }

        // Create hook script
        let hook_content = self.generate_hook_script(&hook_type)?;
        fs::write(&hook_path, hook_content).context("Failed to write hook script")?;

        // Make hook executable
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms).context("Failed to make hook executable")?;

        println!(
            "{}",
            format!("✅ {} hook installed successfully", hook_name)
                .green()
                .bold()
        );
        println!(
            "{}",
            format!("   Location: {}", hook_path.display()).dimmed()
        );

        Ok(())
    }

    /// Uninstall a TermAI hook and restore backup if exists
    pub fn uninstall_hook(&self, hook_type: HookType) -> Result<()> {
        let hook_path = self.get_hook_path(&hook_type);
        let hook_name = self.hook_type_to_name(&hook_type);

        println!(
            "{}",
            format!("🗑️  Uninstalling {} hook...", hook_name).bright_red()
        );

        if !hook_path.exists() {
            println!("{}", format!("⚠️  {} hook not found", hook_name).yellow());
            return Ok(());
        }

        if !self.is_termai_hook(&hook_path)? {
            println!(
                "{}",
                format!("⚠️  {} hook is not managed by TermAI, skipping", hook_name).yellow()
            );
            return Ok(());
        }

        // Remove TermAI hook
        fs::remove_file(&hook_path).context("Failed to remove hook")?;

        // Restore backup if exists
        let backup_path = hook_path.with_extension(&format!("{}.backup", hook_name));
        if backup_path.exists() {
            fs::rename(&backup_path, &hook_path).context("Failed to restore backup hook")?;
            println!("{}", format!("📋 Restored backup hook").green());
        }

        println!(
            "{}",
            format!("✅ {} hook uninstalled successfully", hook_name).green()
        );

        Ok(())
    }

    /// Check if a hook is installed and get its status
    pub fn get_hook_status(&self, hook_type: HookType) -> Result<HookStatus> {
        let hook_path = self.get_hook_path(&hook_type);
        let installed = hook_path.exists();
        let termai_managed = if installed {
            self.is_termai_hook(&hook_path).unwrap_or(false)
        } else {
            false
        };

        let backup_path =
            hook_path.with_extension(&format!("{}.backup", self.hook_type_to_name(&hook_type)));
        let existing_hook = backup_path.exists();

        Ok(HookStatus {
            hook_type: hook_type.clone(),
            installed,
            termai_managed,
            existing_hook,
            path: hook_path,
        })
    }

    /// Get status of all hooks
    pub fn get_all_hook_status(&self) -> Result<Vec<HookStatus>> {
        let hook_types = [
            HookType::PreCommit,
            HookType::CommitMsg,
            HookType::PrePush,
            HookType::PostCommit,
        ];
        let mut statuses = Vec::new();

        for hook_type in hook_types {
            statuses.push(self.get_hook_status(hook_type)?);
        }

        Ok(statuses)
    }

    /// Install all recommended TermAI hooks
    pub fn install_all_hooks(&self) -> Result<()> {
        println!(
            "{}",
            "🚀 Installing all TermAI Git hooks...".bright_blue().bold()
        );

        let recommended_hooks = [HookType::PreCommit, HookType::CommitMsg];

        for hook_type in recommended_hooks {
            if let Err(e) = self.install_hook(hook_type.clone()) {
                println!(
                    "{}",
                    format!(
                        "❌ Failed to install {} hook: {}",
                        self.hook_type_to_name(&hook_type),
                        e
                    )
                    .red()
                );
            }
        }

        println!("{}", "✅ Hook installation completed".green().bold());
        self.display_hook_usage();

        Ok(())
    }

    /// Display hook usage information
    pub fn display_hook_usage(&self) {
        println!("{}", "\n📚 TermAI Git Hooks Usage:".bright_cyan().bold());
        println!("{}", "═══════════════════════════".white().dimmed());
        println!(
            "• {}",
            "Pre-commit: Runs code analysis before each commit".cyan()
        );
        println!(
            "• {}",
            "Commit-msg: Validates and enhances commit messages".cyan()
        );
        println!(
            "• {}",
            "Pre-push: Performs final review before pushing".cyan()
        );
        println!();
        println!("{}", "💡 Commands:".bright_yellow().bold());
        println!(
            "   • {}",
            "termai hooks status - Check hook installation status".white()
        );
        println!(
            "   • {}",
            "termai hooks install <type> - Install specific hook".white()
        );
        println!(
            "   • {}",
            "termai hooks uninstall <type> - Remove specific hook".white()
        );
    }

    /// Get the path to a hook file
    fn get_hook_path(&self, hook_type: &HookType) -> PathBuf {
        let hooks_dir = self.repo.root_path().join(".git").join("hooks");
        hooks_dir.join(self.hook_type_to_name(hook_type))
    }

    /// Convert hook type to hook file name
    fn hook_type_to_name(&self, hook_type: &HookType) -> &'static str {
        match hook_type {
            HookType::PreCommit => "pre-commit",
            HookType::CommitMsg => "commit-msg",
            HookType::PrePush => "pre-push",
            HookType::PostCommit => "post-commit",
        }
    }

    /// Check if a hook file is managed by TermAI
    fn is_termai_hook(&self, hook_path: &Path) -> Result<bool> {
        if !hook_path.exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(hook_path).context("Failed to read hook file")?;

        Ok(content.contains("# TermAI Git Hook") || content.contains("termai"))
    }

    /// Generate hook script content
    fn generate_hook_script(&self, hook_type: &HookType) -> Result<String> {
        let termai_path = std::env::current_exe()
            .context("Failed to get TermAI executable path")?
            .display()
            .to_string();

        let script = match hook_type {
            HookType::PreCommit => format!(
                r#"#!/bin/bash
# TermAI Git Hook - Pre-commit
# This hook runs TermAI code analysis before each commit

echo "🔍 TermAI: Running pre-commit analysis..."

# Check if there are staged changes
if ! git diff --cached --quiet; then
    # Run TermAI review on staged changes
    {} review --format text
    review_exit_code=$?
    
    if [ $review_exit_code -ne 0 ]; then
        echo "❌ TermAI: Code review found issues. Commit aborted."
        echo "💡 Run 'termai review' to see detailed analysis"
        echo "💡 Use 'git commit --no-verify' to bypass this check"
        exit 1
    fi
    
    echo "✅ TermAI: Pre-commit analysis passed"
fi

exit 0
"#,
                termai_path
            ),

            HookType::CommitMsg => format!(
                r#"#!/bin/bash
# TermAI Git Hook - Commit Message
# This hook validates and can enhance commit messages

commit_msg_file="$1"
commit_msg=$(cat "$commit_msg_file")

echo "📝 TermAI: Validating commit message..."

# Check if message follows conventional commits format
if echo "$commit_msg" | grep -qE '^(feat|fix|docs|style|refactor|test|chore|build|ci)(\(.+\))?: .+'; then
    echo "✅ TermAI: Commit message follows conventional format"
else
    echo "⚠️  TermAI: Commit message doesn't follow conventional format"
    echo "💡 Consider using: feat/fix/docs/style/refactor/test/chore: description"
    echo "💡 Run 'termai commit' to generate AI-powered commit messages"
fi

exit 0
"#
            ),

            HookType::PrePush => format!(
                r#"#!/bin/bash
# TermAI Git Hook - Pre-push
# This hook runs final analysis before pushing to remote

echo "🚀 TermAI: Running pre-push analysis..."

# Check if there are unpushed commits
if [ "$(git rev-list @{{u}}..HEAD 2>/dev/null | wc -l)" -gt 0 ]; then
    echo "📊 TermAI: Analyzing unpushed commits..."
    
    # Run branch summary for unpushed changes
    {} branch-summary
    
    echo "✅ TermAI: Pre-push analysis completed"
else
    echo "ℹ️  TermAI: No unpushed commits to analyze"
fi

exit 0
"#,
                termai_path
            ),

            HookType::PostCommit => format!(
                r#"#!/bin/bash
# TermAI Git Hook - Post-commit
# This hook provides post-commit insights and suggestions

echo "🎉 TermAI: Commit successful! Analyzing commit..."

# Show commit summary
commit_hash=$(git rev-parse HEAD)
commit_msg=$(git log -1 --pretty=format:'%s')

echo "   Commit: $commit_hash"
echo "   Message: $commit_msg"

# Optional: Run analysis on the commit
# termai review --format text > /dev/null 2>&1 || true

echo "💡 Tip: Use 'termai branch-summary' to analyze your branch"

exit 0
"#
            ),
        };

        Ok(script)
    }
}

impl std::fmt::Display for HookType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookType::PreCommit => write!(f, "pre-commit"),
            HookType::CommitMsg => write!(f, "commit-msg"),
            HookType::PrePush => write!(f, "pre-push"),
            HookType::PostCommit => write!(f, "post-commit"),
        }
    }
}

impl std::fmt::Display for HookStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = if self.installed {
            if self.termai_managed {
                "✅ Installed (TermAI)".green()
            } else {
                "🔶 Installed (Custom)".yellow()
            }
        } else {
            "❌ Not Installed".red()
        };

        let backup = if self.existing_hook {
            " (has backup)".dimmed()
        } else {
            "".normal()
        };

        write!(f, "{}: {}{}", self.hook_type, status, backup)
    }
}
