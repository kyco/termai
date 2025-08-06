use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use crate::setup::validator::{ApiKeyValidator, ClaudeValidator, OpenAIValidator};
use anyhow::Result;
use colored::*;
use console::Term;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Provider {
    Claude,
    OpenAI,
    Both,
}

impl Provider {
    pub fn description(&self) -> &'static str {
        match self {
            Provider::Claude => "Claude (Anthropic) - Best for analysis & coding",
            Provider::OpenAI => "OpenAI - Versatile general purpose",
            Provider::Both => "Both providers (recommended)",
        }
    }
}

pub struct SetupWizard {
    theme: ColorfulTheme,
    term: Term,
}

impl SetupWizard {
    pub fn new() -> Self {
        Self {
            theme: ColorfulTheme::default(),
            term: Term::stdout(),
        }
    }

    pub async fn run<R: ConfigRepository>(&self, repo: &R) -> Result<()> {
        self.show_welcome()?;
        
        // Step 1: Provider Selection
        let provider = self.select_provider()?;
        
        // Step 2: API Key Configuration
        match &provider {
            Provider::Claude => {
                let api_key = self.get_claude_api_key().await?;
                self.save_claude_config(repo, &api_key)?;
                self.set_provider(repo, "claude")?;
            }
            Provider::OpenAI => {
                let api_key = self.get_openai_api_key().await?;
                self.save_openai_config(repo, &api_key)?;
                self.set_provider(repo, "openapi")?;
            }
            Provider::Both => {
                let claude_key = self.get_claude_api_key().await?;
                let openai_key = self.get_openai_api_key().await?;
                self.save_claude_config(repo, &claude_key)?;
                self.save_openai_config(repo, &openai_key)?;
                
                // Ask which to use as default
                let default_provider = self.select_default_provider()?;
                self.set_provider(repo, &default_provider)?;
            }
        }
        
        // Step 3: Setup Complete
        self.show_completion()?;
        
        Ok(())
    }

    fn show_welcome(&self) -> Result<()> {
        self.term.clear_screen()?;
        
        println!("{}", "🚀 Welcome to TermAI Setup!".bright_cyan().bold());
        println!();
        println!("This wizard will help you configure TermAI with your AI providers.");
        println!("The setup should take less than 2 minutes.");
        println!();
        
        if !Confirm::with_theme(&self.theme)
            .with_prompt("Ready to get started?")
            .default(true)
            .interact()?
        {
            println!("Setup cancelled. You can run 'termai setup' anytime.");
            return Ok(());
        }
        
        Ok(())
    }

    fn select_provider(&self) -> Result<Provider> {
        println!();
        println!("{}", "Step 1 of 3: Choose Your AI Provider".bright_yellow().bold());
        println!();

        let providers = vec![
            Provider::Claude,
            Provider::OpenAI, 
            Provider::Both,
        ];
        
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Which AI provider would you like to use?")
            .items(&providers.iter().map(|p| p.description()).collect::<Vec<_>>())
            .default(2) // Default to "Both"
            .interact()?;
            
        Ok(providers[selection].clone())
    }

    async fn get_claude_api_key(&self) -> Result<String> {
        println!();
        println!("{}", "🔑 Claude API Key Configuration".bright_green().bold());
        println!();
        println!("To get your Claude API key:");
        println!("1. Visit: {}", "https://console.anthropic.com/".bright_blue().underline());
        println!("2. Sign up or log in to your account");
        println!("3. Navigate to 'API Keys' and create a new key");
        println!();
        
        loop {
            let api_key: String = Input::with_theme(&self.theme)
                .with_prompt("Enter your Claude API key")
                .interact_text()?;
                
            if api_key.trim().is_empty() {
                println!("{}", "API key cannot be empty. Please try again.".red());
                continue;
            }
            
            // Validate the API key
            println!();
            println!("🔍 Validating your Claude API key...");
            
            let validator = ClaudeValidator::new();
            let pb = self.create_progress_bar();
            
            match validator.validate(&api_key).await {
                Ok(_) => {
                    pb.finish_with_message("✅ Claude API key validated successfully!");
                    return Ok(api_key);
                }
                Err(e) => {
                    pb.finish_with_message("❌ Validation failed");
                    println!("{}", format!("Error: {}", e).red());
                    
                    if !Confirm::with_theme(&self.theme)
                        .with_prompt("Would you like to try again?")
                        .default(true)
                        .interact()?
                    {
                        return Err(anyhow::anyhow!("Setup cancelled"));
                    }
                }
            }
        }
    }

