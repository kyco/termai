use crate::output::outputter::EnhancedOutputter;
use crate::output::message::Message;
use crate::output::streaming::{StreamingRenderer, StreamingConfig};
use crate::output::export::{ExportFormat, quick_export_message};
use crate::output::themes::ThemeManager;
use crate::output::syntax::SyntaxHighlighter;
use crate::llm::common::model::role::Role;
use anyhow::Result;

/// Demo function to showcase the enhanced output formatting features
pub async fn demo_enhanced_output() -> Result<()> {
    println!("🚀 TermAI Enhanced Output Formatting Demo\n");

    // 1. Theme Manager Demo
    demo_theme_manager().await?;
    
    // 2. Streaming Output Demo
    demo_streaming().await?;
    
    // 3. Syntax Highlighting Demo
    demo_syntax_highlighting().await?;
    
    // 4. Enhanced Messages Demo
    demo_enhanced_messages().await?;
    
    // 5. Export Demo
    demo_export().await?;

    println!("✅ Demo completed successfully!");
    Ok(())
}

async fn demo_theme_manager() -> Result<()> {
    println!("🎨 Theme Manager Demo");
    println!("{}", "=".repeat(50));
    
    let theme_manager = ThemeManager::new();
    
    // Show different message types with theming
    println!("{}", theme_manager.format_role("USER", Role::User));
    println!("{}", theme_manager.format_role("ASSISTANT", Role::Assistant));
    println!("{}", theme_manager.format_role("SYSTEM", Role::System));
    
    println!("{}", theme_manager.format_success("Operation completed successfully"));
    println!("{}", theme_manager.format_warning("This is a warning message"));
    println!("{}", theme_manager.format_error("An error occurred"));
    println!("{}", theme_manager.format_info("Here's some helpful information"));
    
    // Show themed box
    let box_content = theme_manager.create_box("Sample Box", "This is content inside a themed box.\nIt supports multiple lines.", Some(60));
    println!("{}", box_content);
    
    println!();
    Ok(())
}

async fn demo_streaming() -> Result<()> {
    println!("⚡ Streaming Output Demo");
    println!("{}", "=".repeat(50));
    
    let mut renderer = StreamingRenderer::new(StreamingConfig {
        chars_per_batch: 2,
        batch_delay_ms: 20,
        show_typing_indicator: true,
        enable_smooth_scrolling: true,
        min_content_length: 10,
    });
    
    let sample_text = "This is a demonstration of streaming text output. Watch as each character appears with a typewriter effect, making the AI response feel more interactive and engaging.";
    
    renderer.stream_text(sample_text, Some("🤖 AI: ")).await?;
    
    println!("\nNow let's demo a streaming code block:");
    
    let sample_code = r#"fn main() {
    println!("Hello, streaming world!");
    let numbers = vec![1, 2, 3, 4, 5];
    for num in numbers {
        println!("Number: {}", num);
    }
}"#;
    
    renderer.stream_code_block(sample_code, Some("rust"), None).await?;
    
    println!();
    Ok(())
}

async fn demo_syntax_highlighting() -> Result<()> {
    println!("🌈 Syntax Highlighting Demo");
    println!("{}", "=".repeat(50));
    
    let highlighter = SyntaxHighlighter::new();
    
    // Show available languages
    let languages = highlighter.supported_languages();
    println!("Supported languages: {}", languages.len());
    for (i, lang) in languages.iter().take(10).enumerate() {
        print!("{}", lang);
        if i < 9 { print!(", "); }
    }
    println!("...\n");
    
    // Demo different languages
    let rust_code = r#"use std::collections::HashMap;

fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    let mut cache = HashMap::new();
    println!("Fibonacci(10) = {}", fibonacci(10));
}"#;
    
    println!("Rust Code with Syntax Highlighting:");
    println!("┌─ Rust ─────────────────────────────────────┐");
    match highlighter.highlight(rust_code, Some("rust")) {
        Ok(highlighted) => {
            for line in highlighted.lines() {
                println!("│ {}", line);
            }
        }
        Err(_) => println!("│ (Highlighting failed, showing raw code)"),
    }
    println!("└─────────────────────────────────────────────┘\n");
    
    let python_code = r#"import numpy as np
