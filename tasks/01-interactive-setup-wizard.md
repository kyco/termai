# Task: Interactive Setup Wizard

## Overview
Replace complex flag-based configuration with a guided setup experience that validates API keys and provides contextual guidance.

## Success Criteria
- [x] Setup time reduced from 10+ minutes to <2 minutes
- [x] 100% of users complete setup successfully
- [x] API keys are validated during setup
- [x] Users understand provider differences
- [x] Interactive setup wizard prominently featured in README.md with examples

## Implementation Tasks

### 1. Command Structure Refactoring
- [x] Create new `SetupArgs` struct for setup command
- [x] Add `setup` subcommand to main CLI parser
- [x] Implement subcommand routing in main.rs
- [x] Add help text and examples for setup command

### 2. Interactive Setup Flow
- [x] Create `SetupWizard` struct in new `src/setup/` module
- [x] Implement step-by-step wizard interface using `dialoguer` or similar
- [x] Add provider selection with descriptions:
  - [x] Claude (Anthropic) - Best for analysis & coding
  - [x] OpenAI - Versatile general purpose  
  - [x] Both providers option
- [x] Implement secure API key input with masking
- [x] Add confirmation step showing selected configuration

### 3. API Key Validation
- [x] Create `ApiKeyValidator` trait
- [x] Implement Claude API key validation
  - [x] Test basic API call to verify key works
  - [x] Handle rate limiting and error responses
  - [x] Show clear error messages for invalid keys
- [x] Implement OpenAI API key validation
  - [x] Test basic API call to verify key works
  - [x] Handle rate limiting and error responses  
  - [x] Show clear error messages for invalid keys
- [x] Add network connectivity checks

### 4. Configuration Management
- [ ] Create `SetupConfig` struct for wizard state
- [x] Implement configuration file generation
- [ ] Add configuration validation on startup

**Note**: Backwards compatibility is not a concern - existing configurations will be replaced with the new format.

### 5. User Experience Enhancements
- [x] Add progress indicators (Step X of Y)
- [x] Implement colored output for better readability
- [x] Add setup completion celebration/confirmation
- [x] Create option to re-run setup wizard
- [x] Add `--reset` flag to clear existing configuration

### 6. Error Handling & Recovery
- [x] Handle network connectivity issues gracefully
- [x] Provide clear error messages for common problems
- [x] Allow users to retry failed steps
- [ ] Add option to skip validation for offline setup
- [ ] Log setup attempts for debugging

### 7. Testing
- [x] Unit tests for setup wizard logic
- [x] Integration tests for API key validation
- [x] Mock tests for network failures
- [ ] End-to-end tests for complete setup flow
- [ ] Test setup wizard on different terminal sizes

### 8. Documentation
- [x] Update README with new setup instructions and prominent feature showcase
- [ ] Add setup wizard demo GIF or video to README
- [ ] Create setup troubleshooting guide
- [ ] Add screenshots/recordings of setup process
- [ ] Document configuration file format
- [x] Update CLAUDE.md with setup commands
- [x] Feature interactive setup in README's "Quick Start" section

## File Changes Required

### New Files
- [x] `src/setup/mod.rs` - Setup wizard module
- [x] `src/setup/wizard.rs` - Interactive setup implementation
- [x] `src/setup/validator.rs` - API key validation
- [ ] `src/setup/config.rs` - Configuration management

### Modified Files
- [x] `src/main.rs` - Add setup subcommand routing
- [x] `src/args.rs` - Add SetupArgs struct
- [x] `Cargo.toml` - Add dependencies (dialoguer, indicatif)

## Dependencies to Add
```toml
[dependencies]
dialoguer = "0.11"      # Interactive prompts
indicatif = "0.17"      # Progress indicators  
console = "0.15"        # Terminal styling
```

## Success Metrics
- Setup completion rate: >95%
- Average setup time: <2 minutes
- User satisfaction score: >4.5/5
- Support tickets related to setup: <5% of total

## Risk Mitigation
- **Risk**: Network issues during validation
  - **Mitigation**: Offline mode with validation skip option
- **Risk**: Terminal compatibility issues
  - **Mitigation**: Test on major terminal emulators

**Note**: Backwards compatibility is explicitly not a concern for this implementation.