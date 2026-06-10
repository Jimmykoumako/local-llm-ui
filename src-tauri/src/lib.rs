mod audio;
mod fs;
mod git;
mod ollama;
mod shell;
mod tts;

use audio::AudioPrepareResult;
use ollama::{
    cancel_chat, check_connection, list_models, stream_chat, ChatRequest, ModelInfo,
};
use tauri::Window;

#[tauri::command]
async fn ollama_check() -> Result<(), String> {
    check_connection().await
}

#[tauri::command]
async fn ollama_list_models() -> Result<Vec<ModelInfo>, String> {
    list_models().await
}

#[tauri::command]
async fn ollama_chat(window: Window, request: ChatRequest) -> Result<(), String> {
    stream_chat(window, request).await
}

#[tauri::command]
fn ollama_chat_cancel(session_id: String) -> Result<(), String> {
    cancel_chat(session_id)
}

#[tauri::command]
async fn prepare_audio_for_ollama(
    data: Vec<u8>,
    filename: String,
) -> Result<AudioPrepareResult, String> {
    tokio::task::spawn_blocking(move || audio::convert_to_ollama_wav(&data, &filename))
        .await
        .map_err(|e| format!("Audio conversion task failed: {e}"))?
}

#[tauri::command]
fn tts_detect_kokoro(custom_path: String) -> tts::KokoroDetectResult {
    tts::detect_kokoro(&custom_path)
}

#[tauri::command]
fn tts_cancel() {
    tts::cancel_synthesis();
}

#[tauri::command]
fn save_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content.as_bytes()).map_err(|e| format!("Failed to save file: {e}"))
}

#[tauri::command]
async fn tts_synthesize(
    text: String,
    kokoro_path: String,
    voice: String,
    speed: f32,
    auto_detect_language: bool,
) -> Result<tts::TtsResult, String> {
    let speed = if speed > 0.0 { speed } else { 1.0 };
    tokio::task::spawn_blocking(move || {
        tts::synthesize(
            &text,
            &kokoro_path,
            &voice,
            speed,
            auto_detect_language,
        )
    })
    .await
    .map_err(|e| format!("TTS task failed: {e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            ollama_check,
            ollama_list_models,
            ollama_chat,
            ollama_chat_cancel,
            prepare_audio_for_ollama,
            save_text_file,
            tts_detect_kokoro,
            tts_cancel,
            tts_synthesize,
            fs::agent_get_default_roots,
            fs::agent_execute_tool,
            fs::agent_classify_tool,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
