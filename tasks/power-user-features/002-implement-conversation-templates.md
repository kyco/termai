# Task: Implement Conversation Templates System

## Priority: Medium
## Estimated Effort: 4-5 days
## Dependencies: None

## Overview
Add a template system that allows users to save and reuse common prompts, conversation starters, and context setups. This will help power users standardize their workflows and quickly start conversations with predefined contexts.

## Requirements

### Functional Requirements
1. **Template Types**
   - **Prompt Templates**: Reusable prompts with placeholders
   - **Context Templates**: Include files/directories automatically
   - **Conversation Starters**: Multi-turn conversation setups
   - **System Message Templates**: Custom system prompts

2. **Template Management**
   - Create templates from existing conversations
   - Edit template content and metadata
   - Organize templates by categories
   - Import/export template collections
   - Share templates between users

3. **Template Usage**
   - Quick access via keyboard shortcut (`Ctrl+T`)
   - Template browser with search
   - Variable substitution in templates
   - Preview before applying

### Technical Requirements
1. **Database Schema**
   ```sql
   CREATE TABLE templates (
       id TEXT PRIMARY KEY,
       name TEXT NOT NULL,
       description TEXT,
       category TEXT,
       template_type TEXT NOT NULL, -- 'prompt', 'context', 'conversation', 'system'
       content TEXT NOT NULL,
       variables TEXT, -- JSON array of variable definitions
       metadata TEXT, -- JSON metadata
       created_at INTEGER NOT NULL,
       updated_at INTEGER NOT NULL
   );
   
   CREATE TABLE template_categories (
       id TEXT PRIMARY KEY,
       name TEXT NOT NULL UNIQUE,
       color TEXT,
       created_at INTEGER NOT NULL
   );
   ```

2. **Template Structure**
   ```rust
   #[derive(Serialize, Deserialize, Clone)]
   pub struct Template {
       pub id: String,
       pub name: String,
       pub description: Option<String>,
       pub category: Option<String>,
       pub template_type: TemplateType,
       pub content: String,
       pub variables: Vec<TemplateVariable>,
       pub metadata: TemplateMetadata,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub enum TemplateType {
       Prompt,
       Context,
       Conversation,
       System,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub struct TemplateVariable {
       pub name: String,
       pub description: String,
       pub default_value: Option<String>,
       pub required: bool,
       pub variable_type: VariableType,
   }
   
   #[derive(Serialize, Deserialize, Clone)]
   pub enum VariableType {
       Text,
       File,
       Directory,
       Choice(Vec<String>),
   }
   ```

3. **Template Service**
   ```rust
   pub struct TemplateService {
       template_repo: Arc<TemplateRepository>,
   }
   
   impl TemplateService {
       pub async fn create_template(&self, template: Template) -> Result<()>;
       pub async fn get_templates(&self, filter: TemplateFilter) -> Result<Vec<Template>>;
       pub async fn apply_template(&self, template_id: &str, variables: HashMap<String, String>) -> Result<AppliedTemplate>;
       pub async fn create_from_conversation(&self, session_id: &str, template_info: TemplateInfo) -> Result<Template>;
   }
   ```

## Implementation Steps

1. **Database Layer**
   ```rust
   // repositories/template_repository.rs
   impl TemplateRepository {
       pub async fn create_template(&self, template: &Template) -> Result<()>;
       pub async fn get_templates_by_category(&self, category: &str) -> Result<Vec<Template>>;
       pub async fn search_templates(&self, query: &str) -> Result<Vec<Template>>;
       pub async fn get_template(&self, id: &str) -> Result<Option<Template>>;
       pub async fn update_template(&self, template: &Template) -> Result<()>;
       pub async fn delete_template(&self, id: &str) -> Result<()>;
   }
   ```

