# TermAI Git Integration

This document provides comprehensive documentation for TermAI's Git integration features, including interactive rebase assistance, conflict resolution, tag management, and branch analysis.

## Table of Contents

- [Overview](#overview)
- [Tag and Release Management](#tag-and-release-management)
- [Branch Analysis and Naming](#branch-analysis-and-naming)
- [Interactive Rebase Assistant](#interactive-rebase-assistant)
- [Conflict Resolution](#conflict-resolution)
- [Integration Testing](#integration-testing)
- [Architecture](#architecture)
- [Configuration](#configuration)

## Overview

TermAI integrates seamlessly with Git repositories to provide AI-powered assistance for common Git workflows. All Git commands work within any Git repository and provide intelligent analysis, suggestions, and automation.

### Key Features

- **🏷️ Tag Management** - AI-powered semantic version suggestions and release notes
- **🌿 Branch Analysis** - Repository-aware branch naming and analysis
- **🔄 Interactive Rebase** - AI-guided commit organization and squashing
- **⚔️ Conflict Resolution** - Intelligent merge conflict analysis and strategies
- **🤖 AI Integration** - Context-aware suggestions using Claude/OpenAI

## Tag and Release Management

### Commands

#### `termai tag list`
Lists all Git tags with AI-powered analysis and recommendations.

```bash
# List all tags
termai tag list

# Example output:
# 🏷️  TermAI Git Tag & Release Management
# ═══════════════════════════════════════
# 
# 📋 Git Tags:
#   • v1.2.0 (annotated) - Latest Release
#   • v1.1.0 (lightweight) - Previous Version
#   • v1.0.0 (annotated) - Initial Release
# 
# 🤖 AI Release Analysis:
#   • Release cadence: Regular monthly releases
#   • Version strategy: Semantic versioning (recommended)
#   • Next suggested version: v1.3.0
```

#### `termai tag suggest`
Get AI-powered suggestions for the next version tag based on commit analysis.

```bash
# Get version suggestions
termai tag suggest

# Example output:
# 🎯 AI Tag Suggestion
# ═══════════════════
# 
# 🔍 Analyzing recent changes since v1.2.0:
#   • 15 commits analyzed
#   • 3 features added
#   • 2 bug fixes
#   • 1 breaking change detected
# 
# 📊 Change Analysis:
#   • Breaking changes: API signature changes
#   • Features: OAuth integration, user management
#   • Fixes: Memory leaks, parsing errors
# 
# 🎯 AI Recommendation:
#   • Suggested version: v2.0.0 (major)
#   • Reasoning: Breaking changes require major version bump
#   • Alternative: v1.3.0 if breaking changes can be avoided
```

#### `termai tag create <version>`
Create a new tag with AI-generated release notes.

```bash
# Create annotated tag with AI-generated message
termai tag create v1.3.0

# Create lightweight tag
termai tag create v1.3.0 --lightweight

# Force create (overwrite existing)
termai tag create v1.3.0 --force
```

#### `termai tag show <version>`
Show detailed information about a specific tag.

```bash
# Show tag details
termai tag show v1.2.0

# Example output:
# 📋 Tag Details: v1.2.0
# ═══════════════════════
# 
# 🏷️  Tag Information:
#   • Type: Annotated tag
#   • Created: 2024-01-15 14:30:00
#   • Tagger: Developer Name <dev@example.com>
# 
# 📝 Tag Message:
#   Version 1.2.0 Release
#   
#   Features:
#   - Added user authentication
#   - Improved error handling
#   
#   Bug Fixes:
#   - Fixed memory leak in parser
#   - Resolved UI rendering issues
```

#### `termai tag release-notes`
Generate comprehensive release notes between versions.

```bash
# Generate release notes between tags
termai tag release-notes --from-tag v1.1.0 --to-tag v1.2.0

# Generate for different formats
termai tag release-notes --from-tag v1.1.0 --format json
termai tag release-notes --from-tag v1.1.0 --format text
```

### Tag Management Best Practices

1. **Use Semantic Versioning**: Follow semver (major.minor.patch) for predictable releases
2. **Annotated Tags**: Use annotated tags for releases to include metadata
3. **Consistent Messaging**: Let AI generate consistent release notes and tag messages
4. **Regular Analysis**: Run `tag suggest` before each release for version guidance

## Branch Analysis and Naming

### Commands

#### `termai branch-summary`
Analyze the current branch with AI-powered insights.

```bash
# Analyze current branch
termai branch-summary

# Analyze specific branch
termai branch-summary feature/oauth-integration

# Generate PR description format
termai branch-summary --release-notes

# Example output:
# 🔍 Analyzing Git repository and branch...
# 
# 📊 Branch Analysis:
# ═══════════════════
# 
# ℹ️  Branch Information:
#   • Current branch: feature/oauth-integration
#   • Base branch: main
#   • Commits ahead: 12
#   • Files changed: 8
# 
# 🔄 Branch Comparison:
#   • Added: 245 lines
#   • Removed: 67 lines
#   • Modified files: src/auth/, config/
# 
# 📝 Change Summary:
#   • Primary focus: Authentication system
#   • Risk level: Medium (breaking changes)
#   • Test coverage: 85% (good)
# 
# 🤖 AI Insights:
#   • Well-structured feature branch
#   • Consider squashing setup commits
#   • Documentation needs updating
```

#### `termai branch-summary --suggest-name`
Get AI-powered branch name suggestions based on context.

```bash
# Get branch name suggestions with context
termai branch-summary --suggest-name --context "OAuth integration"

# Example output:
# 🌿 AI Branch Naming Assistant
# ══════════════════════════════
# 
# 🔍 Analyzing repository context...
# 
# 📊 Context Analysis:
#   • Repository type: Rust Project
#   • Main framework: Tokio-based CLI
#   • Recent patterns: feature/, fix/, perf/
# 
# 💡 AI Branch Name Suggestions:
#   🥇 feature/oauth-integration (recommended)
#   🥈 auth/oauth-implementation  
#   🥉 feature/user-authentication
#   
# 📋 File-based Analysis:
#   • Detected auth-related files
#   • OAuth configuration patterns
#   • Integration test structure
```

### Branch Naming Conventions

TermAI suggests branch names following these patterns:
- `feature/` - New features
- `fix/` - Bug fixes  
- `perf/` - Performance improvements
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Test improvements

## Interactive Rebase Assistant

### Commands

#### `termai rebase status`
Check the current rebase status and get recommendations.

```bash
# Check rebase status
termai rebase status

# Example output:
# 🔄 TermAI Interactive Rebase Assistant
# ════════════════════════════════════════
# 
# 📊 Rebase Status:
# ═════════════════
# 
# ℹ️ No rebase in progress
# 
# 📈 Repository State:
#   • Current branch: feature/new-auth
#   • Base branch: main
#   • Commits ahead: 8
#   • Last rebase: 2 days ago
```

#### `termai rebase plan`
Generate an AI-powered rebase plan for commit optimization.

```bash
# Generate rebase plan for last 5 commits
termai rebase plan --count 5

# Target specific branch
termai rebase plan --target main

# Interactive planning with AI suggestions
termai rebase plan --interactive

# Example output:
# 📋 Rebase Plan Generation
# ═══════════════════════════
# 
# 🎯 Rebase Target Analysis:
#   • Analyzing last 5 commits
#   • Target branch: main
#   • Conflict potential: Low
# 
# 🤖 AI Rebase Recommendations:
# ═════════════════════════════
# 
#   1. pick a1b2c3d feat: add authentication core
#   2. squash d4e5f6g fix: typo in auth module
#   3. squash g7h8i9j docs: update auth examples  
#   4. pick j0k1l2m refactor: optimize auth flow
#   5. pick m3n4o5p test: add comprehensive auth tests
# 
# 💡 Optimization Benefits:
#   • Reduced commits: 5 → 3
#   • Cleaner history: Logical grouping
#   • Easier review: Related changes combined
# 
# 📋 Suggested Rebase Plan:
#   • Squash documentation and fix commits
#   • Keep feature and test commits separate
#   • Estimated time: 5 minutes
```

#### `termai rebase analyze`
Analyze commits for rebase optimization opportunities.

```bash
# Analyze commits for rebase patterns
termai rebase analyze

# Example output:
# 🔬 Commit Analysis for Rebase
# ═════════════════════════════
# 
# 📊 Commit Statistics:
#   • Total commits analyzed: 8
#   • Squash candidates: 3
#   • Fixup opportunities: 2
#   • Clean commits: 3
# 
# 📈 Commit Type Distribution:
#   • feat: 2 commits (25%)
#   • fix: 3 commits (37.5%)  
#   • docs: 1 commit (12.5%)
#   • test: 2 commits (25%)
# 
# 📝 Commit Details:
#   🟢 a1b2c3d feat: implement OAuth flow (keep)
#   🟡 d4e5f6g fix: handle edge cases (squash candidate)  
#   🔴 g7h8i9j fix: typo in comments (fixup)
#   🟢 j0k1l2m test: add OAuth tests (keep)
# 
# 💡 AI Recommendations:
#   • Group related fixes into main feature commits
#   • Separate test commits for better traceability
#   • Consider interactive rebase for cleanup
```

#### `termai rebase start`
Start an interactive rebase with AI guidance.

```bash
# Start interactive rebase with AI assistance
termai rebase start --interactive --count 5

# Use AI suggestions automatically
termai rebase start --ai-suggestions
```

#### `termai rebase continue`
Continue an interrupted rebase with AI assistance.

```bash
# Continue rebase with conflict resolution help
termai rebase continue

# Auto-resolve simple conflicts
termai rebase continue --auto-resolve
```

#### `termai rebase abort`
Safely abort the current rebase operation.

```bash
# Abort current rebase
termai rebase abort
```

### Rebase Best Practices

1. **Plan Before Rebasing**: Use `termai rebase plan` to preview changes
2. **Analyze Commits**: Run `termai rebase analyze` to identify optimization opportunities  
3. **Small Batches**: Rebase small sets of commits for easier management
4. **Test After Rebase**: Always test your code after completing a rebase

## Conflict Resolution

### Commands

#### `termai conflicts detect`
Detect and analyze all merge conflicts in the repository.

```bash
# Detect all conflicts
termai conflicts detect

# Example output:
# ⚔️ TermAI Conflict Resolution Assistant
# ═════════════════════════════════════
# 
# 🔍 Detecting Merge Conflicts...
# 
# ⚠️  2 conflicts detected in 2 file(s)
# 
#    📁 src/auth/oauth.rs
#       ⚔️ 2 conflict markers
#         • Line 45: HEAD vs feature/oauth-fix
#         • Line 67: HEAD vs feature/oauth-fix
# 
#    📁 config/auth.yaml  
#       ⚔️ 1 conflict markers
#         • Line 12: HEAD vs feature/oauth-fix
# 
# 🚀 Quick Resolution Options:
#    • termai conflicts analyze - Get AI-powered analysis
#    • termai conflicts suggest - Get resolution strategies
#    • termai conflicts resolve - Interactive resolution wizard
#    • git mergetool - Open merge tool
```

#### `termai conflicts analyze`
Get AI-powered analysis of conflict complexity and recommendations.

```bash
# Analyze all conflicts
termai conflicts analyze

# Analyze specific file
termai conflicts analyze --file src/auth/oauth.rs

# Detailed analysis
termai conflicts analyze --detailed

# Example output:
# 🤖 AI Conflict Analysis
# ═══════════════════════
# 
# 📊 Analysis: src/auth/oauth.rs
# ─────────────────────────────
#    Conflict type: Code conflict in Rust file
#    Complexity: Medium
#    Resolution confidence: 85%
# 
#    AI Recommendations:
#       1. Simple conflict - manual resolution recommended
#       2. Review both changes and combine if possible
# 
#    ✨ This conflict appears auto-resolvable
# 
# 📊 Analysis: config/auth.yaml
# ─────────────────────────────────
#    Conflict type: Configuration conflict
#    Complexity: Low  
#    Resolution confidence: 95%
# 
#    AI Recommendations:
#       1. Configuration values differ - choose based on environment
#       2. Consider environment-specific configs
# 
#    🔧 Manual resolution required
```

#### `termai conflicts suggest`
Get intelligent resolution strategies for conflicts.

```bash
# Get resolution strategies
termai conflicts suggest

# Interactive strategy selection
termai conflicts suggest --interactive

# Example output:
# 💡 Resolution Strategy Suggestions
# ═════════════════════════════════
# 
# 🎯 Overall Strategy:
#    Approach: File-by-file approach
#    Order: Start with simple conflicts, then tackle complex ones
#    Estimated time: 15 minutes
# 
# 📋 Strategy: src/auth/oauth.rs
#    Method: Multi-step resolution
#    Tools: rust-analyzer, VS Code, vim
# 
#    Steps:
#       1. Open file in preferred editor
#       2. Locate conflict markers
#       3. Analyze both versions of the code
#       4. Choose appropriate resolution strategy
#       5. Remove conflict markers
#       6. Test the changes
# 
#    Watch out for:
#       • Check syntax after resolution
#       • Run cargo check
```

#### `termai conflicts resolve`
Interactive conflict resolution wizard with AI assistance.

```bash
# Start interactive resolution wizard
termai conflicts resolve

# Auto-resolve simple conflicts
termai conflicts resolve --auto-resolve

# Use specific resolution strategy
termai conflicts resolve --strategy merge

# Example workflow:
# 🧙 Interactive Resolution Wizard
# ═════════════════════════════════
# 
# Found 2 conflicted files
# 
# 🔧 Resolving: src/auth/oauth.rs
# 
# 🔍 Conflict Preview:
#    ⚔️ Conflict 1 (Line 45)
#    Ours: const CLIENT_ID = "old_id";
#    Theirs: const CLIENT_ID = "new_client_id";
# 
# How would you like to resolve this conflict?
# > Accept ours (current branch)
#   Accept theirs (incoming changes) 
#   Manual merge with editor
#   AI-suggested merge
#   Skip this file for now
```

#### `termai conflicts status`
Check the current status of conflict resolution progress.

```bash
# Check resolution progress
termai conflicts status

# Example output:
# 📊 Conflict Status
# ═══════════════════
# 
# 🔄 Current Merge Operation:
#    Merging: feature/oauth → main
# 
# 📈 Progress:
#    Resolved: 1/3 files (33%)
# 
# ⚠️  Remaining Conflicts:
#    • src/auth/oauth.rs
#    • config/auth.yaml
# 
# 💡 Next Steps:
#    • termai conflicts detect - Detect and analyze conflicts
#    • termai conflicts suggest - Get resolution suggestions
#    • termai conflicts resolve - Interactive resolution wizard
```

#### `termai conflicts guide`
Show comprehensive conflict resolution guide and best practices.

```bash
# Show resolution guide
termai conflicts guide

# Example output:
# 📚 Conflict Resolution Guide
# ═════════════════════════════
# 
# 🔍 Understanding Conflict Markers:
#    <<<<<<< HEAD - Marks the start of your changes
#    ======= - Separates your changes from theirs
#    >>>>>>> branch-name - Marks the end of their changes
# 
# 🛠️  Resolution Strategies:
# 
#    Accept Ours (Keep Current)
#       • When your changes are correct
#       • Use: git checkout --ours <file>
# 
#    Accept Theirs (Take Incoming)
#       • When incoming changes are better
#       • Use: git checkout --theirs <file>
# 
#    Manual Merge
#       • When both changes are needed
#       • Edit file to combine changes
#       • Remove conflict markers
# 
#    AI-Assisted Resolution
#       • Get intelligent merge suggestions
#       • Use: termai conflicts suggest
# 
# 🔧 Recommended Tools:
#    • git mergetool - Built-in merge tool
#    • code --merge - VS Code with GitLens
#    • vim -d - Vim with fugitive
#    • External tools - Beyond Compare, P4Merge
```

### Conflict Resolution Best Practices

1. **Analyze First**: Use `termai conflicts analyze` before resolving
2. **Understand Context**: Review the changes that caused conflicts
3. **Test Thoroughly**: Always test resolved conflicts
4. **Document Decisions**: Add comments explaining complex resolutions
5. **Use Tools**: Leverage merge tools for complex conflicts

## Integration Testing

TermAI includes comprehensive test suites for Git integration:

### Test Categories

1. **Unit Tests** (`src/git/`)
   - Repository detection and initialization
   - Diff analysis and language detection
   - Git status parsing and file tracking

2. **Simple Integration Tests** (`tests/simple_git_tests.rs`)
   - Command functionality validation
   - Error handling outside Git repositories
   - Help system integration
   - Performance testing

3. **Comprehensive Integration Tests** (`tests/git_integration_tests.rs`)
   - Full workflow testing with real repositories
   - Repository type detection (Rust, Node.js, Python)
   - AI suggestion quality validation
   - Command interaction testing

### Running Tests

```bash
# Run all Git tests
cargo test git

# Run integration tests only
cargo test --test git_integration_tests
cargo test --test simple_git_tests

# Run unit tests only  
cargo test git::
```

### Test Structure

```
tests/
├── git_integration_tests.rs      # Comprehensive workflow tests
└── simple_git_tests.rs          # Basic functionality tests

src/git/
├── repository.rs                 # Repository operations (tested)
├── diff.rs                      # Diff analysis (tested)  
└── mod.rs                       # Module integration
```

## Architecture

### Module Structure

```
src/
├── commands/
│   ├── tag.rs                   # Tag management commands
│   ├── branch_summary.rs        # Branch analysis commands
│   ├── rebase.rs                # Interactive rebase assistance
│   └── conflicts.rs             # Conflict resolution assistance
├── git/
│   ├── repository.rs            # Git repository abstraction
│   ├── diff.rs                  # Diff parsing and analysis
│   └── mod.rs                   # Git module exports
├── args/
│   ├── structs.rs              # Command argument definitions
│   └── validation.rs           # Input validation
└── discovery.rs                # Command discovery and help
```

### Key Components

#### GitRepository (`src/git/repository.rs`)
- Git repository detection and initialization
- Status parsing and file tracking  
- Branch and tag operations
- User configuration management

#### DiffAnalyzer (`src/git/diff.rs`)
- Git diff parsing and analysis
- Language detection for files
- Change summarization
- Breaking change detection

#### Command Handlers (`src/commands/`)
- Individual command implementations
- AI integration for suggestions
- Interactive workflows and user experience
- Error handling and troubleshooting guidance

## Configuration

### Environment Variables

```bash
# AI Provider Configuration
export TERMAI_PROVIDER=claude          # or openai
export TERMAI_CLAUDE_API_KEY=your_key
export TERMAI_OPENAI_API_KEY=your_key

# Git Integration Settings  
export TERMAI_GIT_PAGER=less          # Pager for Git output
export TERMAI_GIT_EDITOR=vim          # Editor for interactive operations
```

### Git Configuration

TermAI respects standard Git configuration:

```bash
# Git user configuration (required for operations)
git config user.name "Your Name"
git config user.email "your.email@example.com"

# Git editor for interactive operations
git config core.editor vim

# Git merge tool preferences
git config merge.tool vimdiff
```

### CLAUDE.md Integration

Add Git-specific instructions to your project's CLAUDE.md:

```markdown
## Git Workflow

When working with this repository:
- Use `termai tag suggest` before releases
- Run `termai rebase plan` before rebasing
- Use `termai conflicts analyze` for merge conflicts
- Follow semantic versioning for tags

## Branch Naming

Preferred patterns:
- `feature/` for new features
- `fix/` for bug fixes  
- `perf/` for performance improvements
```

## Troubleshooting

### Common Issues

#### "No Git repository found"
- Ensure you're running commands from within a Git repository
- Check that `.git` directory exists in current or parent directories

#### "Git tag management failed"
- Verify Git user configuration is set up
- Ensure you have write permissions to the repository
- Check for conflicting tag names

#### "Rebase command failed"  
- Verify repository is in a clean state
- Check for ongoing merge or rebase operations
- Use `git status` to diagnose repository state

#### "Conflict detection failed"
- Ensure you're in the middle of a merge operation
- Check that conflicts actually exist with `git status`
- Verify repository permissions

### Getting Help

1. **Command-specific help**: `termai COMMAND --help`
2. **Error troubleshooting**: Each command provides specific guidance
3. **Discovery system**: `termai --help` shows available commands
4. **Validation**: Commands validate inputs and provide corrections

### Performance Considerations

- Git operations are optimized for typical repository sizes
- Large repositories (>1000 files) may experience slower analysis
- AI operations require network connectivity
- Local Git operations are prioritized over remote calls

## Examples and Workflows

### Complete Release Workflow

```bash
# 1. Analyze current state
termai branch-summary

# 2. Clean up commits with rebase
termai rebase plan --count 10
termai rebase start --interactive

# 3. Create release tag
termai tag suggest
termai tag create v1.4.0

# 4. Generate release notes
termai tag release-notes --from-tag v1.3.0 --to-tag v1.4.0
```

### Feature Branch Workflow

```bash
# 1. Get branch name suggestion
termai branch-summary --suggest-name --context "user management system"

# 2. Create and work on branch
git checkout -b feature/user-management

# 3. Analyze progress periodically
termai branch-summary

# 4. Prepare for merge
termai rebase plan --target main
termai conflicts detect
```

### Conflict Resolution Workflow

```bash
# 1. Detect conflicts during merge
termai conflicts detect

# 2. Get AI analysis
termai conflicts analyze --detailed

# 3. Get resolution strategies  
termai conflicts suggest

# 4. Interactive resolution
termai conflicts resolve

# 5. Verify resolution
termai conflicts status
```

This comprehensive documentation covers all aspects of TermAI's Git integration. For additional examples and advanced usage, see the command-specific help documentation with `termai COMMAND --help`.