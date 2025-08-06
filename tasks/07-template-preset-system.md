# Task: Template & Preset System

## Overview
Create a reusable template and preset system for common AI interactions, enabling users to standardize workflows and share best practices.

## Success Criteria
- [ ] 60% of repeat users utilize templates for common tasks
- [ ] Template usage reduces time to effective prompts by 50%
- [ ] Team template sharing improves consistency across projects
- [ ] Built-in presets cover 80% of common developer use cases
- [ ] Template and preset system highlighted in README.md as a key efficiency feature

## Implementation Tasks

### 1. Template System Architecture
- [ ] Create `Template` struct for storing reusable prompt templates
- [ ] Design template variable substitution system
- [ ] Implement template inheritance and composition
- [ ] Add template validation and error handling
- [ ] Support conditional template sections

### 2. Preset Management
- [ ] Create `Preset` struct combining templates with configuration
- [ ] Implement preset creation, editing, and deletion
- [ ] Add preset categorization and tagging
- [ ] Support preset versioning and updates
- [ ] Add preset sharing and import/export

### 3. Built-in Preset Library
- [ ] Create **Code Review** preset
  - [ ] Security vulnerability analysis
  - [ ] Performance optimization suggestions
  - [ ] Code style and maintainability review
- [ ] Create **Documentation** preset
  - [ ] API documentation generation
  - [ ] README file creation
  - [ ] Code comment generation
- [ ] Create **Testing** preset
  - [ ] Unit test generation
  - [ ] Test case suggestions
  - [ ] Mock data creation
- [ ] Create **Debugging** preset
  - [ ] Error analysis and solutions
  - [ ] Log analysis and interpretation
  - [ ] Performance profiling assistance
- [ ] Create **Refactoring** preset
  - [ ] Code improvement suggestions
  - [ ] Design pattern recommendations
  - [ ] Architecture optimization

### 4. Template Variable System
- [ ] Implement variable placeholder syntax (e.g., `{{variable}}`)
- [ ] Add support for different variable types:
  - [ ] Simple text substitution
  - [ ] File content inclusion
  - [ ] Environment variable expansion
  - [ ] Date/time formatting
  - [ ] Git repository information
- [ ] Create variable validation and type checking
- [ ] Add default values and optional variables

### 5. Interactive Template Usage
- [ ] Create `termai preset use <name>` command
- [ ] Implement interactive variable prompting
- [ ] Add template preview before execution
- [ ] Support template modification during use
- [ ] Add template suggestion based on context

### 6. Preset Creation and Management
- [ ] Implement `termai preset create` command
- [ ] Add preset editing capabilities
- [ ] Create preset validation and testing
- [ ] Support preset templates for creating new presets
- [ ] Add preset usage statistics and analytics

### 7. Team and Community Features
- [ ] Add preset sharing via files or URLs
- [ ] Create preset marketplace/repository concept
- [ ] Support team preset libraries
- [ ] Add preset collaboration and review workflows
- [ ] Implement preset rating and feedback system

### 8. Advanced Template Features
- [ ] Add conditional logic in templates (if/else)
- [ ] Support loops and iterations in templates
- [ ] Create template composition and includes
- [ ] Add template testing and validation framework
- [ ] Support multi-language templates

### 9. Integration with Existing Features
- [ ] Integrate presets with smart context discovery
- [ ] Connect templates with session management
- [ ] Add preset suggestions in interactive chat mode
- [ ] Support preset-based conversation branching
- [ ] Integrate with Git workflows for commit/review presets

### 10. Configuration and Customization
- [ ] Add global preset configuration options
- [ ] Support project-specific preset libraries
- [ ] Create preset auto-discovery from `.termai/` directories
- [ ] Add preset precedence and override rules
- [ ] Support preset environment-specific variations

### 11. Testing
- [ ] Unit tests for template parsing and substitution
- [ ] Integration tests for preset workflow
- [ ] Template validation and error handling tests
- [ ] Performance tests with complex templates
- [ ] User acceptance tests for preset usability

### 12. Documentation
- [ ] Create comprehensive preset user guide
- [ ] Document template syntax and variables
- [ ] Add preset creation best practices
- [ ] Create examples for different use cases
- [ ] Document community preset sharing

## File Changes Required

### New Files
- `src/preset/mod.rs` - Preset system module
- `src/preset/template.rs` - Template parsing and rendering
- `src/preset/manager.rs` - Preset management operations
- `src/preset/builtin.rs` - Built-in preset definitions
- `src/preset/variables.rs` - Variable system implementation
- `presets/` - Directory for built-in preset files

### Modified Files
- `src/main.rs` - Add preset subcommands
- `src/args.rs` - Add preset command arguments
- `src/chat/interactive.rs` - Integrate preset suggestions
- `Cargo.toml` - Add template engine dependencies

