use crate::chat::formatter::ChatFormatter;
use crate::llm::common::model::role::Role;
use chrono::Local;
use anyhow::Result;
use colored::*;

/// Demo function to showcase enhanced chat formatting
pub async fn demo_enhanced_chat_formatting() -> Result<()> {
    println!("🚀 Enhanced Chat Formatting Demo\n");
    
    let mut formatter = ChatFormatter::new();
    
    // Demo different types of responses that would come from an AI
    let sample_responses = [(
            "Simple response",
            "Hello! I'm your AI assistant. How can I help you today?"
        ),
        (
            "Response with code",
            r#"Here's how to create a simple Rust function:

```rust
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    println!("Fibonacci(10) = {}", fibonacci(10));
}
```

This function uses recursion to calculate Fibonacci numbers."#
        ),
        (
            "Response with table",
            r#"Here's a comparison of programming languages:

| Language | Type Safety | Performance | Learning Curve |
|----------|-------------|-------------|----------------|
| Rust | ✅ Excellent | ⚡ Very High | 🔴 Steep |
| Python | 🟡 Dynamic | 🟡 Medium | 🟢 Easy |
| JavaScript | 🟡 Improving | 🟡 Medium | 🟢 Easy |
| C++ | 🟡 Manual | ⚡ Very High | 🔴 Steep |

Each language has its strengths and use cases."#
        ),
        (
            "Response with markdown formatting",
            r#"# Project Setup Guide

## Prerequisites

Before you begin, make sure you have:

1. **Rust installed** - Get it from [rustup.rs](https://rustup.rs)
2. **Git** - For version control
3. **A good editor** - VS Code with rust-analyzer works great

## Steps

- Clone the repository
- Run `cargo build` to compile
- Use `cargo test` to run tests

> **Note**: This is just an example of how markdown formatting works in chat responses!

### Additional Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

Happy coding! 🦀"#
        ),
        (
            "Mixed content response", 
            r#"Let me explain async/await in Rust:

## What is async/await?

Async/await is a **concurrency model** that allows you to write asynchronous code that looks like synchronous code.

### Key Benefits

- Non-blocking I/O operations
- Better resource utilization
- Improved application responsiveness

Here's a simple example:

```rust
use tokio::time::{sleep, Duration};

async fn fetch_data() -> Result<String, Box<dyn std::error::Error>> {
    // Simulate network delay
    sleep(Duration::from_millis(100)).await;
    Ok("Data fetched!".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = fetch_data().await?;
    println!("{}", result);
    Ok(())
}
```

### Performance Comparison

| Operation | Sync Time | Async Time | Improvement |
|-----------|-----------|------------|-------------|
| File I/O | 100ms | 10ms | 🚀 10x faster |
| Network | 200ms | 25ms | 🚀 8x faster |
| Database | 50ms | 8ms | 🚀 6x faster |

> The key is that async operations don't block the thread while waiting!

This makes your applications much more efficient when dealing with I/O-bound workloads."#
        )];

    // Demonstrate each type of response
    for (i, (description, content)) in sample_responses.iter().enumerate() {
        println!("{}", format!("Demo {}: {}", i + 1, description).bright_magenta().bold());
        println!("{}", "─".repeat(60).bright_black());
        
        // Format as if it's an AI response
        formatter.format_message_async(
            &Role::Assistant,
            content,
            Some(Local::now()),
        ).await?;
        
        println!("{}", "─".repeat(60).bright_black());
        println!(); // Extra spacing between examples
    }

    println!("✅ Enhanced chat formatting demo completed!");
    println!();
    println!("🎨 Features demonstrated:");
    println!("  • {} Syntax highlighting for code blocks", "✅".bright_green());
    println!("  • {} Beautiful table rendering", "✅".bright_green());
    println!("  • {} Markdown headers and formatting", "✅".bright_green());
    println!("  • {} List formatting with bullets", "✅".bright_green());
    println!("  • {} Blockquote styling", "✅".bright_green());
    println!("  • {} Inline code highlighting", "✅".bright_green());
    println!("  • {} Themed role indicators", "✅".bright_green());
    println!("  • {} Streaming support (configurable)", "✅".bright_green());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enhanced_chat_demo() {
        // Test that the demo runs without errors
        assert!(demo_enhanced_chat_formatting().await.is_ok());
    }
}