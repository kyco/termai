# GPT-5.1 Features in TermAI

This document outlines the new GPT-5.1 features implemented in TermAI, including usage examples and best practices.

## Overview

TermAI now fully supports GPT-5.1's advanced capabilities, including:

- **GPT-5.1 Model Family**: `gpt-5.1`, `gpt-5-mini`, `gpt-5-nano`
- **Enhanced Reasoning**: None, Low, Medium, High reasoning efforts
- **Verbosity Control**: Low, Medium, High response lengths
- **Dual API Support**: Both Chat Completions and new Responses API
- **Custom Tools**: Freeform text input with optional grammar constraints
- **Allowed Tools**: Restricted tool usage for safety
- **Preambles**: Explanations before tool calls
- **Zero Data Retention**: Encrypted reasoning for privacy

## Quick Start

### Basic Usage

```bash
# Set provider to OpenAI (if not already set)
termai config set-provider openai

# Set your OpenAI API key
termai config set-openai your-api-key-here

# Start chatting with GPT-5.1 (now the default)
termai chat
```

### Model Selection

GPT-5.1 offers three variants:

- **`gpt-5.1`**: Best for complex reasoning, broad world knowledge, code-heavy tasks
- **`gpt-5-mini`**: Cost-optimized reasoning, balances speed/cost/capability  
- **`gpt-5-nano`**: High-throughput tasks, simple instruction-following

## New Features

### 1. Enhanced Reasoning Effort

GPT-5.1 introduces a new "none" reasoning level, perfect for low-latency interactions:

```rust
// The commit command now uses none reasoning for faster code generation
reasoning_effort: ReasoningEffort::None
```

**Reasoning Levels:**
- `None`: Fastest, best for low-latency interactions and simple tasks
- `Low`: Quick responses with basic reasoning
- `Medium`: Balanced reasoning
- `High`: Thorough reasoning for complex problems (default for TermAI coding tasks)

### 2. Verbosity Control

Control response length independently of reasoning:

```rust
// Available verbosity levels
pub enum Verbosity {
    Low,    // Concise answers, minimal commentary
    Medium, // Balanced explanations (default)
    High,   // Thorough explanations and extensive details
}
```

### 3. Responses API

TermAI automatically uses the optimal API:

- **GPT-5.1 models**: Prefer Responses API for better performance
- **Older models**: Use Chat Completions for compatibility
- **Auto-detection**: Based on model and requested features

**Benefits of Responses API:**
- Better caching and lower latency
- Chain of thought persistence between turns
- Support for encrypted reasoning (ZDR mode)

### 4. Custom Tools

Define tools that accept freeform text instead of structured JSON:

```rust
let code_exec_tool = CustomTool::new(
    "code_exec".to_string(),
    "Executes arbitrary Python code and returns the result".to_string(),
);
```

**Features:**
- Freeform text input (code, SQL, shell commands, prose)
- Optional grammar constraints using context-free grammars
- Direct integration with tool calling

### 5. Allowed Tools

Restrict which tools the model can use for safety:

```rust
let allowed_choice = AllowedToolsChoice {
    choice_type: "allowed_tools".to_string(),
    mode: AllowedToolsMode::Auto,
    tools: vec![
        AllowedToolReference::custom("safe_calculator".to_string()),
        AllowedToolReference::function("get_weather".to_string()),
    ],
};
```

**Benefits:**
- Enhanced safety by limiting tool access
- Better prompt caching
- Reduced risk of unintended tool usage

### 6. Configuration Presets

TermAI provides optimized configurations for common use cases:

```rust
// Optimized for different scenarios
let configs = vec![
    Gpt5Config::for_coding(),    // High reasoning, medium verbosity (for complex coding and agentic tasks)
    Gpt5Config::for_reasoning(), // High reasoning, high verbosity, preambles
    Gpt5Config::for_speed(),     // None reasoning, low verbosity
    Gpt5Config::for_privacy(),   // ZDR mode, no storage
];
```

## Usage Examples

