# TermAI Preset System - Quick Reference

## 📋 Commands

### Basic Operations
```bash
termai preset list                    # List all presets
termai preset list --detailed         # Detailed view
termai preset show "Name"             # Show preset info
termai preset show "Name" --template  # Show template content
termai preset search "query"          # Search presets
```

### Using Presets
```bash
termai preset use "Name"                           # Interactive mode
termai preset use "Name" --use-defaults            # Use all defaults  
termai preset use "Name" --preview                 # Preview before execution
termai preset use "Name" --variables "key=value"   # Override variables
```

### Context Integration
```bash
termai preset use "Name" --smart-context           # Auto-select relevant files
termai preset use "Name" --git-staged              # Use staged Git files
termai preset use "Name" --session "name"          # Save to session
termai preset use "Name" src/                      # Specific directory
```

### Creation & Editing
```bash
termai preset create "Name"                        # Interactive creation wizard
termai preset create "Name" --edit                 # Use external editor
termai preset edit "Name"                          # Full editor
termai preset edit "Name" --template               # Edit template only
termai preset edit "Name" --metadata               # Edit metadata only
```

### Import/Export
```bash
termai preset export "Name" --file preset.yaml     # Export preset
termai preset import preset.yaml                   # Import preset
termai preset delete "Name"                        # Delete preset (with confirmation)
```

## 🏷️ Built-in Presets

| Preset | Use Case | Best With |
|--------|----------|-----------|
| **Code Review Assistant** | Code analysis & review | `--git-staged`, `--smart-context` |
| **Documentation Generator** | Create docs & comments | `--smart-context`, specific files |
| **Test Generator** | Unit & integration tests | `--smart-context`, code files |
| **Debugging Assistant** | Error analysis & solutions | error logs, stack traces |
| **Refactoring Assistant** | Code improvements | `--smart-context`, legacy code |

## 🔧 Template Syntax

### Variables
```handlebars
{{variable_name}}                   # Simple substitution
{{#if boolean_var}}...{{/if}}      # Conditional blocks
{{#unless var}}...{{/unless}}      # Negative conditionals  
{{#if var}}...{{else}}...{{/if}}   # If-else blocks
```

### Special Variables
```handlebars
{{file_content}}        # Automatically populated with file content
{{git_branch}}          # Current Git branch (with --git-staged)
{{git_commit}}          # Current Git commit (with --git-staged)  
{{git_status}}          # Repository status (with --git-staged)
```

## 📊 Variable Types

| Type | Input | Description | Example |
|------|-------|-------------|---------|
| `String` | Text input | General text | Names, descriptions |
| `Boolean` | true/false | Feature flags | Include examples? |
| `Number` | Numeric | Counts, limits | Max suggestions: 5 |
| `File` | File path | Read file content | Config file path |
| `Directory` | Directory path | List directory | Project structure |
| `DateTime` | Format string | Current timestamp | Log timestamps |
| `GitInfo` | Info type | Git repository data | Branch, commit info |
| `Environment` | Variable name | Environment variable | API keys, paths |

## 🎯 Usage Patterns

### Code Review Workflow
```bash
git add .
termai preset use "Code Review Assistant" --git-staged --session review
```

### Documentation Generation  
```bash
termai preset use "Documentation Generator" --smart-context src/api/
termai preset use "Documentation Generator" README.md --variables "doc_type=README"
```

### Testing Workflow
```bash
termai preset use "Test Generator" --smart-context --preview
termai preset use "Test Generator" src/components/ --variables "test_type=unit"
```

### Debug Analysis
```bash
termai preset use "Debugging Assistant" error.log --variables "issue_description=Memory leak in production"
```

## 🔍 Flags & Options

### Context Flags
- `--smart-context` - Auto-select relevant files using AI
- `--git-staged` - Include Git staged files
- `--session "name"` - Save to named session  
- `--directory "path"` - Include directory content
- `--directories "p1" "p2"` - Multiple directories

### Execution Flags  
- `--preview` - Preview template before execution
- `--use-defaults` - Use default values for all variables
- `--variables "key=value"` - Override specific variables

### Display Flags
- `--detailed` - Show detailed information
- `--template` - Show template content
- `--category "name"` - Filter by category

## 📁 File Locations

```bash
~/.config/termai/presets/          # User presets directory
~/.config/termai/presets/*.yaml    # Individual preset files
```

## 💡 Quick Tips

### Auto-Complete
```bash
termai preset <TAB>                # Available commands
termai preset use <TAB>            # Available presets
```

### Chat Integration
```bash
termai chat src/                   # Shows preset suggestions
# 💡 Preset Suggestions based on context
```

### Template Testing
```bash
termai preset use "Name" --preview           # Test without execution  
termai preset create "Test" --template "{{var}}"   # Quick template test
```

### Variable Shortcuts
```bash
# Multiple variables at once
termai preset use "Name" \
  --variables "type=API" \
  --variables "format=markdown" \
  --variables "examples=true"
```

## 🚨 Troubleshooting

### Common Errors
```bash
❌ Template validation failed        # Check Handlebars syntax
❌ Required variable not provided     # Add missing variables  
❌ Preset 'Name' not found           # Check spelling, use preset list
⚠️ No staged files found             # Use git add first
```

### Quick Fixes
```bash
termai preset list                   # Check available presets
termai preset show "Name"            # Check required variables
git add .                           # Stage files for --git-staged
termai preset use "Name" --preview   # Test before execution
```

## 🎨 Example Templates

### Simple Template
```handlebars
Please analyze the following {{content_type}}:

{{#if file_content}}
{{file_content}}
{{else}}
Please provide the content to analyze.
{{/if}}

Focus on {{analysis_type}} aspects.
```

### Advanced Template with Git Context
```handlebars
Code Review for {{git_branch}} branch ({{git_commit}})

{{#if git_status}}
Repository Status: {{git_status}}
{{/if}}

Please review for:
{{#each review_aspects}}
- {{this}}
{{/each}}

{{file_content}}
```

---

**💡 Pro Tip:** Use `termai preset list --detailed` to see all preset details, then `termai preset use "Name" --preview` to test before execution!