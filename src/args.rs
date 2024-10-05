use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub chat_gpt_api_key: Option<String>,
    #[arg(short, long)]
    pub print_config: bool,
}