### Coding Tasks

The system is automatically optimized for coding with GPT-5.1:

```bash
# The commit command now uses high reasoning with medium verbosity for complex coding and agentic tasks
termai commit

# Interactive coding sessions benefit from the optimized settings
termai chat src/
```

### Complex Reasoning

For complex analysis, TermAI automatically adapts:

```bash
# Multi-turn conversations maintain reasoning context
termai chat --session analysis

# Smart context discovery works better with GPT-5.1's improved reasoning
termai chat --smart-context
```

### Privacy Mode

For sensitive data, enable zero data retention:

```rust
// In configuration
let config = Gpt5Config::for_privacy()
    .with_zdr(true);  // Enables encrypted reasoning items
```

## Migration from Previous Models

### From GPT-5 to GPT-5.1
- OpenAI's GPT-5.1 defaults to "none" reasoning for low-latency
- TermAI defaults to "high" reasoning with "medium" verbosity for coding and agentic tasks
- GPT-5.1 is a drop-in replacement for GPT-5

### From o3/o4 Models
- Use `gpt-5.1` with medium or high reasoning
- Start with medium reasoning, increase to high if needed

### From gpt-4.1
- Use `gpt-5.1` with none or low reasoning
- Start with none reasoning and tune prompts

### From gpt-4.1-mini/o4-mini
- Use `gpt-5-mini` with prompt tuning

### From gpt-4.1-nano
- Use `gpt-5-nano` with prompt tuning

## Best Practices

### For Coding and Agentic Tasks
- Use high reasoning effort with medium verbosity for complex coding, bug fixing, and multi-step planning
- Use none reasoning for simple, quick tasks
- Enable preambles for complex tool-calling scenarios
- Medium verbosity provides good balance of detail and speed

### For Analysis
- Use high reasoning effort for complex problems
- Enable conversation storage for multi-turn context
- High verbosity for thorough explanations

### For Production
- Consider ZDR mode for sensitive data
- Use allowed tools for safety constraints
- Enable preambles for transparency

## Performance Optimizations

GPT-5.1 in TermAI includes several performance improvements:

1. **Automatic API Selection**: Uses Responses API for GPT-5.1 models
2. **Reasoning Context**: Maintains chain of thought between turns
3. **Smart Caching**: Better cache hit rates with Responses API
4. **High Reasoning Default**: Optimized for complex coding and agentic tasks

## Configuration

TermAI's GPT-5.1 configuration is backward compatible but adds new options:

```toml
# .termai.toml
[providers.openai]
model = "gpt-5.1"
reasoning_effort = "high"
verbosity = "medium"
prefer_responses_api = true
preambles = false
store_conversations = false
zero_data_retention = false
```

## Troubleshooting

### Common Issues

1. **API Errors**: Ensure you're using a GPT-5.1 compatible API key
2. **Feature Not Available**: Some features require the Responses API
3. **Performance**: Try reducing reasoning effort or verbosity for faster responses

### Debug Information

Enable debug mode to see which API and features are being used:

```bash
TERMAI_DEBUG=1 termai chat
```

## Implementation Details

The GPT-5.1 integration includes:

- **New Model Types**: Added `Gpt5`, `Gpt5Mini`, `Gpt5Nano` to model enum
- **Enhanced Requests**: Support for verbosity, custom tools, allowed tools
- **Dual Adapters**: Both `open_ai_adapter` and `gpt5_adapter` available  
- **Automatic Migration**: Intelligent API selection based on model and features
- **Configuration System**: Flexible config with sensible presets

## Future Enhancements

Planned improvements:

1. **Interactive Tool Selection**: UI for choosing allowed tools
2. **Grammar Editor**: Visual editor for custom tool grammars
3. **Reasoning Visualization**: Display chain of thought in debug mode
4. **Performance Metrics**: Track reasoning tokens and response times
5. **Advanced Preambles**: Configurable preamble templates

---

For more examples and detailed API usage, see `src/llm/openai/examples/gpt5_usage.rs`.