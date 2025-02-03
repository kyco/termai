use crate::output::message::Message;
use colored::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

pub fn print(messages: Vec<Message>) {
    println!();

    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let theme = &ts.themes["base16-ocean.dark"];

    for message in messages {
        match message.role.to_string().as_str() {
            "user" => print!("{}", "user".green().bold()),
            "system" => print!("{}", "system".cyan().bold()),
            "assistant" => print!("{}", "assistant".magenta().bold()),
            _ => print!("{}", message.role.to_string().yellow().bold()),
        }
        println!(":");

        let lines = message.message.split('\n');
        let mut in_code = false;
        let mut h = HighlightLines::new(ps.find_syntax_by_extension("rs").unwrap(), theme);

        for line in lines {
            if line.trim_start().starts_with("```") {
                in_code = !in_code;
                if in_code {
                    println!(
                        "{}",
                        "────────────────────────────────────".white().dimmed()
                    );
                } else {
                    println!(
                        "{}",
                        "────────────────────────────────────".white().dimmed()
                    );
                }
                continue;
            }

            if in_code {
                let ranges: Vec<(Style, &str)> = h.highlight_line(line, &ps).unwrap();
                let escaped = as_24_bit_terminal_escaped(&ranges, false);
                println!("{}", escaped);
            } else {
                println!("{}", line.white());
            }
        }
        println!();
    }
}
