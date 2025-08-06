# Task: Interactive Setup Wizard

## Overview
Replace complex flag-based configuration with a guided setup experience that validates API keys and provides contextual guidance.

## Success Criteria
- [ ] Setup time reduced from 10+ minutes to <2 minutes
- [ ] 100% of users complete setup successfully
- [ ] API keys are validated during setup
- [ ] Users understand provider differences
- [ ] Interactive setup wizard prominently featured in README.md with examples

## Implementation Tasks

### 1. Command Structure Refactoring
- [ ] Create new `SetupArgs` struct for setup command
- [ ] Add `setup` subcommand to main CLI parser
- [ ] Implement subcommand routing in main.rs
- [ ] Add help text and examples for setup command

### 2. Interactive Setup Flow
- [ ] Create `SetupWizard` struct in new `src/setup/` module
- [ ] Implement step-by-step wizard interface using `dialoguer` or similar
- [ ] Add provider selection with descriptions:
  - [ ] Claude (Anthropic) - Best for analysis & coding
  - [ ] OpenAI - Versatile general purpose  
  - [ ] Both providers option
- [ ] Implement secure API key input with masking
- [ ] Add confirmation step showing selected configuration

### 3. API Key Validation
- [ ] Create `ApiKeyValidator` trait
- [ ] Implement Claude API key validation
  - [ ] Test basic API call to verify key works
  - [ ] Handle rate limiting and error responses
  - [ ] Show clear error messages for invalid keys
- [ ] Implement OpenAI API key validation
  - [ ] Test basic API call to verify key works
  - [ ] Handle rate limiting and error responses  
  - [ ] Show clear error messages for invalid keys
- [ ] Add network connectivity checks

### 4. Configuration Management
- [ ] Create `SetupConfig` struct for wizard state
- [ ] Implement configuration file generation
- [ ] Add configuration validation on startup

**Note**: Backwards compatibility is not a concern - existing configurations will be replaced with the new format.

### 5. User Experience Enhancements
- [ ] Add progress indicators (Step X of Y)
- [ ] Implement colored output for better readability
- [ ] Add setup completion celebration/confirmation
- [ ] Create option to re-run setup wizard
- [ ] Add `--reset` flag to clear existing configuration

### 6. Error Handling & Recovery
- [ ] Handle network connectivity issues gracefully
- [ ] Provide clear error messages for common problems
- [ ] Allow users to retry failed steps
- [ ] Add option to skip validation for offline setup
- [ ] Log setup attempts for debugging

### 7. Testing
- [ ] Unit tests for setup wizard logic
- [ ] Integration tests for API key validation
- [ ] Mock tests for network failures
- [ ] End-to-end tests for complete setup flow
- [ ] Test setup wizard on different terminal sizes

### 8. Documentation
- [ ] Update README with new setup instructions and prominent feature showcase
- [ ] Add setup wizard demo GIF or video to README
- [ ] Create setup troubleshooting guide
- [ ] Add screenshots/recordings of setup process
- [ ] Document configuration file format
- [ ] Update CLAUDE.md with setup commands
- [ ] Feature interactive setup in README's "Quick Start" section

## File Changes Required

### New Files
- `src/setup/mod.rs` - Setup wizard module
- `src/setup/wizard.rs` - Interactive setup implementation
- `src/setup/validator.rs` - API key validation
- `src/setup/config.rs` - Configuration management

### Modified Files
- `src/main.rs` - Add setup subcommand routing
- `src/args.rs` - Add SetupArgs struct
- `Cargo.toml` - Add dependencies (dialoguer, indicatif)

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