use termai::ui::markdown::{MarkdownDisplay, MarkdownResult};

#[test]
fn test_markdown_display_creation() -> MarkdownResult<()> {
    let _display = MarkdownDisplay::new()?;
    Ok(())
}

#[test]
fn test_basic_markdown_rendering() -> MarkdownResult<()> {
    let mut display = MarkdownDisplay::new()?;
    
    let markdown = "# Hello World\n\nThis is **bold** text.";
    let rendered = display.render_to_text(markdown)?;
    
    assert!(!rendered.lines.is_empty());
    Ok(())
}

#[test]
fn test_code_block_rendering() -> MarkdownResult<()> {
    let mut display = MarkdownDisplay::new()?;
    
    let markdown = r#"
Here's some Rust code:

```rust
fn main() {
    println!("Hello, world!");
}
```
"#;
    
    let rendered = display.render_to_text(markdown)?;
    
    // Should have multiple lines including the syntax highlighted code
    assert!(rendered.lines.len() > 5);
    
    // Look for the language label line
    let has_rust_label = rendered.lines.iter().any(|line| {
        line.spans.iter().any(|span| span.content.contains("rust"))
    });
    
    assert!(has_rust_label, "Should contain rust language label");
    
    Ok(())
}

#[test]
fn test_error_handling() {
    let mut display = MarkdownDisplay::new().unwrap();
    
    // Even with malformed markdown, it should not panic
    let result = display.render_to_text("```\nunclosed code block");
    assert!(result.is_ok());
}

#[test]
fn test_multiple_languages() -> MarkdownResult<()> {
    let mut display = MarkdownDisplay::new()?;
    
    let markdown = r#"
Python example:
```python
def hello():
    return "world"
```

JavaScript example:
```javascript
function hello() {
    return "world";
}
```
"#;
    
    let rendered = display.render_to_text(markdown)?;
    
    // Should contain both language labels
    let content = rendered.lines.iter()
        .flat_map(|line| line.spans.iter())
        .map(|span| span.content.as_ref())
        .collect::<String>();
    
    assert!(content.contains("python"));
    assert!(content.contains("javascript"));
    
    Ok(())
}

#[test]
fn test_text_formatting() -> MarkdownResult<()> {
    let mut display = MarkdownDisplay::new()?;
    
    let markdown = "This is **bold**, *italic*, and `code`.";
    let rendered = display.render_to_text(markdown)?;
    
    // Should successfully render without errors
    assert!(!rendered.lines.is_empty());
    
    Ok(())
}