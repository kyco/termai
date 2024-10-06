use crate::config::model::keys::ConfigKeys;
use crate::config::repository::ConfigRepository;
use crate::config::service::config_service;
use anyhow::Result;
use crate::args::Args;

pub fn fetch_redactions<R: ConfigRepository>(repo: &R) -> Vec<String> {
    match config_service::fetch_by_key(repo, &ConfigKeys::Redacted.to_key()) {
        Ok(config) => {
            let redacted: Vec<&str> = config.value.split(',').collect();
            redacted.iter().map(|redacted| redacted.to_string()).collect()
        }
        Err(_) => vec![]
    }
}

pub fn redaction<R: ConfigRepository>(repo: &R, args: &Args) -> Result<()> {
    if let Some(ref redacted_add) = args.redact_add {
        add_redaction(repo, redacted_add)
    } else if let Some(ref redacted_remove) = args.redact_remove {
        remove_redaction(repo, redacted_remove)
    } else if args.redact_list {
        print_redacted(repo)
    } else {
        Ok(())
    }
}

fn add_redaction<R: ConfigRepository>(repo: &R, redacted_add: &String) -> Result<()> {
    let existing_redacted: String = match config_service::fetch_by_key(repo, &ConfigKeys::Redacted.to_key()) {
        Ok(config) => format!("{},{}", config.value, redacted_add),
        Err(_) => redacted_add.clone(),
    };
    config_service::write_config(
        repo,
        &ConfigKeys::Redacted.to_key(),
        &existing_redacted,
    )
}

fn remove_redaction<R: ConfigRepository>(repo: &R, redacted_remove: &String) -> Result<()> {
    let existing_redacted: String = match config_service::fetch_by_key(repo, &ConfigKeys::Redacted.to_key()) {
        Ok(config) => config.value,
        Err(_) => "".to_string(),
    };
    let redacted_remove = existing_redacted.split(',').filter(|&x| x != redacted_remove).collect::<Vec<&str>>().join(",");
    config_service::write_config(
        repo,
        &ConfigKeys::Redacted.to_key(),
        &redacted_remove,
    )
}

fn print_redacted<R: ConfigRepository>(repo: &R) -> Result<()> {
    match config_service::fetch_by_key(repo, &ConfigKeys::Redacted.to_key()) {
        Ok(config) => {
            let redacted: Vec<&str> = config.value.split(',').collect();
            redacted.iter().for_each(|redacted| println!("{:}", redacted));
            Ok(())
        }
        Err(err) => {
            println!("failed to fetch redacted {:?}", err);
            Ok(())
        }
    }
}