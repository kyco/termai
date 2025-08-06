# Task: Enhanced Output Formatting

## Overview
Improve the output experience with better formatting, streaming responses, multiple export formats, and enhanced visual presentation.

## Success Criteria
- [ ] Responses feel more interactive with streaming
- [ ] Code syntax highlighting works for 20+ languages
- [ ] Export formats enable seamless workflow integration
- [ ] Visual improvements increase readability by 40%
- [ ] Enhanced output formatting showcased in README.md with visual examples

## Implementation Tasks

### 1. Streaming Response Display
- [ ] Implement token-by-token streaming for real-time feedback
- [ ] Add typing indicators and progress animations
- [ ] Create smooth scrolling and text rendering
- [ ] Handle network interruptions gracefully during streaming
- [ ] Add streaming controls (pause/resume/cancel)
- [ ] Support streaming in both chat and one-shot modes

### 2. Enhanced Syntax Highlighting
- [ ] Extend language support beyond Rust to 20+ languages
- [ ] Add intelligent language detection from code blocks
- [ ] Implement theme customization (dark/light/custom)
- [ ] Support nested language highlighting (e.g., SQL in Python)
- [ ] Add line numbers and line highlighting options
- [ ] Create language-specific formatting rules

### 3. Rich Text Formatting
- [ ] Implement Markdown rendering improvements:
  - [ ] Better table formatting and alignment
  - [ ] Enhanced list rendering (nested, ordered, unordered)
  - [ ] Improved quote block styling
  - [ ] Link formatting and click handling
- [ ] Add emoji and Unicode symbol support
- [ ] Create consistent typography and spacing
- [ ] Implement text wrapping and justification options

### 4. Multiple Export Formats
- [ ] **Markdown Export**
  - [ ] Clean Markdown with proper formatting
  - [ ] Include metadata headers and footers
  - [ ] Support for GitHub-flavored Markdown
- [ ] **HTML Export**
  - [ ] Styled HTML with CSS for web viewing
  - [ ] Embedded syntax highlighting
  - [ ] Responsive design for mobile viewing
- [ ] **PDF Export** (optional)
  - [ ] Professional formatting for sharing
  - [ ] Code block preservation
  - [ ] Custom styling and branding
- [ ] **JSON/YAML Export**
  - [ ] Structured data for programmatic access
  - [ ] Preserve conversation metadata
  - [ ] API-friendly format

### 5. Interactive Output Features
- [ ] Add collapsible code blocks and sections
- [ ] Implement copy-to-clipboard functionality
- [ ] Create inline code execution buttons (where safe)
- [ ] Add response rating and feedback options
- [ ] Support output filtering and search within responses

### 6. Visual Enhancement System
- [ ] Create consistent color scheme and theming
- [ ] Add progress bars for long operations
- [ ] Implement better error message formatting
- [ ] Add visual separators and section breaks
- [ ] Create status indicators and icons
- [ ] Support custom terminal capabilities detection

### 7. Code Block Enhancements
- [ ] Add language badges and indicators
- [ ] Implement diff highlighting for code changes
- [ ] Add code execution status indicators
- [ ] Support code folding and expansion
- [ ] Create code block metadata (file names, line numbers)
- [ ] Add vulnerability and quality indicators

### 8. Browser Preview Integration
- [ ] Create local server for HTML preview
- [ ] Add `--preview browser` option for complex responses
- [ ] Generate temporary HTML files for viewing
- [ ] Support live refresh during streaming
- [ ] Add print-friendly formatting options

### 9. Accessibility Improvements
- [ ] Add screen reader support and ARIA labels
- [ ] Implement high contrast mode options
- [ ] Support keyboard navigation for interactive elements
- [ ] Add text scaling options for different displays
- [ ] Create audio feedback options (optional)

### 10. Output Customization
- [ ] Add user preference system for formatting
- [ ] Support output templates and themes
- [ ] Create project-specific output configurations
- [ ] Add output filtering and content selection
- [ ] Support custom CSS injection for HTML exports

### 11. Performance Optimization
- [ ] Optimize rendering for large responses
- [ ] Implement lazy loading for long conversations
- [ ] Add efficient diff calculation for streaming
- [ ] Create output caching for repeated requests
- [ ] Optimize memory usage during rendering

### 12. Testing
- [ ] Unit tests for formatting and rendering logic
- [ ] Integration tests for different export formats
- [ ] Performance tests with large responses
- [ ] Cross-platform terminal compatibility tests
- [ ] Accessibility testing with screen readers

### 13. Documentation
- [ ] Create output formatting user guide
- [ ] Document export format options and use cases
- [ ] Add customization and theming guide
- [ ] Create troubleshooting guide for display issues
- [ ] Document accessibility features

## File Changes Required

