# Task: Smart Context Discovery

## Overview
Automatically detect and suggest relevant project context, eliminating manual file specification and providing intelligent token management.

## Success Criteria
- [ ] 90% accuracy in relevant file detection
- [ ] Users rarely need to manually specify files
- [ ] Context size stays within token limits automatically
- [ ] Project type detection works for major languages/frameworks
- [ ] Smart context discovery featured in README.md as a key productivity enhancement

## Implementation Tasks

### 1. Project Detection System
- [ ] Create `ProjectDetector` trait for different project types
- [ ] Implement `RustProjectDetector` 
  - [ ] Detect Cargo.toml and workspace structure
  - [ ] Identify src/, tests/, examples/ directories
  - [ ] Parse Cargo.toml for dependencies and metadata
- [ ] Implement `JavaScriptProjectDetector`
  - [ ] Detect package.json and workspace structure
  - [ ] Identify src/, test/, dist/ directories
  - [ ] Parse package.json for dependencies
- [ ] Implement `PythonProjectDetector`
  - [ ] Detect pyproject.toml, setup.py, requirements.txt
  - [ ] Identify standard Python project structure
- [ ] Implement `GitProjectDetector`
  - [ ] Detect .git directory and common patterns
  - [ ] Identify recently modified files
  - [ ] Parse .gitignore for exclusion patterns

### 2. File Analysis and Ranking
- [ ] Create `FileAnalyzer` for content relevance scoring
- [ ] Implement file importance scoring algorithm:
  - [ ] Recently modified files get higher priority
  - [ ] Main entry points (main.rs, index.js) get high priority
  - [ ] Configuration files get medium priority
  - [ ] Test files included based on context
- [ ] Add file type analysis and filtering
- [ ] Implement content-based relevance scoring
- [ ] Add dependency relationship analysis

### 3. Token Management System
- [ ] Create `TokenOptimizer` for context size management
- [ ] Implement token counting for different models
- [ ] Add intelligent file truncation strategies:
  - [ ] Prioritize function signatures over implementations
  - [ ] Keep imports and type definitions
  - [ ] Truncate large data files and logs
- [ ] Implement context summarization for large files
- [ ] Add token budget allocation across files

### 4. Context Selection Algorithm  
- [ ] Implement smart context selection based on query
- [ ] Add keyword-based file relevance scoring
- [ ] Implement context caching for repeated queries
- [ ] Add context diff detection for incremental updates
- [ ] Support manual context overrides and additions

### 5. Configuration and Customization
- [ ] Create `.termai.toml` configuration file support
- [ ] Add include/exclude patterns configuration
- [ ] Implement project-specific context rules
- [ ] Add file type preferences and priorities
- [ ] Support custom context templates

### 6. User Interface for Context Discovery
- [ ] Add `--smart-context` flag for automatic detection
- [ ] Show context discovery process with progress indicators
- [ ] Display selected files and token usage before processing
- [ ] Add interactive context refinement prompts
- [ ] Implement context preview and editing

### 7. Context Persistence and Caching
- [ ] Cache project structure analysis results
- [ ] Implement file change detection for cache invalidation
- [ ] Store user context preferences per project
- [ ] Add context sharing between team members
- [ ] Implement context versioning for reproducibility

### 8. Integration with Existing Systems
- [ ] Integrate with current path extraction in `src/path/`
- [ ] Update session management to store smart context info
- [ ] Modify LLM adapters to handle optimized context
- [ ] Add context info to conversation history

### 9. Advanced Context Intelligence
- [ ] Implement semantic code analysis for relevance
- [ ] Add cross-file dependency tracking
- [ ] Implement context recommendation engine
- [ ] Add learning from user context selections
- [ ] Support context templates for common workflows

### 10. Error Handling and Fallbacks
- [ ] Handle permission errors gracefully
- [ ] Provide fallbacks when auto-detection fails
- [ ] Add manual override options for edge cases
- [ ] Implement graceful degradation for unsupported projects
- [ ] Handle large repositories efficiently

### 11. Testing
- [ ] Unit tests for each project detector
- [ ] Integration tests with various project structures
- [ ] Performance tests with large codebases
- [ ] Accuracy tests for file relevance scoring
- [ ] Token counting accuracy tests

### 12. Documentation
- [ ] Document supported project types and patterns
- [ ] Create configuration guide for .termai.toml
- [ ] Add troubleshooting guide for context issues
- [ ] Document context optimization strategies
- [ ] Create examples for different project types
- [ ] Showcase smart context discovery in README with before/after examples
- [ ] Add context intelligence demo to README features section

## File Changes Required

### New Files
- `src/context/mod.rs` - Context discovery module
- `src/context/detector.rs` - Project detection implementations
- `src/context/analyzer.rs` - File analysis and ranking
- `src/context/optimizer.rs` - Token optimization
- `src/context/config.rs` - Configuration management

### Modified Files  
- `src/path/extract.rs` - Integrate with smart context
- `src/main.rs` - Add smart context options
- `src/args.rs` - Add context-related arguments
- `Cargo.toml` - Add parsing dependencies

## Dependencies to Add
```toml
[dependencies]
toml = "0.8"           # Configuration file parsing
walkdir = "2.4"        # Directory traversal
ignore = "0.4"         # Gitignore parsing
tiktoken-rs = "0.5"    # Token counting
tree-sitter = "0.20"  # Code parsing (optional)
```

## Configuration Example
```toml
# .termai.toml
[context]
max_tokens = 4000
include = ["src/**/*.rs", "tests/**/*.rs", "Cargo.toml", "README.md"]
exclude = ["target/**", "**/*.log", "**/node_modules/**"]
priority_patterns = ["main.rs", "lib.rs", "mod.rs"]

[project]
type = "rust"
entry_points = ["src/main.rs", "src/lib.rs"]
```

## Success Metrics
- File relevance accuracy: >90%
- Context size optimization: Stay within 80% of token limits
- Auto-detection success rate: >95% for supported project types
- User manual overrides: <20% of sessions
- Context loading time: <2 seconds for typical projects

## Risk Mitigation
- **Risk**: Large repositories causing performance issues
  - **Mitigation**: Implement file filtering and lazy loading
- **Risk**: Inaccurate project detection
  - **Mitigation**: Fallback to manual selection, user feedback loop
- **Risk**: Token counting inaccuracies
  - **Mitigation**: Conservative estimates, user-configurable limits

**Note**: Backwards compatibility is explicitly not a concern for this implementation.