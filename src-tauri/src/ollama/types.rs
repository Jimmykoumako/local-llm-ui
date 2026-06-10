use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size: u64,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub think: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(default = "default_stream")]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Value>,
}

fn default_stream() -> bool {
    true
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatChunk {
    pub thinking: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<Value>>,
    pub done: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TagsResponse {
    pub models: Vec<TagModel>,
}

#[derive(Debug, Deserialize)]
pub struct TagModel {
    pub name: String,
    pub size: u64,
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ShowResponse {
    #[serde(default)]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StreamLine {
    message: Option<StreamMessage>,
    done: bool,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct StreamMessage {
    thinking: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<Value>>,
}

impl StreamLine {
    pub fn into_chunk(self) -> ChatChunk {
        let message = self.message.unwrap_or(StreamMessage {
            thinking: None,
            content: None,
            tool_calls: None,
        });

        ChatChunk {
            thinking: message.thinking.filter(|s| !s.is_empty()),
            content: message.content.filter(|s| !s.is_empty()),
            tool_calls: message.tool_calls,
            done: self.done,
            error: self.error,
        }
    }
}

pub fn parse_stream_line(line: &str) -> Result<ChatChunk, String> {
    let parsed: StreamLine = serde_json::from_str(line)
        .map_err(|e| format!("Failed to parse Ollama stream line: {e}"))?;
    Ok(parsed.into_chunk())
}