    async fn get_openai_api_key(&self) -> Result<String> {
        println!();
        println!("{}", "🔑 OpenAI API Key Configuration".bright_green().bold());
        println!();
        println!("To get your OpenAI API key:");
        println!("1. Visit: {}", "https://platform.openai.com/api-keys".bright_blue().underline());
        println!("2. Sign up or log in to your account");
        println!("3. Click 'Create new secret key'");
        println!();
        
        loop {
            let api_key: String = Input::with_theme(&self.theme)
                .with_prompt("Enter your OpenAI API key")
                .interact_text()?;
                
            if api_key.trim().is_empty() {
                println!("{}", "API key cannot be empty. Please try again.".red());
                continue;
            }
            
            // Validate the API key
            println!();
            println!("🔍 Validating your OpenAI API key...");
            
            let validator = OpenAIValidator::new();
            let pb = self.create_progress_bar();
            
            match validator.validate(&api_key).await {
                Ok(_) => {
                    pb.finish_with_message("✅ OpenAI API key validated successfully!");
                    return Ok(api_key);
                }
                Err(e) => {
                    pb.finish_with_message("❌ Validation failed");
                    println!("{}", format!("Error: {}", e).red());
                    
                    if !Confirm::with_theme(&self.theme)
                        .with_prompt("Would you like to try again?")
                        .default(true)
                        .interact()?
                    {
                        return Err(anyhow::anyhow!("Setup cancelled"));
                    }
                }
            }
        }
    }

    fn select_default_provider(&self) -> Result<String> {
        println!();
        println!("{}", "Choose Default Provider".bright_yellow().bold());
        println!("Since you configured both providers, which would you like to use as default?");
        println!();
        
        let providers = vec!["Claude", "OpenAI"];
        let selection = Select::with_theme(&self.theme)
            .with_prompt("Default provider")
            .items(&providers)
            .default(0) // Default to Claude
            .interact()?;
            
        Ok(match selection {
            0 => "claude".to_string(),
            1 => "openapi".to_string(),
            _ => "claude".to_string(),
        })
    }

    fn save_claude_config<R: ConfigRepository>(&self, repo: &R, api_key: &str) -> Result<()> {
        config_service::write_config(repo, &ConfigKeys::ClaudeApiKey.to_key(), api_key)?;
        Ok(())
    }

    fn save_openai_config<R: ConfigRepository>(&self, repo: &R, api_key: &str) -> Result<()> {
        config_service::write_config(repo, &ConfigKeys::ChatGptApiKey.to_key(), api_key)?;
        Ok(())
    }

    fn set_provider<R: ConfigRepository>(&self, repo: &R, provider: &str) -> Result<()> {
        config_service::write_config(repo, &ConfigKeys::ProviderKey.to_key(), provider)?;
        Ok(())
    }

    fn show_completion(&self) -> Result<()> {
        println!();
        println!("{}", "🎉 Setup Complete!".bright_green().bold());
        println!();
        println!("TermAI has been configured successfully. You can now:");
        println!("• Start a chat session: {}", "termai chat".bright_cyan());
        println!("• Ask a quick question: {}", "termai ask \"your question\"".bright_cyan());
        println!("• View your configuration: {}", "termai config show".bright_cyan());
        println!();
        println!("Need help? Run {} for more information.", "termai --help".bright_cyan());
        println!();
        
        Ok(())
    }

    fn create_progress_bar(&self) -> ProgressBar {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap());
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    }
}