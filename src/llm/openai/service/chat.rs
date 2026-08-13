use crate::llm::common::model::role::Role;
use crate::llm::openai::{
    adapter::responses_adapter::ResponsesAdapter,
    model::{
        model::Model,
        responses_api::{
            CompactionInputItem, ContentItem, ExtendedInputItem, InputMessage, ResponseOutput,
            ResponsesRequest, ToolChoice,
        },
    },
    service::compaction,
    tools::{builtin::get_enabled_tools, executor::ToolExecutor},
};
use crate::session::model::message::{Message, MessageType};
use crate::session::model::session::Session;
use crate::ui::web_indicator::WebIndicator;
use anyhow::{anyhow, Result};

/// Chat without tools - simple request/response
#[allow(dead_code)]
pub async fn chat(api_key: &str, session: &mut Session) -> Result<()> {
    chat_with_model(api_key, session, None).await
}

pub async fn chat_with_model(
    api_key: &str,
    session: &mut Session,
    model_param: Option<&str>,
) -> Result<()> {
    chat_internal(api_key, session, false, model_param).await
}

/// Chat with tools enabled - allows model to execute bash, read/write files, etc.
pub async fn chat_with_tools(api_key: &str, session: &mut Session) -> Result<()> {
    chat_internal(api_key, session, true, None).await
}

/// Maximum characters for tool output before truncation
const MAX_TOOL_OUTPUT_SIZE: usize = 8000;

/// Maximum tool execution iterations
const MAX_TOOL_ITERATIONS: usize = 5;

/// Truncate tool output if too long
fn truncate_tool_output(output: &str) -> String {
    if output.len() <= MAX_TOOL_OUTPUT_SIZE {
        output.to_string()
    } else {
        let half = MAX_TOOL_OUTPUT_SIZE / 2;
        format!(
            "{}\n\n... [truncated {} characters] ...\n\n{}",
            &output[..half],
            output.len() - MAX_TOOL_OUTPUT_SIZE,
            &output[output.len() - half..]
        )
    }
}

/// Internal chat implementation that optionally enables tools
async fn chat_internal(
    api_key: &str,
    session: &mut Session,
    enable_tools: bool,
    model_param: Option<&str>,
) -> Result<()> {
    let model_name = model_param
        .map(str::to_string)
        .unwrap_or_else(|| Model::Gpt5.to_string());
    let executor = ToolExecutor::new(std::env::current_dir()?);

    // Check and perform compaction if needed (before the main loop)
    if compaction::needs_compaction(session) {
        compaction::try_compact_session(api_key, session, &model_name).await;
    }

    let mut iteration = 0;

    loop {
        iteration += 1;
        if iteration > MAX_TOOL_ITERATIONS {
            // Force a final response without tools
            return finish_without_tools(api_key, session, &model_name).await;
        }

        // Check if we have any compaction items (requires extended input format)
        let has_compaction = session
            .messages
            .iter()
            .any(|m| matches!(m.message_type, MessageType::Compaction { .. }));

        // Build the request based on whether we have compaction items
        let request = if has_compaction {
            // Use extended input format to support compaction items
            let extended_items: Vec<ExtendedInputItem> = session
                .messages
                .iter()
                .map(|m| match &m.message_type {
                    MessageType::Standard => ExtendedInputItem::Message(InputMessage {
                        role: m.role.to_string(),
                        content: m.content.clone(),
                    }),
                    MessageType::Compaction {
                        compaction_id,
                        encrypted_content,
                    } => ExtendedInputItem::Compaction(CompactionInputItem::new(
                        compaction_id.clone(),
                        encrypted_content.clone(),
                    )),
                })
                .collect();

            ResponsesRequest::from_extended(model_name.clone(), extended_items)
        } else {
            // Standard message format
            let input_messages: Vec<InputMessage> = session
                .messages
                .iter()
                .map(|m| InputMessage {
                    role: m.role.to_string(),
                    content: m.content.to_string(),
                })
                .collect();

            // Check total input size to prevent hanging on extremely large inputs
            let total_input_size: usize = input_messages.iter().map(|m| m.content.len()).sum();

            if total_input_size > 500_000 {
                // 500KB limit
                return Err(anyhow!(
                    "Input too large ({} characters). Please reduce input size to under 500,000 characters.",
                    total_input_size
                ));
            }

            // Create the request
            if input_messages.len() == 1 && input_messages[0].role == "user" {
                // For single user message, use simple text input
                ResponsesRequest::simple(model_name.clone(), input_messages[0].content.clone())
            } else {
                // For conversation, use messages format
                ResponsesRequest::from_messages(model_name.clone(), input_messages)
            }
        };

        let mut request = request;

        // Add tools if enabled (but warn model if approaching limit)
        if enable_tools {
            request.tools = Some(get_enabled_tools());
            request.tool_choice = Some(ToolChoice::Auto("auto".to_string()));

            // Add guidance when approaching iteration limit
            if iteration >= MAX_TOOL_ITERATIONS - 1 {
                request.instructions = Some(
                    "IMPORTANT: You are approaching the tool usage limit. \
                    Please provide your final answer based on the information gathered so far. \
                    Do not make any more tool calls."
                        .to_string(),
                );
            }
        }

        // Make the request
        let response = ResponsesAdapter::chat(&request, api_key).await?;

        // Check if request was successful
        if response.status != "completed" {
            if let Some(error) = response.error {
                return Err(anyhow!("OpenAI API error: {}", error.message));
            } else {
                return Err(anyhow!("Request failed with status: {}", response.status));
            }
        }

        // Process the output and collect tool calls
        let mut tool_calls = Vec::new();

        for output in response.output {
            match output {
                ResponseOutput::Message { role, content, .. } => {
                    // Extract text from content items
                    let message_text = extract_text_from_content(content);
                    if !message_text.is_empty() {
                        session.messages.push(Message::new(
                            "".to_string(),
                            Role::from_str(&role),
                            message_text,
                        ));
                    }
                }
                ResponseOutput::FunctionCall {
                    id,
                    name,
                    arguments,
                    ..
                } => {
                    tool_calls.push((id, name, arguments));
                }
                ResponseOutput::Reasoning { .. } => {
                    // Skip reasoning output - it's metadata, not user-facing content
                }
            }
        }

        // If no tool calls, we're done
        if tool_calls.is_empty() {
            break;
        }

        // Execute tools and add results to session
        for (call_id, tool_name, tool_arguments) in tool_calls {
            // Show a transient animated indicator while web tools execute
            let mut web_indicator = if is_web_tool(&tool_name) {
                let mut indicator = WebIndicator::new();
                indicator.start(web_indicator_message(&tool_name, &tool_arguments));
                Some(indicator)
            } else {
                None
            };

            let result = executor.execute(&tool_name, &tool_arguments).await;

            if let Some(indicator) = web_indicator.as_mut() {
                indicator.stop();
            }

            let result = result?;

            // Truncate large tool outputs to prevent context explosion
            let truncated_output = truncate_tool_output(&result.output);

            // Format the tool result message
            let status = if result.success { "success" } else { "error" };
            let result_content = format!(
                "[Tool: {} ({})] {}\n\nResult:\n{}",
                tool_name, status, call_id, truncated_output
            );

            // Add tool result as a user message so model can see it
            session
                .messages
                .push(Message::new("".to_string(), Role::User, result_content));
        }

        // Loop continues - model will respond to tool results
    }

    Ok(())
}

