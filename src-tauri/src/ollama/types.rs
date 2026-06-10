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
    #[serde(default, skip_serializing_if = "String::is_empty")]
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
    #[serde(rename = "sessionId")]
    #[serde(skip_serializing)]
    pub session_id: String,
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
    pub cancelled: Option<bool>,
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
    #[serde(default)]
    done_reason: Option<String>,
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

        let mut error = self.error;
        if self.done && error.is_none() {
            if matches!(self.done_reason.as_deref(), Some("length")) {
                error = Some(
                    "Response stopped early: output token limit reached. \
                     Try a new chat, a larger model, or increase num_predict."
                        .to_string(),
                );
            } else if matches!(self.done_reason.as_deref(), Some("load")) {
                error = Some(
                    "Response stopped early: prompt is too large for the model context. \
                     Start a new chat or remove older messages."
                        .to_string(),
                );
            }
        }

        ChatChunk {
            thinking: message.thinking.filter(|s| !s.is_empty()),
            content: message.content.filter(|s| !s.is_empty()),
            tool_calls: message.tool_calls,
            done: self.done,
            cancelled: None,
            error,
        }
    }
}

pub fn parse_stream_line(line: &str) -> Result<ChatChunk, String> {
    let parsed: StreamLine = serde_json::from_str(line)
        .map_err(|e| format!("Failed to parse Ollama stream line: {e}"))?;
    Ok(parsed.into_chunk())
}