### New Files
- `src/output/streaming.rs` - Streaming response implementation
- `src/output/export.rs` - Export format handlers
- `src/output/themes.rs` - Theme and styling system
- `src/output/browser.rs` - Browser preview functionality
- `src/output/accessibility.rs` - Accessibility features

### Modified Files
- `src/output/outputter.rs` - Enhanced output formatting
- `src/output/message.rs` - Rich message structure
- `src/llm/*/service/chat.rs` - Integration with streaming
- `Cargo.toml` - Add formatting dependencies

## Dependencies to Add
```toml
[dependencies]
tokio-stream = "0.1"    # Async streaming
pulldown-cmark = "0.10" # Markdown processing  
syntect = "5.0"         # Enhanced syntax highlighting
comrak = "0.21"         # GitHub-flavored Markdown
html2text = "0.6"       # HTML to text conversion
wkhtmltopdf = "0.5"     # PDF generation (optional)
tiny_http = "0.12"      # Local server for preview
```

## Command Examples

### Streaming Responses
```bash
# Stream response with typing indicator
termai ask "Explain async/await in Rust"
> ⌨️  Assistant is typing...
> 
> Async/await in Rust is a powerful concurrency model that allows you to write 
> asynchronous code that looks and feels like synchronous code. Here's how it works:
> 
> ```rust
> async fn fetch_data() -> Result<String, Error> {
>     let response = reqwest::get("https://api.example.com/data").await?;
>     let text = response.text().await?;
>     Ok(text)
> }
> ```
> [Response continues streaming...]
```

### Enhanced Export Options
```bash
# Export with multiple formats
termai ask "Create a REST API design" --export markdown --file api-design.md
termai session export api-session --format html --preview browser

# Export with custom styling
termai ask "Code review this file" src/main.rs --export pdf --theme dark
```

### Rich Formatting Display
```rust
// Example of enhanced output display
┌─ Code Review Results ────────────────────────────────────────┐
│                                                              │
│ 📁 src/main.rs                                              │
│                                                              │  
│ ✅ Strengths:                                               │
│   • Good error handling patterns                            │
│   • Clear function naming                                   │
│   • Appropriate use of Result types                         │
│                                                              │
│ ⚠️  Areas for Improvement:                                  │
│                                                              │
│   Line 42-45:                                              │
│   ```rust                                                   │
│   let result = some_function().unwrap();  // ← Risky!      │
│   ```                                                       │
│   💡 Suggestion: Use proper error handling instead          │
│                                                              │
│ 🔧 Refactoring Opportunities:                              │
│   • Extract complex logic in main() to separate functions   │
│   • Consider using builder pattern for configuration        │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

### Interactive Features
```bash
# Response with interactive elements
termai ask "Generate unit tests" src/calculator.rs
> 📝 Generated 5 unit tests for Calculator module:
> 
> [📋 Copy All] [💾 Save to File] [🔧 Modify Tests] [✨ Generate More]
> 
> ```rust
> #[cfg(test)]
> mod tests {
>     use super::*;
>     
>     #[test]  [📋 Copy]
>     fn test_add() {
>         // ... test implementation
>     }
> }
> ```
```

## Advanced Features

### Theme System
```toml
# ~/.config/termai/themes/custom.toml  
[colors]
primary = "#7C3AED"
success = "#059669"
warning = "#D97706"
error = "#DC2626"
text = "#F9FAFB"
background = "#111827"

[syntax]
keyword = "#8B5CF6"
string = "#10B981" 
comment = "#6B7280"
function = "#3B82F6"
```

### Browser Preview Mode
```bash
termai ask "Create component architecture diagram" --preview browser
> 🌐 Opening browser preview at http://localhost:8080
> 📊 Generated interactive diagram with:
>   • Clickable components
>   • Zoom and pan functionality  
>   • Export options (PNG, SVG, PDF)
```

### Accessibility Features
```bash
# High contrast mode for accessibility
termai config set display.accessibility true
termai config set display.contrast high

# Screen reader friendly output
termai ask "Explain this code" --format screen-reader
```

## Success Metrics
- Streaming satisfaction: >90% users prefer streaming over instant display
- Export usage: >40% of responses exported to external formats
- Syntax highlighting accuracy: >95% correct language detection
- Visual clarity improvement: 40% better readability scores
- Accessibility compliance: WCAG 2.1 AA standard compliance

## Risk Mitigation
- **Risk**: Streaming performance on slow connections
  - **Mitigation**: Adaptive streaming with fallback to instant display
- **Risk**: Export format compatibility issues
  - **Mitigation**: Extensive testing across different platforms and viewers
- **Risk**: Terminal compatibility problems
  - **Mitigation**: Graceful degradation for unsupported terminals**Note**: Backwards compatibility is explicitly not a concern for this implementation.
