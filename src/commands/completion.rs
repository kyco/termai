/// Handler for shell completion generation
use crate::args::{CompletionAction, CompletionArgs, CompletionShell};
use crate::completion::DynamicCompleter;
use anyhow::Result;
use clap::Command;
use clap_complete::{generate, Generator, Shell};
use colored::*;
use std::io;

/// Handle completion subcommands
pub fn handle_completion_command(
    action: &CompletionAction,
    _args: &CompletionArgs,
    cmd: &mut Command,
) -> Result<()> {
    match action {
        CompletionAction::Bash => {
            println!(
                "{}",
                "🐚 Generating Bash completion script".bright_green().bold()
            );
            println!("{}", "═══════════════════════════════════".white().dimmed());
            println!();
            generate_completion(Shell::Bash, cmd);
            print_completion_instructions("bash", "~/.bashrc", "source <(termai completion bash)");
        }
        CompletionAction::Zsh => {
            println!(
                "{}",
                "🐚 Generating Zsh completion script".bright_green().bold()
            );
            println!("{}", "══════════════════════════════════".white().dimmed());
            println!();
            generate_completion(Shell::Zsh, cmd);
            print_completion_instructions("zsh", "~/.zshrc", "source <(termai completion zsh)");
        }
        CompletionAction::Fish => {
            println!(
                "{}",
                "🐚 Generating Fish completion script".bright_green().bold()
            );
            println!("{}", "═══════════════════════════════════".white().dimmed());
            println!();
            generate_completion(Shell::Fish, cmd);
            print_completion_instructions(
                "fish",
                "~/.config/fish/config.fish",
                "termai completion fish | source",
            );
        }
        CompletionAction::PowerShell => {
            println!(
                "{}",
                "🐚 Generating PowerShell completion script"
                    .bright_green()
                    .bold()
            );
            println!(
                "{}",
                "═════════════════════════════════════".white().dimmed()
            );
            println!();
            generate_completion(Shell::PowerShell, cmd);
            print_completion_instructions(
                "PowerShell",
                "$PROFILE",
                "Invoke-Expression (& termai completion powershell)",
            );
        }
        CompletionAction::Enhanced { shell } => {
            let shell_name = match shell {
                CompletionShell::Bash => "bash",
                CompletionShell::Zsh => "zsh",
                CompletionShell::Fish => "fish",
            };

            println!(
                "{}",
                format!(
                    "🚀 Generating Enhanced {} Completion",
                    shell_name.to_uppercase()
                )
                .bright_green()
                .bold()
            );
            println!(
                "{}",
                "═══════════════════════════════════════════"
                    .white()
                    .dimmed()
            );
            println!();
            println!("{}", "Features:".bright_yellow().bold());
            println!("   • Dynamic session name completion");
            println!("   • Context-aware argument suggestions");
            println!("   • Smart directory and pattern completion");
            println!("   • Model and provider name completion");
            println!();

            let script = DynamicCompleter::generate_enhanced_completion_script(shell_name);
            print!("{}", script);

            println!();
            print_enhanced_completion_instructions(shell_name);
        }
    }
    Ok(())
}

/// Generate completion script for a specific shell
fn generate_completion<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

