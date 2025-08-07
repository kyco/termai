# TermAI Git Commands Quick Reference

A quick reference guide for all TermAI Git integration commands.

## Tag Management

| Command | Description | Example |
|---------|-------------|---------|
| `termai tag list` | List all tags with AI analysis | `termai tag list` |
| `termai tag suggest` | Get AI version suggestions | `termai tag suggest` |
| `termai tag create <version>` | Create tag with AI message | `termai tag create v1.2.0` |
| `termai tag show <version>` | Show tag details | `termai tag show v1.1.0` |
| `termai tag release-notes` | Generate release notes | `termai tag release-notes --from-tag v1.0.0` |

### Tag Command Options

- `--lightweight` - Create lightweight tag
- `--force` - Force overwrite existing tag
- `--message <msg>` - Custom tag message
- `--from-tag <tag>` - Start tag for release notes
- `--to-tag <tag>` - End tag for release notes  
- `--format <fmt>` - Output format (markdown, text, json)

## Branch Analysis

| Command | Description | Example |
|---------|-------------|---------|
| `termai branch-summary` | Analyze current/specified branch | `termai branch-summary` |
| `termai branch-summary <branch>` | Analyze specific branch | `termai branch-summary feature/auth` |
| `termai branch-summary --suggest-name` | Get branch name suggestions | `termai branch-summary --suggest-name --context "OAuth"` |

### Branch Command Options

- `--suggest-name` - Generate branch name suggestions
- `--context <text>` - Provide context for suggestions
- `--release-notes` - Format output for PR descriptions
- `--from-tag <tag>` - Compare from specific tag

## Interactive Rebase

| Command | Description | Example |
|---------|-------------|---------|
| `termai rebase status` | Check rebase status | `termai rebase status` |
| `termai rebase plan` | Generate AI rebase plan | `termai rebase plan --count 5` |
| `termai rebase analyze` | Analyze commits for optimization | `termai rebase analyze` |
| `termai rebase start` | Start interactive rebase | `termai rebase start --interactive` |
| `termai rebase continue` | Continue interrupted rebase | `termai rebase continue` |
| `termai rebase abort` | Abort current rebase | `termai rebase abort` |

### Rebase Command Options

- `--count <n>` - Number of commits to analyze
- `--target <branch>` - Target branch for rebase
- `--interactive` - Enable interactive mode
- `--ai-suggestions` - Use AI suggestions automatically
- `--autosquash` - Enable autosquash mode

## Conflict Resolution

| Command | Description | Example |
|---------|-------------|---------|
| `termai conflicts detect` | Find all merge conflicts | `termai conflicts detect` |
| `termai conflicts analyze` | AI analysis of conflicts | `termai conflicts analyze` |
| `termai conflicts suggest` | Get resolution strategies | `termai conflicts suggest` |
| `termai conflicts resolve` | Interactive resolution wizard | `termai conflicts resolve` |
| `termai conflicts status` | Check resolution progress | `termai conflicts status` |
| `termai conflicts guide` | Show resolution guide | `termai conflicts guide` |

### Conflicts Command Options

- `--file <path>` - Analyze specific file
- `--detailed` - Detailed analysis output
- `--auto-resolve` - Attempt automatic resolution
- `--strategy <name>` - Use specific resolution strategy
- `--interactive` - Enable interactive mode

## Common Workflows

### Release Preparation

```bash
# 1. Analyze current branch
termai branch-summary

# 2. Plan commit cleanup  
termai rebase plan --count 10

# 3. Get version suggestion
termai tag suggest

# 4. Create release tag
termai tag create v1.3.0

# 5. Generate release notes
termai tag release-notes --from-tag v1.2.0
```

### Feature Development

```bash
# 1. Get branch name suggestion
termai branch-summary --suggest-name --context "user authentication"

# 2. Create branch
git checkout -b feature/user-authentication

# 3. Develop and commit changes
# ... make changes ...

# 4. Analyze branch progress
termai branch-summary

# 5. Prepare for merge
termai rebase plan --target main
```

### Conflict Resolution

```bash
# 1. Start merge (conflicts occur)
git merge feature-branch

# 2. Detect and analyze conflicts
termai conflicts detect
termai conflicts analyze

# 3. Get resolution strategies
termai conflicts suggest

# 4. Resolve interactively
termai conflicts resolve

# 5. Complete merge
git commit
```

### Commit Cleanup

```bash
# 1. Analyze commits for optimization
termai rebase analyze

# 2. Generate rebase plan
termai rebase plan --count 8

# 3. Start interactive rebase
termai rebase start --interactive

# 4. Follow AI suggestions
# ... interactive rebase session ...

# 5. Verify result
termai branch-summary
```

## Error Handling

All TermAI Git commands provide helpful error messages and troubleshooting guidance:

- **Repository detection failures** - Guidance to ensure you're in a Git repo
- **Permission issues** - Instructions for Git configuration
- **Conflict resolution failures** - Step-by-step recovery instructions
- **Invalid inputs** - Suggestions for correct command usage

## Exit Codes

- `0` - Success
- `1` - General error (with troubleshooting guidance)
- `2` - Invalid arguments (with usage help)

## Integration with Git

TermAI works alongside standard Git commands:

- Respects Git configuration (user, editor, merge tools)
- Uses Git's native conflict markers and merge states
- Compatible with all Git workflows and branching strategies
- Enhances but doesn't replace core Git functionality

## Tips

1. **Use `--help`** - Every command has detailed help: `termai tag --help`
2. **Check status first** - Use status commands before operations
3. **Preview changes** - Use plan/analyze commands before making changes  
4. **Combine with Git** - Use alongside standard Git commands
5. **Read error messages** - They include specific troubleshooting steps

For comprehensive documentation, see [GIT_INTEGRATION.md](./GIT_INTEGRATION.md).