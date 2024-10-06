use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub chat_gpt_api_key: Option<String>,
    #[arg(long)]
    pub redact_add: Option<String>,
    #[arg(long)]
    pub redact_remove: Option<String>,
    #[arg(long)]
    pub redact_list: bool,
    #[arg(short, long)]
    pub print_config: bool,
    pub data: Option<String>,
    pub(crate) directory: Option<String>,
    #[arg(short, long, value_delimiter = ',')]
    pub(crate) exclude: Vec<String>,
}

impl Args {
    pub fn is_redaction(&self) -> bool {
        self.redact_add.is_some() || self.redact_remove.is_some() || self.redact_list
    }

    pub fn is_chat_gpt_api_key(&self) -> bool {
        self.chat_gpt_api_key.is_some()
    }
}