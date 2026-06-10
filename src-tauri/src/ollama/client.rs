use futures_util::StreamExt;
use reqwest::Client;
use tauri::{Emitter, Window};
use tokio::select;

use super::sessions::{cancel as cancel_session, register, remove};
use super::types::{parse_stream_line, ChatChunk, ChatRequest, ModelInfo, ShowResponse, TagsResponse};

const OLLAMA_BASE: &str = "http://127.0.0.1:11434";

fn format_chat_error(status: u16, body: &str) -> String {
    if body.contains("exceed_context_size_error") || body.contains("exceeds the available context size")
    {
        return "Audio or chat history is too large for this model's context window. \
                Try a shorter audio clip (max 30s), start a new chat, or use a model with a larger context."
            .to_string();
    }

    format!("Ollama chat error ({status}): {body}")
}

pub fn cancel_chat(session_id: String) -> Result<(), String> {
    if cancel_session(&session_id) {
        Ok(())
    } else {
        Err(format!("No active stream for session {session_id}"))
    }
}

pub async fn check_connection() -> Result<(), String> {
    Client::new()
        .get(format!("{OLLAMA_BASE}/api/tags"))
        .send()
        .await
        .map_err(|e| format!("Cannot reach Ollama at {OLLAMA_BASE}: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Ollama returned an error: {e}"))?;
    Ok(())
}

pub async fn list_models() -> Result<Vec<ModelInfo>, String> {
    let client = Client::new();
    let response = client
        .get(format!("{OLLAMA_BASE}/api/tags"))
        .send()
        .await
        .map_err(|e| format!("Cannot reach Ollama: {e}"))?
        .error_for_status()
        .map_err(|e| format!("Ollama returned an error: {e}"))?
        .json::<TagsResponse>()
        .await
        .map_err(|e| format!("Failed to parse model list: {e}"))?;

    let mut models = Vec::with_capacity(response.models.len());
    for model in response.models {
        let capabilities = if model.capabilities.is_empty() {
            fetch_capabilities(&client, &model.name).await.unwrap_or_default()
        } else {
            model.capabilities
        };

        models.push(ModelInfo {
            name: model.name,
            size: model.size,
            capabilities,
        });
    }

    models.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(models)
}

async fn fetch_capabilities(client: &Client, model: &str) -> Result<Vec<String>, String> {
    let response = client
        .post(format!("{OLLAMA_BASE}/api/show"))
        .json(&serde_json::json!({ "model": model }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json::<ShowResponse>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(response.capabilities)
}

fn emit_cancelled(window: &Window, event_name: &str) -> Result<(), String> {
    window
        .emit(
            event_name,
            &ChatChunk {
                thinking: None,
                content: None,
                tool_calls: None,
                done: true,
                cancelled: Some(true),
                error: None,
            },
        )
        .map_err(|e| format!("Failed to emit cancel chunk: {e}"))
}

pub async fn stream_chat(window: Window, request: ChatRequest) -> Result<(), String> {
    let session_id = request.session_id.clone();
    let event_name = format!("chat-chunk-{session_id}");
    let cancel_token = register(&session_id);

    let client = Client::new();
    let response = client
        .post(format!("{OLLAMA_BASE}/api/chat"))
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Chat request failed: {e}"))?;

    if !response.status().is_success() {
        remove(&session_id);
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format_chat_error(status.as_u16(), &body));
    }

    let mut byte_stream = response.bytes_stream();
    let mut buffer = String::new();
    let mut was_cancelled = false;

    loop {
        if cancel_token.is_cancelled() {
            was_cancelled = true;
            break;
        }

        let chunk_result = select! {
            biased;
            _ = cancel_token.cancelled() => {
                was_cancelled = true;
                break;
            }
            next = byte_stream.next() => next,
        };

        let Some(chunk) = chunk_result else {
            break;
        };

        let chunk = chunk.map_err(|e| format!("Stream read failed: {e}"))?;
        if chunk.is_empty() {
            continue;
        }

        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some(newline_idx) = buffer.find('\n') {
            if cancel_token.is_cancelled() {
                was_cancelled = true;
                break;
            }

            let line = buffer[..newline_idx].trim().to_string();
            buffer.drain(..=newline_idx);

            if line.is_empty() {
                continue;
            }

            let chat_chunk = match parse_stream_line(&line) {
                Ok(chunk) => chunk,
                Err(error) => ChatChunk {
                    thinking: None,
                    content: None,
                    tool_calls: None,
                    done: true,
                    cancelled: None,
                    error: Some(error),
                },
            };

            window
                .emit(&event_name, &chat_chunk)
                .map_err(|e| format!("Failed to emit chat chunk: {e}"))?;

            if chat_chunk.done {
                remove(&session_id);
                return Ok(());
            }
        }

        if was_cancelled {
            break;
        }
    }

    if was_cancelled {
        emit_cancelled(&window, &event_name)?;
    } else {
        window
            .emit(
                &event_name,
                &ChatChunk {
                    thinking: None,
                    content: None,
                    tool_calls: None,
                    done: true,
                    cancelled: Some(false),
                    error: None,
                },
            )
            .map_err(|e| format!("Failed to emit final chunk: {e}"))?;
    }

    remove(&session_id);
    Ok(())
}
