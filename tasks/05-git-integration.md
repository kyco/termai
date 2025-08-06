# Task: Git Integration

## Overview
Seamlessly integrate TermAI with Git workflows to provide intelligent commit message generation, code review assistance, and diff analysis.

## Success Criteria
- [ ] 50% of git commits use AI-generated messages
- [ ] Code review quality improves with AI suggestions
- [ ] Developers spend less time on routine Git tasks
- [ ] Integration feels natural and non-intrusive
- [ ] Git integration prominently featured in README.md as a developer workflow enhancement

## Implementation Tasks

### 1. Git Repository Detection
- [ ] Create `GitRepository` struct for repository operations
- [ ] Implement repository root detection from any subdirectory
- [ ] Add `.git` directory validation and health checks
- [ ] Support Git worktrees and submodules
- [ ] Handle bare repositories appropriately

### 2. Git Status and Diff Analysis
- [ ] Implement Git status parsing and analysis
- [ ] Add staged/unstaged change detection
- [ ] Create diff parsing for meaningful change analysis
- [ ] Implement file change categorization (added/modified/deleted)
- [ ] Add binary file detection and handling
- [ ] Support merge conflict detection

### 3. Commit Message Generation
- [ ] Create `CommitMessageGenerator` for AI-powered messages
- [ ] Implement `termai commit` subcommand
- [ ] Add diff analysis for commit context
- [ ] Generate conventional commit format messages
- [ ] Support custom commit message templates
- [ ] Add interactive editing and approval workflow
- [ ] Implement message regeneration options

### 4. Code Review Assistant
- [ ] Create `termai review` subcommand for code review
- [ ] Analyze staged changes for potential issues
- [ ] Generate review comments and suggestions
- [ ] Check for common code quality issues:
  - [ ] Security vulnerabilities
  - [ ] Performance concerns
  - [ ] Code style violations
  - [ ] Missing documentation
  - [ ] Error handling gaps
- [ ] Support different review depths (quick/thorough)

### 5. Branch and History Analysis
- [ ] Implement branch comparison and analysis
- [ ] Add `termai branch-summary` for branch overview
- [ ] Generate release notes from commit history
- [ ] Analyze commit patterns and suggest improvements
- [ ] Support Git blame integration for context

### 6. Git Hook Integration
- [ ] Create pre-commit hook for automatic suggestions
- [ ] Implement commit-msg hook for message validation
- [ ] Add pre-push hook for final code review
- [ ] Support custom hook configurations
- [ ] Handle hook installation and management

### 7. Interactive Git Workflows
- [ ] Add interactive rebase assistance
- [ ] Implement conflict resolution suggestions
- [ ] Create branch naming suggestions
- [ ] Add PR/MR description generation
- [ ] Support Git flow workflow integration

### 8. Git Configuration Integration
- [ ] Read Git user configuration for personalization
- [ ] Support Git aliases and custom commands
- [ ] Integrate with existing Git hooks
- [ ] Respect .gitignore patterns
- [ ] Handle different Git configurations

### 9. Advanced Git Features
- [ ] Support Git worktree operations
- [ ] Add stash management assistance
- [ ] Implement tag and release management
- [ ] Support Git LFS file handling
- [ ] Add remote repository analysis

### 10. Error Handling and Edge Cases
- [ ] Handle repositories without commits gracefully
- [ ] Deal with large diffs efficiently
- [ ] Support repositories with unusual configurations
- [ ] Handle network issues for remote operations
- [ ] Provide fallbacks when Git commands fail

### 11. Testing
- [ ] Unit tests for Git parsing and analysis
- [ ] Integration tests with real Git repositories
- [ ] Test with various Git configurations
- [ ] Performance tests with large repositories
- [ ] Test hook integration scenarios

### 12. Documentation
- [ ] Create Git integration user guide
- [ ] Document all Git-related commands
- [ ] Add troubleshooting guide for Git issues
- [ ] Create examples for different workflows
- [ ] Document hook installation process
- [ ] Feature Git integration prominently in README with workflow examples
- [ ] Add Git workflow demo GIFs to README

## File Changes Required

### New Files
- `src/git/mod.rs` - Git integration module
- `src/git/repository.rs` - Git repository operations
- `src/git/diff.rs` - Diff parsing and analysis
- `src/git/commit.rs` - Commit message generation
- `src/git/review.rs` - Code review functionality
- `src/git/hooks.rs` - Git hook management

### Modified Files
- `src/main.rs` - Add Git subcommands
- `src/args.rs` - Add Git command arguments
- `Cargo.toml` - Add Git dependencies

## Dependencies to Add
```toml
[dependencies]
git2 = "0.18"          # Git operations
regex = "1.10"         # Pattern matching for diffs
tempfile = "3.8"       # Temporary file handling
```

## Command Examples

### Commit Message Generation
```bash
# Analyze staged changes and generate commit message
termai commit
> 📝 Analyzing staged changes...
> 
> Suggested commit message:
> feat(auth): add OAuth2 integration with token refresh
> 
> - Add OAuth2Provider trait implementation
> - Implement token refresh mechanism
> - Add comprehensive error handling
> - Update tests for new auth flow
> 
> [e]dit, [a]ccept, [r]egenerate, [c]ancel?

# Quick commit with auto-generated message
termai commit --auto
```

### Code Review
```bash
# Review staged changes
termai review
> 🔍 Reviewing staged changes...
> 
> ⚠️  Found potential issues:
> 
> src/auth.rs:42
> - Consider using `SecretString` instead of `String` for passwords
> - Missing error handling for network requests
> 
> src/auth.rs:58  
> - Function is getting complex (15+ lines), consider refactoring
> - Add documentation for public function
> 
> ✅ Positive findings:
> - Good test coverage for new functionality
> - Follows consistent error handling patterns
```

### Branch Analysis
```bash
# Get branch summary
termai branch-summary feature/oauth
> 📊 Branch: feature/oauth (5 commits ahead of main)
> 
> Summary of changes:
> - 3 new files added (auth module)
> - 147 lines added, 23 deleted
> - 5 test files updated
> - No breaking changes detected
> 
> Suggested PR description:
> ## OAuth2 Integration
> This PR adds OAuth2 authentication support...
```

## Success Metrics
- Commit message generation usage: >50% of commits
- Code review adoption: >30% of staged changes reviewed
- Time saved on Git tasks: >20% reduction
- User satisfaction with Git integration: >4.0/5
- Reduction in commit message rewrites: >40%

## Risk Mitigation
- **Risk**: Performance issues with large repositories
  - **Mitigation**: Implement diff size limits and incremental analysis
- **Risk**: Git hook conflicts with existing tools
  - **Mitigation**: Detect existing hooks and provide integration options
- **Risk**: Complex merge scenarios causing confusion
  - **Mitigation**: Focus on common cases first, add advanced features gradually**Note**: Backwards compatibility is explicitly not a concern for this implementation.
