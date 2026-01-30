use serde::{Serialize, Deserialize};

/// Request structure for the /v1/responses/compact endpoint
#[derive(Serialize, Debug, Clone)]
pub struct CompactRequest {
    /// Model to use for compaction
    pub model: String,

    /// Input items to compact - can be messages or existing compaction items
    pub input: Vec<CompactInputItem>,

    /// Optional system instructions to preserve
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Input items for compaction - either a message or an existing compaction
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum CompactInputItem {
    /// A regular message to include in compaction
    Message {
        role: String,
        content: String,
    },
    /// An existing compaction item to include
    Compaction {
        #[serde(rename = "type")]
        item_type: String, // "compaction"
        id: String,
        encrypted_content: String,
    },
}

impl CompactInputItem {
    /// Create a new message input item
    pub fn message(role: String, content: String) -> Self {
        CompactInputItem::Message { role, content }
    }

    /// Create a new compaction input item
    pub fn compaction(id: String, encrypted_content: String) -> Self {
        CompactInputItem::Compaction {
            item_type: "compaction".to_string(),
            id,
            encrypted_content,
        }
    }
}

/// Response structure from the /v1/responses/compact endpoint
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct CompactResponse {
    /// Response ID
    pub id: String,

    /// Output items - compacted messages and compaction blobs
    pub output: Vec<CompactOutputItem>,

    /// Usage statistics
    pub usage: Option<CompactUsage>,
}

/// Output items from compaction - either a message or a compaction blob
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum CompactOutputItem {
    /// A message that was preserved (e.g., recent messages)
    #[serde(rename = "message")]
    Message {
        role: String,
        content: Vec<CompactContentItem>,
    },
    /// A compaction blob containing encrypted conversation history
    #[serde(rename = "compaction")]
    Compaction {
        id: String,
        encrypted_content: String,
    },
}

/// Content item within a message output
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum CompactContentItem {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "output_text")]
    OutputText { text: String },
}

impl CompactContentItem {
    /// Extract the text content from this item
    pub fn text(&self) -> &str {
        match self {
            CompactContentItem::Text { text } => text,
            CompactContentItem::OutputText { text } => text,
        }
    }
}

/// Usage statistics for compaction
#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct CompactUsage {
    /// Input tokens processed
    pub input_tokens: u32,

    /// Output tokens generated
    pub output_tokens: u32,

    /// Total tokens
    pub total_tokens: u32,
}

impl CompactRequest {
    /// Create a new compact request
    pub fn new(model: String, input: Vec<CompactInputItem>) -> Self {
        Self {
            model,
            input,
            instructions: None,
        }
    }

    /// Create a compact request with system instructions
    #[allow(dead_code)]
    pub fn with_instructions(model: String, input: Vec<CompactInputItem>, instructions: String) -> Self {
        Self {
            model,
            input,
            instructions: Some(instructions),
        }
    }
}
