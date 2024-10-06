use crate::output::message::Message;
use colored::*;

pub fn print(messages: Vec<Message>) {
    println!("{}", "=== Message Log ===".bold().underline().blue());
    println!();

    for message in messages {
        match message.role.to_string().as_str() {
            "user" => print!("{}", "User".green().bold()),
            "system" => print!("{}", "System".cyan().bold()),
            "assistant" => print!("{}", "Assistant".magenta().bold()),
            _ => print!("{}", message.role.to_string().yellow().bold()),
        }
        println!(":");

        let lines = message.message.split('\n');
        for line in lines {
            println!("{}", line.white());
        }
        println!();
    }

    println!("{}", "=== End of Log ===".bold().underline().blue());
}
