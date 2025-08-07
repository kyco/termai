# TermAI Preset System User Guide

## 🎯 Overview

The TermAI Preset System allows you to create, customize, and reuse templated AI interactions. Presets combine carefully crafted prompts with variable substitution, making it easy to standardize workflows and achieve consistent results.

## 🚀 Quick Start

### Using Built-in Presets

TermAI comes with 5 production-ready presets:

```bash
# List all available presets
termai preset list

# Use a preset with current context
termai preset use "Code Review Assistant" --smart-context

# Use a preset with Git staged files
termai preset use "Code Review Assistant" --git-staged

# Use a preset with specific directory
termai preset use "Documentation Generator" src/

# Preview before execution
termai preset use "Test Generator" --preview --smart-context
```

### Built-in Presets

| Preset | Category | Purpose |
|--------|----------|---------|
| **Code Review Assistant** | Development | Comprehensive code review with security, performance, and maintainability analysis |
| **Documentation Generator** | Writing | Generate API docs, README files, and code comments |  
| **Test Generator** | Testing | Create unit tests, integration tests, and test cases |
| **Debugging Assistant** | Debugging | Analyze errors, debug issues, and provide solutions |
| **Refactoring Assistant** | Development | Suggest code improvements, design patterns, and optimization |

## 📝 Creating Custom Presets

### Interactive Creation Wizard

```bash
termai preset create "My Custom Preset"
```

The wizard will guide you through:
1. **Basic Information** - Description and category
2. **Template Content** - Your prompt template with variables
3. **Variable Definitions** - Types, descriptions, and defaults
4. **Configuration** - AI provider settings
5. **Preview & Confirmation** - Review before saving

### Template Syntax

Use Handlebars syntax for variables and logic:

```handlebars
Please {{action}} the following {{content_type}}:

{{#if include_examples}}
Include practical examples and usage patterns.
{{/if}}

{{#if file_content}}
Files to analyze:
{{file_content}}
{{else}}
Please provide the content you'd like me to analyze.
{{/if}}

Additional context: {{context}}
```

### Variable Types

| Type | Description | Example |
|------|-------------|---------|
| `String` | Simple text input | User names, descriptions |
| `Boolean` | True/false values | Feature flags, options |
| `Number` | Numeric values | Counts, limits, scores |
| `File` | File path that gets read | Configuration files |
| `Directory` | Directory listing | Project structure |
| `DateTime` | Current date/time | Timestamps, logs |
| `GitInfo` | Git repository data | Branch, commit, status |
| `Environment` | Environment variables | API keys, paths |

### Example: Custom Bug Report Analyzer

```bash
termai preset create "Bug Report Analyzer"
```

**Template:**
```handlebars
Analyze this bug report and provide:

1. **Root Cause Analysis**
   - Identify the likely cause
   - Explain technical reasoning

2. **Reproduction Steps** 
   {{#if include_repro}}
   - How to reproduce the issue
   - Required environment setup
   {{/if}}

3. **Solution Recommendations**
   - Immediate fixes
   - Long-term improvements
   - Testing approach

{{#if severity}}
**Priority Level:** {{severity}}
{{/if}}

**Bug Report:**
{{bug_description}}

{{#if file_content}}
**Related Code:**
{{file_content}}
{{/if}}
```

**Variables:**
- `bug_description` (String, required) - The bug report content
- `include_repro` (Boolean, default: true) - Include reproduction steps
- `severity` (String, optional) - Bug severity level

## ✏️ Editing Presets

### Full Editor Interface

```bash
termai preset edit "My Custom Preset"
```

Choose what to edit:
- **Template** - Modify the prompt template
- **Metadata** - Update description, category  
- **Variables** - Add, edit, or remove variables
- **Configuration** - Change AI provider settings
- **All** - Edit everything

### External Editor Support

For complex templates, use your preferred editor:

```bash
termai preset edit "My Custom Preset" --template
# Choose "Edit in external editor"
```

Supports `$EDITOR` and `$VISUAL` environment variables.

### Built-in Preset Customization

```bash
termai preset edit "Code Review Assistant"
# Creates custom copy: "Code Review Assistant (Custom)"
```

Built-in presets are protected, but you can create customizable copies.

## 🔧 Advanced Features

### Smart Context Integration

```bash
# Automatically select relevant files
termai preset use "Code Review Assistant" --smart-context

# Combine with directory specification
termai preset use "Documentation Generator" --smart-context src/api/

# Preview context selection
termai preset use "Test Generator" --smart-context --preview
```

### Git Workflow Integration

```bash
# Review staged changes
termai preset use "Code Review Assistant" --git-staged

# Combine with smart context for enhanced analysis
termai preset use "Code Review Assistant" --git-staged --smart-context

# Include Git metadata in templates
# Available variables: git_branch, git_commit, git_status, etc.
```

### Session Management

```bash
# Save to named session
termai preset use "Debugging Assistant" --session bug-hunt-123

# Continue session later
termai chat --session bug-hunt-123
```

### Variable Override

```bash
# Override specific variables
termai preset use "Documentation Generator" \
  --variables "doc_type=API" \
  --variables "audience=developers"

# Use all defaults
termai preset use "Test Generator" --use-defaults
```