/// Force a final response without tools when iteration limit is exceeded
async fn finish_without_tools(api_key: &str, session: &mut Session, model: &str) -> Result<()> {
    // Add a message asking the model to summarize
    session.messages.push(Message::new(
        "".to_string(),
        Role::User,
        "[System: Tool usage limit reached. Please provide your final answer based on the information gathered above.]".to_string(),
    ));

    // Build request without tools
    let input_messages: Vec<InputMessage> = session
        .messages
        .iter()
        .map(|m| InputMessage {
            role: m.role.to_string(),
            content: m.content.to_string(),
        })
        .collect();

    let request = ResponsesRequest::from_messages(model.to_string(), input_messages);

    let response = ResponsesAdapter::chat(&request, api_key).await?;

    if response.status != "completed" {
        if let Some(error) = response.error {
            return Err(anyhow!("OpenAI API error: {}", error.message));
        } else {
            return Err(anyhow!("Request failed with status: {}", response.status));
        }
    }

    // Extract the final message
    for output in response.output {
        if let ResponseOutput::Message { role, content, .. } = output {
            let message_text = extract_text_from_content(content);
            if !message_text.is_empty() {
                session.messages.push(Message::new(
                    "".to_string(),
                    Role::from_str(&role),
                    message_text,
                ));
            }
        }
    }

    Ok(())
}

/// Whether a tool is a (read-only) web tool that shows the transient indicator
fn is_web_tool(tool_name: &str) -> bool {
    matches!(tool_name, "web_search" | "web_fetch")
}

/// Build the transient indicator message for a web tool call
fn web_indicator_message(tool_name: &str, arguments: &str) -> String {
    let parsed: serde_json::Value = serde_json::from_str(arguments).unwrap_or_default();
    match tool_name {
        "web_search" => match parsed.get("query").and_then(|v| v.as_str()) {
            Some(query) if !query.is_empty() => format!("Searching the web for \"{}\"…", query),
            _ => "Searching the web…".to_string(),
        },
        _ => match parsed.get("url").and_then(|v| v.as_str()) {
            Some(url) if !url.is_empty() => format!("Fetching {}…", host_of(url)),
            _ => "Fetching page…".to_string(),
        },
    }
}

/// Extract the host portion of a URL for display purposes
fn host_of(url: &str) -> String {
    let without_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(without_scheme)
        .to_string()
}

/// Extract text content from content items
fn extract_text_from_content(content: Vec<ContentItem>) -> String {
    content
        .into_iter()
        .map(|item| match item {
            ContentItem::OutputText { text, .. } => text,
        })
        .collect::<Vec<String>>()
        .join("\n")
}
