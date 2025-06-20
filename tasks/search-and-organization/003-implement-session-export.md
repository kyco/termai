# Task: Implement Session Export Functionality

## Priority: High
## Estimated Effort: 2-3 days
## Dependencies: None

## Overview
Add the ability to export sessions in various formats (Markdown, JSON, PDF, HTML) to allow users to share conversations, create documentation, or archive important discussions.

## Requirements

### Functional Requirements
1. **Export Formats**
   - **Markdown**: Clean, readable format with code blocks
   - **JSON**: Complete data including metadata
   - **HTML**: Styled, standalone HTML with syntax highlighting
   - **PDF**: Via HTML rendering (optional, phase 2)

2. **Export Options**
   - Single session export
   - Bulk export multiple sessions
   - Export with or without system messages
   - Include session metadata (date, title, tags)
   - Custom header/footer text

3. **UI Integration**
   - Export option in session context menu
   - Keyboard shortcut (Ctrl+E)
   - Export progress indicator
   - Success notification with file location

### Technical Requirements
1. **Export Service**
   ```rust
   // services/export_service.rs
   pub struct ExportService {
       session_repo: Arc<SessionRepository>,
       message_repo: Arc<MessageRepository>,
   }
   
   pub enum ExportFormat {
       Markdown,
       Json,
       Html,
       Pdf,
   }
   
   impl ExportService {
       pub async fn export_session(
           &self, 
           session_id: &str, 
           format: ExportFormat,
           options: ExportOptions,
       ) -> Result<PathBuf>;
       
       pub async fn export_sessions(
           &self,
           session_ids: Vec<String>,
           format: ExportFormat,
           options: ExportOptions,
       ) -> Result<Vec<PathBuf>>;
   }
   ```

2. **Format Renderers**
   - Trait-based design for extensibility
   - Template support for HTML/Markdown
   - Syntax highlighting preservation

3. **File Management**
   - Default export directory
   - Filename sanitization
   - Overwrite protection

## Implementation Steps

1. **Create Export Module**
   ```rust
   // modules/export/mod.rs
   pub mod markdown;
   pub mod json;
   pub mod html;
   pub mod service;
   
   pub trait Exporter {
       fn export(&self, session: &Session, messages: &[Message], options: &ExportOptions) -> Result<String>;
   }
   ```

2. **Markdown Exporter**
   ```rust
   // modules/export/markdown.rs
   impl Exporter for MarkdownExporter {
       fn export(&self, session: &Session, messages: &[Message], options: &ExportOptions) -> Result<String> {
           // Format as clean markdown with:
           // - Session title as H1
           // - Metadata as frontmatter
           // - Messages with role indicators
           // - Code blocks with language tags
       }
   }
   ```

3. **UI Integration**
   - Add export action to events.rs
   - Create export dialog widget
   - Update keybindings help

## Testing Requirements
- Unit tests for each export format
- Integration tests for full export flow
- Large session export performance tests
- Character encoding tests (emoji, unicode)

## Acceptance Criteria
- [ ] Export works for all specified formats
- [ ] Exported files are valid and readable
- [ ] Code blocks maintain syntax highlighting info
- [ ] Bulk export completes successfully
- [ ] Export location is configurable
- [ ] Progress shown for large exports

## Example Outputs

### Markdown Format
```markdown
# Session: Implementing User Authentication
Date: 2024-01-15
Tags: backend, security

## Conversation

**You**: How do I implement JWT authentication in Rust?

**Assistant**: Here's how to implement JWT authentication in Rust...

```rust
use jsonwebtoken::{encode, decode, Header, Validation};
// ... code example
```
```

### JSON Format
```json
{
  "session": {
    "id": "uuid",
    "title": "Implementing User Authentication",
    "created_at": "2024-01-15T10:00:00Z",
    "tags": ["backend", "security"]
  },
  "messages": [
    {
      "role": "user",
      "content": "How do I implement JWT authentication in Rust?",
      "timestamp": "2024-01-15T10:00:00Z"
    }
  ]
}
```

## Future Enhancements
- Export templates customization
- Direct sharing to GitHub Gists
- Export to Notion/Obsidian formats
- Batch export with zip compression
- Export scheduling/automation