from datetime import datetime

class DataProcessor:
    def __init__(self, data):
        self.data = np.array(data)
        self.timestamp = datetime.now()
    
    def process(self):
        # Apply some transformations
        result = np.mean(self.data) * 2
        return result

if __name__ == "__main__":
    processor = DataProcessor([1, 2, 3, 4, 5])
    print(f"Result: {processor.process()}")"#;
    
    println!("Python Code with Language Detection:");
    if let Some(detected) = highlighter.detect_language(python_code, None) {
        println!("Detected language: {}", detected.name);
    }
    
    println!();
    Ok(())
}

async fn demo_enhanced_messages() -> Result<()> {
    println!("💬 Enhanced Message Display Demo");  
    println!("{}", "=".repeat(50));
    
    let mut outputter = EnhancedOutputter::new();
    
    let messages = vec![
        Message {
            role: Role::User,
            message: "Can you explain async/await in Rust with a code example?".to_string(),
        },
        Message {
            role: Role::Assistant,
            message: r#"Async/await in Rust provides a way to write asynchronous code that looks and feels like synchronous code. Here's how it works:

```rust
use tokio::time::{sleep, Duration};

async fn fetch_data() -> Result<String, Box<dyn std::error::Error>> {
    println!("Starting data fetch...");
    
    // Simulate async work
    sleep(Duration::from_secs(1)).await;
    
    Ok("Data fetched successfully!".to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = fetch_data().await?;
    println!("{}", result);
    Ok(())
}
```

Key points:
- `async fn` declares an asynchronous function
- `.await` waits for async operations to complete  
- Functions return `Future` trait objects
- Use `#[tokio::main]` for the async runtime

This allows you to write non-blocking code that's easy to read and maintain!"#.to_string(),
        },
    ];
    
    // Disable streaming for cleaner demo output
    outputter.set_streaming(false);
    outputter.print_messages(messages).await?;
    
    println!();
    Ok(())
}

async fn demo_export() -> Result<()> {
    println!("📁 Export Functionality Demo");
    println!("{}", "=".repeat(50));
    
    let sample_content = r#"Here's a sample response with code:

```rust
fn hello_world() {
    println!("Hello from TermAI!");
}
```

And a simple table:

| Feature | Status |
|---------|--------|
| Streaming | ✅ Complete |
| Syntax Highlighting | ✅ Complete |
| Export | ✅ Complete |
| Themes | ✅ Complete |

This demonstrates the export functionality!"#;
    
    // Create temp directory for demo exports
    let temp_dir = std::env::temp_dir().join("termai_demo");
    std::fs::create_dir_all(&temp_dir)?;
    
    // Export to different formats
    let formats = vec![
        (ExportFormat::Markdown, "demo.md"),
        (ExportFormat::Html, "demo.html"),
        (ExportFormat::Json, "demo.json"),
        (ExportFormat::PlainText, "demo.txt"),
    ];
    
    for (format, filename) in formats {
        let output_path = temp_dir.join(filename);
        match quick_export_message(sample_content, &Role::Assistant, format, &output_path) {
            Ok(()) => println!("✅ Exported to: {}", output_path.display()),
            Err(e) => println!("❌ Failed to export {}: {}", filename, e),
        }
    }
    
    println!("\n💡 Export files created in: {}", temp_dir.display());
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_demo_components() {
        // Test individual demo components
        assert!(demo_theme_manager().await.is_ok());
        assert!(demo_syntax_highlighting().await.is_ok());
        assert!(demo_enhanced_messages().await.is_ok());
        assert!(demo_export().await.is_ok());
    }
}