## Dependencies to Add
```toml
[dependencies]
handlebars = "5.1"     # Template engine
serde_yaml = "0.9"     # Preset file format
directories = "5.0"    # User directories for presets
```

## Preset File Format
```yaml
# presets/code-review.yaml
name: "Code Review Assistant"
description: "Comprehensive code review with security and performance focus"
category: "development"
version: "1.0"

template: |
  Please review the following code for:
  
  {{#if security}}
  🔒 **Security Issues:**
  - Look for potential vulnerabilities
  - Check input validation and sanitization
  - Review authentication and authorization
  {{/if}}
  
  {{#if performance}}  
  ⚡ **Performance Concerns:**
  - Identify bottlenecks and inefficiencies
  - Suggest optimization opportunities
  - Review algorithmic complexity
  {{/if}}
  
  {{#if maintainability}}
  🔧 **Maintainability:**
  - Code clarity and readability
  - Documentation completeness
  - Design pattern usage
  {{/if}}
  
  Context: {{context_description}}
  Files to review:
  {{file_content}}

variables:
  security:
    type: boolean
    default: true
    description: "Include security analysis"
  performance:
    type: boolean  
    default: true
    description: "Include performance analysis"
  maintainability:
    type: boolean
    default: true
    description: "Include maintainability analysis"
  context_description:
    type: string
    required: false
    default: "General code review"
    description: "Additional context for the review"

config:
  provider: "claude"  # Preferred provider
  max_tokens: 4000
  temperature: 0.3
```

## Command Examples

### Using Built-in Presets
```bash
# Use code review preset with current git changes
termai preset use code-review --git-staged
> 🔍 Code Review Assistant
> 
> Include security analysis? [Y/n]: y
> Include performance analysis? [Y/n]: y  
> Include maintainability analysis? [Y/n]: y
> Additional context: Adding OAuth authentication
> 
> 📝 Analyzing 3 staged files...

# Quick preset usage with defaults
termai preset use documentation src/api/
```

### Creating Custom Presets
```bash
# Create new preset interactively
termai preset create "bug-analysis"
> 📋 Creating new preset: bug-analysis
> 
> Description: Analyze and debug software issues
> Category [debugging]: 
> Template file: /Users/name/.termai/presets/bug-analysis.yaml
> 
> ✅ Preset created! Edit with: termai preset edit bug-analysis

# Create preset from current conversation
termai preset create "api-design" --from-session current
```

### Managing Presets
```bash
# List available presets
termai preset list
┌─────────────────┬─────────────┬─────────────────┬─────────┐
│ Preset          │ Category    │ Description     │ Usage   │
├─────────────────┼─────────────┼─────────────────┼─────────┤
│ code-review     │ development │ Code analysis   │ 127×    │
│ documentation   │ writing     │ Doc generation  │ 43×     │
│ unit-testing    │ testing     │ Test creation   │ 38×     │
│ bug-analysis    │ debugging   │ Issue analysis  │ 12×     │
└─────────────────┴─────────────┴─────────────────┴─────────┘

# Search presets
termai preset search "test"
> 🔍 Found 2 presets matching "test":
> unit-testing - Generate comprehensive unit tests
> integration-testing - Create integration test suites
```

### Team Preset Sharing
```bash
# Export preset for sharing
termai preset export code-review --file team-code-review.yaml

# Import team preset
termai preset import team-code-review.yaml

# Sync team presets from repository
termai preset sync --team-repo https://github.com/company/termai-presets
```

## Built-in Preset Examples

### Code Review Preset
- Security vulnerability analysis
- Performance optimization suggestions  
- Code style and maintainability review
- Documentation completeness check

### Documentation Preset
- API documentation generation
- README file creation
- Code comment suggestions
- Architecture documentation

### Testing Preset
- Unit test generation
- Test case suggestions
- Mock data creation
- Test coverage analysis

### Debugging Preset  
- Error analysis and solutions
- Log interpretation
- Performance profiling
- Stack trace analysis

## Success Metrics
- Preset adoption rate: >60% of regular users
- Time to effective prompt: 50% reduction with presets
- Template reuse: >5 uses per successful template
- Community contribution: >10 user-created presets monthly
- Consistency improvement: 40% reduction in prompt variations for same tasks

## Risk Mitigation
- **Risk**: Template complexity overwhelming users
  - **Mitigation**: Start with simple presets, progressive complexity
- **Risk**: Preset maintenance burden
  - **Mitigation**: Community-driven preset development and curation  
- **Risk**: Template security issues (code injection)
  - **Mitigation**: Sandboxed template execution, input validation**Note**: Backwards compatibility is explicitly not a concern for this implementation.
