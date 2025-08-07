/// Built-in preset library for common development tasks
use crate::preset::manager::{Preset, PresetConfig};
use crate::preset::template::{Template, TemplateVariable};
use std::collections::HashMap;

/// Built-in preset library
pub struct BuiltinPresets;

impl BuiltinPresets {
    /// Get all built-in presets
    pub fn get_all() -> Vec<Preset> {
        vec![
            Self::code_review_preset(),
            Self::documentation_preset(),
            Self::testing_preset(),
            Self::debugging_preset(),
            Self::refactoring_preset(),
        ]
    }
    
    /// Get preset by name
    pub fn get_by_name(name: &str) -> Option<Preset> {
        Self::get_all()
            .into_iter()
            .find(|preset| preset.name == name)
    }
    
    /// Code Review preset
    pub fn code_review_preset() -> Preset {
        let mut variables = HashMap::new();
        
        variables.insert(
            "security".to_string(),
            TemplateVariable::boolean("Include security analysis".to_string(), true),
        );
        
        variables.insert(
            "performance".to_string(),
            TemplateVariable::boolean("Include performance analysis".to_string(), true),
        );
        
        variables.insert(
            "maintainability".to_string(),
            TemplateVariable::boolean("Include maintainability analysis".to_string(), true),
        );
        
        variables.insert(
            "context_description".to_string(),
            TemplateVariable::optional_string(
                "Additional context for the review".to_string(),
                "General code review".to_string(),
            ),
        );
        
        let template_content = r#"Please review the following code for:

{{#if security}}
🔒 **Security Issues:**
- Look for potential vulnerabilities
- Check input validation and sanitization
- Review authentication and authorization
- Identify injection attack possibilities
- Check for information disclosure
{{/if}}

{{#if performance}}
⚡ **Performance Concerns:**
- Identify bottlenecks and inefficiencies
- Suggest optimization opportunities
- Review algorithmic complexity
- Check for unnecessary operations
- Memory usage patterns
{{/if}}

{{#if maintainability}}
🔧 **Maintainability:**
- Code clarity and readability
- Documentation completeness
- Design pattern usage
- Code organization and structure
- Error handling practices
{{/if}}

**Context:** {{context_description}}

Please provide specific feedback with:
- Clear explanations of issues found
- Actionable recommendations for improvement
- Code examples where applicable
- Priority levels for different issues

{{#if file_content}}
Files to review:
{{file_content}}
{{else}}
Please provide the code you'd like me to review.
{{/if}}"#;
        
        let template = Template::new(
            "code-review".to_string(),
            "Comprehensive code review with security and performance focus".to_string(),
            template_content.to_string(),
            variables,
        ).unwrap();
        
        Preset {
            name: "Code Review Assistant".to_string(),
            description: "Comprehensive code review with security, performance, and maintainability analysis".to_string(),
            category: "development".to_string(),
            version: "1.0".to_string(),
            template,
            config: PresetConfig {
                provider: Some("claude".to_string()),
                max_tokens: Some(4000),
                temperature: Some(0.3),
                ..Default::default()
            },
        }
    }
    
    /// Documentation preset
    pub fn documentation_preset() -> Preset {
        let mut variables = HashMap::new();
        
        variables.insert(
            "doc_type".to_string(),
            TemplateVariable::optional_string(
                "Type of documentation (API, README, comments, etc.)".to_string(),
                "general".to_string(),
            ),
        );
        
        variables.insert(
            "audience".to_string(),
            TemplateVariable::optional_string(
                "Target audience (developers, users, maintainers)".to_string(),
                "developers".to_string(),
            ),
        );
        
        variables.insert(
            "include_examples".to_string(),
            TemplateVariable::boolean("Include code examples".to_string(), true),
        );
        
        let template_content = r#"Generate {{doc_type}} documentation for the following code.

**Target Audience:** {{audience}}
**Documentation Type:** {{doc_type}}

Requirements:
- Clear, concise explanations
- Professional tone appropriate for {{audience}}
{{#if include_examples}}
- Include practical code examples
- Show usage patterns and best practices
{{/if}}
- Cover key functionality and parameters
- Include any important caveats or limitations

{{#if (eq doc_type "API")}}
For each function/method, include:
- Purpose and functionality
- Parameters and their types
- Return values
- Usage examples
- Error conditions
{{/if}}

{{#if (eq doc_type "README")}}
Include sections for:
- Project overview and purpose
- Installation instructions
- Usage examples
- Configuration options
- Contributing guidelines
{{/if}}

{{#if file_content}}
Code to document:
{{file_content}}
{{else}}
Please provide the code you'd like me to document.
{{/if}}"#;
        
        let template = Template::new(
            "documentation".to_string(),
            "Generate comprehensive documentation for code".to_string(),
            template_content.to_string(),
            variables,
        ).unwrap();
        
        Preset {
            name: "Documentation Generator".to_string(),
            description: "Generate API documentation, README files, and code comments".to_string(),
            category: "writing".to_string(),
            version: "1.0".to_string(),
            template,
            config: PresetConfig {
                provider: Some("claude".to_string()),
                max_tokens: Some(3000),
                temperature: Some(0.4),
                ..Default::default()
            },
        }
    }
    
    /// Testing preset
    pub fn testing_preset() -> Preset {
        let mut variables = HashMap::new();
        
        variables.insert(
            "test_type".to_string(),
            TemplateVariable::optional_string(
                "Type of tests (unit, integration, e2e)".to_string(),
                "unit".to_string(),
            ),
        );
        
        variables.insert(
            "test_framework".to_string(),
            TemplateVariable::optional_string(
                "Testing framework to use".to_string(),
                "auto-detect".to_string(),
            ),
        );
        
        variables.insert(
            "coverage_focus".to_string(),
            TemplateVariable::boolean("Focus on edge cases and error conditions".to_string(), true),
        );
        
        let template_content = r#"Generate {{test_type}} tests for the following code.

**Test Framework:** {{test_framework}}
**Test Type:** {{test_type}}

Requirements:
- Comprehensive test coverage
- Clear, descriptive test names
- Well-organized test structure
{{#if coverage_focus}}
- Include edge cases and error conditions
- Test boundary conditions
- Verify error handling
{{/if}}
- Follow testing best practices
- Include setup and teardown if needed

{{#if (eq test_type "unit")}}
Focus on:
- Individual function/method testing
- Input/output validation
- Mock external dependencies
- Test all code paths
{{/if}}

{{#if (eq test_type "integration")}}
Focus on:
- Component interaction testing
- Data flow validation
- External service integration
- End-to-end scenarios
{{/if}}

{{#if file_content}}
Code to test:
{{file_content}}
{{else}}
Please provide the code you'd like me to test.
{{/if}}"#;
        
        let template = Template::new(
            "testing".to_string(),
            "Generate comprehensive test suites".to_string(),
            template_content.to_string(),
            variables,
        ).unwrap();
        
        Preset {
            name: "Test Generator".to_string(),
            description: "Generate unit tests, integration tests, and test cases".to_string(),
            category: "testing".to_string(),
            version: "1.0".to_string(),
            template,
            config: PresetConfig {
                provider: Some("claude".to_string()),
                max_tokens: Some(3500),
                temperature: Some(0.2),
                ..Default::default()
            },
        }
    }
    
    /// Debugging preset
    pub fn debugging_preset() -> Preset {
        let mut variables = HashMap::new();
        
        variables.insert(
            "issue_description".to_string(),
            TemplateVariable::required_string("Describe the issue you're experiencing".to_string()),
        );
        
        variables.insert(
            "error_message".to_string(),
            TemplateVariable::optional_string(
                "Error message or stack trace (if any)".to_string(),
                "".to_string(),
            ),
        );
        
        variables.insert(
            "include_logs".to_string(),
            TemplateVariable::boolean("Include log analysis".to_string(), true),
        );
        
        let template_content = r#"Help debug the following issue:

**Issue Description:** {{issue_description}}

{{#if error_message}}
**Error Message/Stack Trace:**
```
{{error_message}}
```
{{/if}}

Please provide:
1. **Root Cause Analysis:**
   - Identify potential causes of the issue
   - Explain why this problem occurs
   
2. **Debugging Steps:**
   - Systematic approach to isolate the problem
   - What to check and in what order
   
3. **Solution Recommendations:**
   - Immediate fixes to resolve the issue
   - Long-term improvements to prevent recurrence
   
4. **Testing Approach:**
   - How to verify the fix works
   - Regression testing considerations

{{#if include_logs}}
5. **Log Analysis:**
   - What to look for in logs
   - Key indicators and patterns
{{/if}}

{{#if file_content}}
Code with issues:
{{file_content}}
{{else}}
Please provide the code or error information you'd like me to debug.
{{/if}}"#;
        
        let template = Template::new(
            "debugging".to_string(),
            "Debug and analyze software issues".to_string(),
            template_content.to_string(),
            variables,
        ).unwrap();
        
        Preset {
            name: "Debugging Assistant".to_string(),
            description: "Analyze errors, debug issues, and provide solutions".to_string(),
            category: "debugging".to_string(),
            version: "1.0".to_string(),
            template,
            config: PresetConfig {
                provider: Some("claude".to_string()),
                max_tokens: Some(3500),
                temperature: Some(0.3),
                ..Default::default()
            },
        }
    }
    
    /// Refactoring preset
    pub fn refactoring_preset() -> Preset {
        let mut variables = HashMap::new();
        
        variables.insert(
            "refactoring_goal".to_string(),
            TemplateVariable::optional_string(
                "Refactoring goal (performance, readability, maintainability)".to_string(),
                "general improvement".to_string(),
            ),
        );
        
        variables.insert(
            "preserve_behavior".to_string(),
            TemplateVariable::boolean("Preserve existing behavior".to_string(), true),
        );
        
        variables.insert(
            "suggest_patterns".to_string(),
            TemplateVariable::boolean("Suggest design patterns".to_string(), true),
        );
        
        let template_content = r#"Suggest refactoring improvements for the following code.

**Refactoring Goal:** {{refactoring_goal}}
{{#if preserve_behavior}}
**Constraint:** Preserve existing functionality and behavior
{{/if}}

Please provide:

1. **Code Analysis:**
   - Identify areas for improvement
   - Highlight code smells and anti-patterns
   - Assess current architecture

2. **Refactoring Suggestions:**
   - Specific improvements with explanations
   - Before/after code examples
   - Step-by-step refactoring approach

3. **Architecture Improvements:**
   - Better code organization
   - Separation of concerns
   {{#if suggest_patterns}}
   - Applicable design patterns
   {{/if}}

4. **Quality Benefits:**
   - How changes improve maintainability
   - Performance implications
   - Testing considerations

5. **Implementation Plan:**
   - Prioritized list of changes
   - Risk assessment for each change
   - Testing strategy

{{#if file_content}}
Code to refactor:
{{file_content}}
{{else}}
Please provide the code you'd like me to refactor.
{{/if}}"#;
        
        let template = Template::new(
            "refactoring".to_string(),
            "Suggest code improvements and refactoring".to_string(),
            template_content.to_string(),
            variables,
        ).unwrap();
        
        Preset {
            name: "Refactoring Assistant".to_string(),
            description: "Suggest code improvements, design patterns, and architecture optimization".to_string(),
            category: "development".to_string(),
            version: "1.0".to_string(),
            template,
            config: PresetConfig {
                provider: Some("claude".to_string()),
                max_tokens: Some(4000),
                temperature: Some(0.4),
                ..Default::default()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_builtin_presets_creation() {
        let presets = BuiltinPresets::get_all();
        assert_eq!(presets.len(), 5);
        
        let names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"Code Review Assistant"));
        assert!(names.contains(&"Documentation Generator"));
        assert!(names.contains(&"Test Generator"));
        assert!(names.contains(&"Debugging Assistant"));
        assert!(names.contains(&"Refactoring Assistant"));
    }
    
    #[test]
    fn test_get_preset_by_name() {
        let preset = BuiltinPresets::get_by_name("Code Review Assistant");
        assert!(preset.is_some());
        
        let preset = preset.unwrap();
        assert_eq!(preset.category, "development");
        assert!(preset.template.variables.contains_key("security"));
    }
    
    #[test]
    fn test_preset_template_validation() {
        let presets = BuiltinPresets::get_all();
        
        for preset in presets {
            // Validate that template syntax is correct
            Template::validate_template_syntax(&preset.template.template)
                .unwrap_or_else(|e| panic!("Invalid template syntax in preset '{}': {}", preset.name, e));
        }
    }
}