## 🎨 Best Practices

### Template Design

1. **Clear Instructions** - Be specific about what you want
2. **Conditional Logic** - Use `{{#if}}` for optional sections
3. **Fallback Content** - Always provide `{{else}}` alternatives
4. **Variable Names** - Use descriptive, consistent names
5. **Context Awareness** - Design for `file_content` variable

### Variable Configuration

1. **Required vs Optional** - Mark variables appropriately
2. **Good Defaults** - Provide sensible default values
3. **Clear Descriptions** - Help users understand each variable
4. **Appropriate Types** - Choose the right variable type

### Organization

1. **Meaningful Names** - Use descriptive preset names
2. **Proper Categories** - Organize by purpose
3. **Version Control** - Export/import for team sharing
4. **Documentation** - Include usage examples

## 📁 Preset Management

### Listing and Discovery

```bash
# List all presets
termai preset list

# Detailed view with descriptions
termai preset list --detailed

# Filter by category
termai preset list --category development

# Search presets
termai preset search "test"
termai preset search "review" --category development
```

### Import/Export

```bash
# Export for sharing
termai preset export "My Custom Preset" --file team-preset.yaml

# Import team presets
termai preset import team-preset.yaml

# View preset details
termai preset show "Code Review Assistant"
termai preset show "My Preset" --template
```

### File Locations

- **User Presets**: `~/.config/termai/presets/`  
- **Built-in Presets**: Embedded in application
- **Export Format**: YAML files with full configuration

## 💡 Tips & Tricks

### Context-Aware Chat Suggestions

Start interactive chat with context for preset suggestions:

```bash
# Chat mode suggests relevant presets based on files
termai chat src/components/

# Example output:
# 💡 Preset Suggestions
# Based on your context, these presets might be helpful:
#   📦 Code Review Assistant - Perfect for reviewing code changes
#   📦 Test Generator - Generate tests for your code
#   📦 Documentation Generator - Generate documentation for your code
```

### Template Testing

```bash
# Preview without execution
termai preset use "My Preset" --preview

# Test with specific context
termai preset use "My Preset" --preview src/test-file.js

# Validate template syntax during creation
# The wizard automatically validates Handlebars syntax
```

### Workflow Automation

```bash
# Code review workflow
git add .
termai preset use "Code Review Assistant" --git-staged --session review

# Documentation workflow  
termai preset use "Documentation Generator" --smart-context src/
termai preset use "Documentation Generator" README.md --variables "doc_type=README"

# Testing workflow
termai preset use "Test Generator" --smart-context tests/
termai preset use "Test Generator" --git-staged --variables "test_type=unit"
```

### Variable Best Practices

```yaml
# Good variable definition
variables:
  output_format:
    type: string
    required: false
    default: "markdown"
    description: "Output format (markdown, html, json)"
  
  include_examples:
    type: boolean  
    required: false
    default: true
    description: "Include code examples in output"
  
  max_suggestions:
    type: number
    required: false
    default: 5
    description: "Maximum number of suggestions to provide"
```

## 🔍 Troubleshooting

### Common Issues

**Template Validation Errors**
```bash
❌ Template validation failed: Unknown helper: myhelper
```
- Use standard Handlebars helpers only
- Check variable names match definitions
- Validate syntax: `{{variable}}` not `{variable}`

**Variable Type Mismatches**  
```bash
❌ Variable 'count' has incorrect type. Expected Number, got String
```
- Ensure variable values match their defined types
- Use appropriate input methods for each type

**File Access Issues**
```bash
⚠️ Failed to read file: src/nonexistent.js
```
- Check file paths are correct and accessible
- Use relative paths for portability

### Getting Help

```bash
# View preset details
termai preset show "Preset Name" --template

# List all variables
termai preset show "Preset Name" 

# Test with preview
termai preset use "Preset Name" --preview

# Check syntax
# Template validation happens automatically during creation/editing
```

## 📚 Examples

See the `examples/presets/` directory for more preset examples:

- **Code Quality Checker** - Comprehensive code analysis
- **API Documentation Generator** - REST API documentation  
- **Security Review Assistant** - Security-focused code review
- **Performance Analyzer** - Performance optimization suggestions
- **Migration Assistant** - Help with code migrations

## 🤝 Sharing Presets

### Team Workflow

1. **Create** preset with your team's standards
2. **Test** thoroughly with various inputs  
3. **Export** to team repository
4. **Share** YAML files via version control
5. **Import** on team member machines
6. **Standardize** on naming conventions

### Community Presets

Share useful presets with the community:
- Export well-documented presets
- Include usage examples  
- Follow naming conventions
- Test across different projects

---

## 🎉 Conclusion

The TermAI Preset System transforms ad-hoc AI interactions into reproducible, high-quality workflows. Whether you're doing code reviews, generating documentation, or debugging issues, presets ensure consistent, professional results every time.

**Next Steps:**
1. Try the built-in presets with `--smart-context`
2. Create your first custom preset
3. Integrate presets into your daily workflow
4. Share useful presets with your team

Happy prompting! 🚀