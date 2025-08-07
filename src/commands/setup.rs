/// Handler for the Setup command - interactive configuration wizard
use crate::args::SetupArgs;
use crate::repository::db::SqliteRepository;
use crate::setup::wizard::SetupWizard;
use anyhow::Result;
use colored::*;

/// Handle the setup command to run the interactive configuration wizard
pub async fn handle_setup_command(repo: &SqliteRepository, args: &SetupArgs) -> Result<()> {
    if args.force {
        println!("{}", "🔄 Forcing reconfiguration...".bright_yellow());
    }

    if args.skip_validation {
        println!("{}", "⚠️  Skipping API key validation (test mode)".yellow());
    }

    if args.auto_accept {
        println!(
            "{}",
            "⚡ Auto-accepting defaults (non-interactive mode)".bright_cyan()
        );
    }

    let wizard = SetupWizard::new();
    wizard.run(repo).await
}