2. **Template Engine**
   ```rust
   // services/template_engine.rs
   pub struct TemplateEngine;
   
   impl TemplateEngine {
       pub fn render(&self, template: &str, variables: &HashMap<String, String>) -> Result<String> {
           // Use handlebars or similar for variable substitution
           // Support for {{variable}} syntax
           // Include file contents with {{#include "path"}}
           // Conditional blocks with {{#if condition}}
       }
       
       pub fn extract_variables(&self, template: &str) -> Vec<String> {
           // Parse template and find all variable references
       }
       
       pub fn validate_template(&self, template: &str) -> Result<()> {
           // Validate template syntax
       }
   }
   ```

3. **Template Browser UI**
   ```rust
   // In ui.rs
   pub struct TemplateBrowser {
       templates: Vec<Template>,
       selected_index: usize,
       categories: Vec<String>,
       selected_category: Option<String>,
       search_query: String,
   }
   
   impl TemplateBrowser {
       pub fn new() -> Self;
       pub fn filter_by_category(&mut self, category: Option<String>);
       pub fn search(&mut self, query: &str);
       pub fn get_selected_template(&self) -> Option<&Template>;
   }
   ```

4. **Variable Input Dialog**
   ```rust
   pub struct VariableInputDialog {
       variables: Vec<TemplateVariable>,
       values: HashMap<String, String>,
       current_field: usize,
   }
   
   impl VariableInputDialog {
       pub fn new(variables: Vec<TemplateVariable>) -> Self;
       pub fn set_value(&mut self, name: &str, value: String);
       pub fn validate(&self) -> Result<()>;
       pub fn get_values(&self) -> HashMap<String, String>;
   }
   ```

5. **Template Creation from Conversation**
   ```rust
   impl TemplateService {
       pub async fn create_from_conversation(
           &self,
           session_id: &str,
           name: String,
           description: Option<String>,
           include_messages: MessageSelection,
       ) -> Result<Template> {
           let messages = self.message_repo.get_messages(session_id).await?;
           let filtered_messages = self.filter_messages(messages, include_messages);
           
           let content = self.messages_to_template_content(filtered_messages);
           let variables = self.extract_variables_from_content(&content);
           
           let template = Template {
               id: Uuid::new_v4().to_string(),
               name,
               description,
               category: None,
               template_type: TemplateType::Conversation,
               content,
               variables,
               metadata: TemplateMetadata::default(),
               created_at: Utc::now(),
               updated_at: Utc::now(),
           };
           
           self.template_repo.create_template(&template).await?;
           Ok(template)
       }
   }
   ```

## Built-in Template Examples

1. **Code Review Template**
   ```
   Please review the following {{language}} code:
   
   {{#include "{{file_path}}"}}
   
   Focus on:
   - Code quality and best practices
   - Performance considerations
   - Security issues
   - Maintainability
   
   {{#if specific_concerns}}
   Pay special attention to: {{specific_concerns}}
   {{/if}}
   ```

2. **Bug Report Template**
   ```
   I'm experiencing a bug in my {{project_type}} project:
   
   **Expected Behavior:** {{expected_behavior}}
   **Actual Behavior:** {{actual_behavior}}
   **Steps to Reproduce:** {{steps}}
   
   {{#include "{{log_file}}"}}
   
   Environment:
   - OS: {{os}}
   - Version: {{version}}
   ```

## Testing Requirements
- Unit tests for template engine
- Integration tests for template CRUD operations
- UI tests for template browser
- Performance tests with many templates
- Template validation tests

## Acceptance Criteria
- [ ] Users can create templates from conversations
- [ ] Template browser is searchable and filterable
- [ ] Variable substitution works correctly
- [ ] File inclusion works in templates
- [ ] Templates can be organized by categories
- [ ] Export/import functionality works
- [ ] Performance is good with 100+ templates

## Future Enhancements
- Template sharing marketplace
- Version control for templates
- Template inheritance
- Conditional template logic
- Template statistics and usage tracking
- AI-assisted template generation
- Template validation and linting