/// Print installation instructions for the completion script
fn print_completion_instructions(shell: &str, config_file: &str, command: &str) {
    println!(
        "\n{}",
        format!("💡 Installation Instructions for {}:", shell)
            .bright_yellow()
            .bold()
    );
    println!(
        "{}",
        "═══════════════════════════════════════════"
            .white()
            .dimmed()
    );
    println!();
    println!(
        "{}",
        "Option 1: Direct sourcing (temporary)".bright_cyan().bold()
    );
    println!("   {}", command.cyan());
    println!();
    println!(
        "{}",
        "Option 2: Save to file (permanent)".bright_cyan().bold()
    );
    println!("   # Save completion script to a file");
    println!(
        "   {}",
        format!(
            "termai completion {} > ~/.termai-completion.{}",
            shell.to_lowercase(),
            shell.to_lowercase()
        )
        .cyan()
    );
    println!();
    println!("   # Add to your shell configuration");
    println!(
        "   {}",
        format!(
            "echo 'source ~/.termai-completion.{}' >> {}",
            shell.to_lowercase(),
            config_file
        )
        .cyan()
    );
    println!();
    println!("   # Reload your shell configuration");
    println!("   {}", format!("source {}", config_file).cyan());
    println!();
    println!(
        "{}",
        "Option 3: System-wide installation".bright_cyan().bold()
    );
    match shell {
        "bash" => {
            println!("   # For system-wide bash completions:");
            println!(
                "   {}",
                "sudo termai completion bash > /etc/bash_completion.d/termai".cyan()
            );
        }
        "zsh" => {
            println!("   # For system-wide zsh completions:");
            println!(
                "   {}",
                "sudo mkdir -p /usr/local/share/zsh/site-functions".cyan()
            );
            println!(
                "   {}",
                "sudo termai completion zsh > /usr/local/share/zsh/site-functions/_termai".cyan()
            );
        }
        "fish" => {
            println!("   # For system-wide fish completions:");
            println!(
                "   {}",
                "sudo termai completion fish > /usr/share/fish/vendor_completions.d/termai.fish"
                    .cyan()
            );
        }
        "PowerShell" => {
            println!("   # For PowerShell profile-wide completions:");
            println!("   {}", "Add-Content -Path $PROFILE -Value \"Invoke-Expression (& termai completion powershell)\"".cyan());
        }
        _ => {}
    }
    println!();
    println!(
        "{}",
        "✨ Features enabled with completion:".bright_green().bold()
    );
    println!("   • Tab completion for all commands and subcommands");
    println!("   • Automatic completion of session names");
    println!("   • File path completion for context arguments");
    println!("   • Provider and option completion");
    println!();
    println!(
        "{}",
        "💡 Test completion after installation:"
            .bright_yellow()
            .bold()
    );
    println!("   {}", "termai <TAB><TAB>".cyan());
    println!("   {}", "termai config <TAB><TAB>".cyan());
    println!("   {}", "termai session <TAB><TAB>".cyan());
}

/// Print enhanced completion installation instructions
fn print_enhanced_completion_instructions(shell: &str) {
    println!(
        "{}",
        format!("💡 Enhanced Installation for {}:", shell)
            .bright_yellow()
            .bold()
    );
    println!(
        "{}",
        "═════════════════════════════════════".white().dimmed()
    );
    println!();

    match shell {
        "bash" => {
            println!(
                "{}",
                "Step 1: Save the completion script".bright_cyan().bold()
            );
            println!(
                "   {}",
                "termai completion enhanced bash > ~/.termai-completion.bash".cyan()
            );
            println!();
            println!(
                "{}",
                "Step 2: Source in your ~/.bashrc".bright_cyan().bold()
            );
            println!(
                "   {}",
                "echo 'source ~/.termai-completion.bash' >> ~/.bashrc".cyan()
            );
            println!();
            println!("{}", "Step 3: Reload your shell".bright_cyan().bold());
            println!("   {}", "source ~/.bashrc".cyan());
        }
        "zsh" => {
            println!(
                "{}",
                "Step 1: Save the completion script".bright_cyan().bold()
            );
            println!(
                "   {}",
                "termai completion enhanced zsh > ~/.termai-completion.zsh".cyan()
            );
            println!();
            println!("{}", "Step 2: Source in your ~/.zshrc".bright_cyan().bold());
            println!(
                "   {}",
                "echo 'source ~/.termai-completion.zsh' >> ~/.zshrc".cyan()
            );
            println!();
            println!("{}", "Step 3: Reload your shell".bright_cyan().bold());
            println!("   {}", "source ~/.zshrc".cyan());
        }
        "fish" => {
            println!(
                "{}",
                "Step 1: Save the completion script".bright_cyan().bold()
            );
            println!(
                "   {}",
                "termai completion enhanced fish > ~/.config/fish/completions/termai.fish".cyan()
            );
            println!();
            println!("{}", "Step 2: Restart Fish shell".bright_cyan().bold());
            println!("   {}", "exec fish".cyan());
        }
        _ => {}
    }

    println!();
    println!(
        "{}",
        "✨ Enhanced Features Available:".bright_green().bold()
    );
    println!(
        "   • {} - Tab complete with your actual session names",
        "termai session delete <TAB>".bright_white()
    );
    println!(
        "   • {} - Complete provider names",
        "termai config set-provider <TAB>".bright_white()
    );
    println!(
        "   • {} - Common exclude patterns",
        "termai ask --exclude <TAB>".bright_white()
    );
    println!(
        "   • {} - Context directory suggestions",
        "termai chat --directory <TAB>".bright_white()
    );

    println!();
    println!("{}", "🔧 Troubleshooting:".bright_yellow().bold());
    println!("   • Make sure TermAI is in your PATH");
    println!("   • Ensure the completion script has proper permissions");
    println!("   • Try {} for testing", "termai <TAB><TAB>".cyan());
}
