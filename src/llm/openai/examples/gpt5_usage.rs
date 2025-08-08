/// Examples of using GPT-5 features
/// This module demonstrates the new capabilities added for GPT-5
#[allow(dead_code, unused_variables)]
mod examples {
    use crate::llm::openai::adapter::gpt5_adapter::Gpt5Adapter;
    use crate::llm::openai::config::Gpt5Config;
    use crate::llm::openai::model::{
        reasoning_effort::ReasoningEffort,
        responses_api::ResponsesRequest,
        custom_tools::{CustomTool, AllowedToolsChoice, AllowedToolsMode, AllowedToolReference},
        // Temporarily commented out during migration
        // chat_completion_request::ChatCompletionRequest,
        // chat_message::ChatMessage,
    };
    // use crate::llm::common::model::role::Role; // Commented during migration
    use anyhow::Result;

    /// Example: Basic GPT-5 usage with Responses API
    pub async fn basic_gpt5_example() -> Result<()> {
        let adapter = Gpt5Adapter::new();
        let api_key = "your-api-key";

        // Create a simple request with custom reasoning and verbosity
        let request = ResponsesRequest::with_reasoning("gpt-5".to_string(), "Explain quantum computing".to_string(), ReasoningEffort::Medium);

        let response = adapter.responses(&request, api_key).await?;
        
        if let Some(content) = Gpt5Adapter::extract_response_text(&response) {
            println!("GPT-5 Response: {}", content);
        }

        Ok(())
    }

    /// Example: Using custom tools with GPT-5
    pub async fn custom_tools_example() -> Result<()> {
        let adapter = Gpt5Adapter::new();
        let api_key = "your-api-key";

        // Create a custom tool for code execution
        let code_exec_tool = CustomTool::new(
            "code_exec".to_string(),
            "Executes arbitrary Python code and returns the result".to_string(),
        );

        // Create a custom tool with grammar constraints
        let sql_tool = CustomTool::with_grammar(
            "sql_query".to_string(),
            "Executes SQL queries against the database".to_string(),
            "SELECT | INSERT | UPDATE | DELETE".to_string(), // Simplified grammar
        );

        let mut request = ResponsesRequest::simple(
            "gpt-5".to_string(), 
            "Calculate the fibonacci sequence for n=10 using the code_exec tool".to_string()
        );

        request.tools = Some(vec![
            crate::llm::openai::model::responses_api::Tool::Custom(code_exec_tool),
            crate::llm::openai::model::responses_api::Tool::Custom(sql_tool),
        ]);

        request.tool_choice = Some(crate::llm::openai::model::responses_api::ToolChoice::Auto("auto".to_string()));

        let response = adapter.responses(&request, api_key).await?;
        println!("Response with custom tools: {:?}", response);

        Ok(())
    }

    /// Example: Using allowed tools for safety
    pub async fn allowed_tools_example() -> Result<()> {
        let adapter = Gpt5Adapter::new();
        let api_key = "your-api-key";

        // Define multiple tools but restrict usage
        let allowed_choice = AllowedToolsChoice {
            choice_type: "allowed_tools".to_string(),
            mode: AllowedToolsMode::Auto,
            tools: vec![
                AllowedToolReference::custom("safe_calculator".to_string()),
                AllowedToolReference::function("get_weather".to_string()),
            ],
        };

        let mut request = ResponsesRequest::simple(
            "gpt-5".to_string(),
            "Help me calculate 2+2 and get weather for San Francisco".to_string()
        );

        request.tool_choice = Some(crate::llm::openai::model::responses_api::ToolChoice::AllowedTools(allowed_choice));

        let response = adapter.responses(&request, api_key).await?;
        println!("Response with allowed tools: {:?}", response);

        Ok(())
    }

    /// Example: Using configuration presets
    pub async fn config_presets_example() -> Result<()> {
        let adapter = Gpt5Adapter::new();
        let api_key = "your-api-key";

        // Different configurations for different use cases
        let configs = vec![
            ("Coding", Gpt5Config::for_coding()),
            ("Reasoning", Gpt5Config::for_reasoning()), 
            ("Speed", Gpt5Config::for_speed()),
            ("Privacy", Gpt5Config::for_privacy()),
        ];

        for (name, config) in configs {
            println!("Using {} config: {}", name, config.describe());

            let request = Gpt5Adapter::create_simple_request(
                "gpt-5".to_string(),
                "Write a Python function to sort a list".to_string(),
                Some(config.reasoning_effort),
                Some(config.verbosity),
            );

            // In a real implementation, you'd use the response
            println!("Request created for {}: {:?}", name, request.reasoning);
        }

        Ok(())
    }

    /// Example: Migrating from Chat Completions to Responses API
    /// TODO: Remove this example - migration is complete
    #[allow(dead_code)]
    pub async fn migration_example() -> Result<()> {
        // This example is deprecated during migration
        Ok(())
    }
    
    /// Old migration example (commented out)
    #[allow(dead_code)]
    async fn old_migration_example() -> Result<()> {
        // Commented out during migration
        Ok(())
    }

    /// Example: Multi-turn conversation with reasoning persistence
    pub async fn multi_turn_example() -> Result<()> {
        let adapter = Gpt5Adapter::new();
        let api_key = "your-api-key";

        // First turn
        let mut request = ResponsesRequest::simple(
            "gpt-5".to_string(),
            "Let's solve this step by step: What's 15 * 24?".to_string()
        );
        request.store = Some(true); // Enable storage for multi-turn

        let response1 = adapter.responses(&request, api_key).await?;
        println!("First response: {:?}", Gpt5Adapter::extract_response_text(&response1));

        // Second turn - reference previous response
        let mut request2 = ResponsesRequest::simple(
            "gpt-5".to_string(),
            "Now divide that result by 6".to_string()
        );
        request2.previous_response_id = Some(response1.id.clone());
        request2.store = Some(true);

        let response2 = adapter.responses(&request2, api_key).await?;
        println!("Second response: {:?}", Gpt5Adapter::extract_response_text(&response2));

        Ok(())
    }

    /// Example: Zero Data Retention (ZDR) mode with encrypted reasoning
    pub async fn zdr_example() -> Result<()> {
        let adapter = Gpt5Adapter::new();
        let api_key = "your-api-key";

        let mut request = ResponsesRequest::simple(
            "gpt-5".to_string(),
            "Analyze this sensitive data: [redacted for example]".to_string()
        );

        // Enable ZDR mode
        request.store = Some(false);
        if let Some(ref mut reasoning) = request.reasoning {
            // In a real implementation, you'd get this from a previous response
            reasoning.encrypted_content = Some("encrypted_reasoning_tokens_here".to_string());
        }

        let response = adapter.responses(&request, api_key).await?;
        
        // The response will contain reasoning information
        if let Some(reasoning) = &response.reasoning {
            if reasoning.effort.is_some() {
                println!("Received reasoning information for future use");
            }
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[tokio::test]
        async fn test_examples_compile() {
            // These tests just verify the examples compile
            // In practice, you'd need real API keys and mock responses
            
            // Just verify the request creation works
            let request = ResponsesRequest::simple(
                "gpt-5".to_string(),
                "test".to_string()
            );
            assert_eq!(request.model, "gpt-5");
            // Check input content (it's now wrapped in RequestInput enum)
            if let Some(crate::llm::openai::model::responses_api::RequestInput::Text(text)) = &request.input {
                assert_eq!(text, "test");
            } else {
                panic!("Expected text input");
            }
            
            let config = Gpt5Config::for_coding();
            assert_eq!(config.reasoning_effort, ReasoningEffort::Minimal);
        }
    }
}