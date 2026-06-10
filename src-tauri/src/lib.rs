mod audio;
mod ollama;

use audio::AudioPrepareResult;
use ollama::{check_connection, list_models, stream_chat, ChatRequest, ModelInfo};
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
async fn prepare_audio_for_ollama(
    data: Vec<u8>,
    filename: String,
) -> Result<AudioPrepareResult, String> {
    tokio::task::spawn_blocking(move || audio::convert_to_ollama_wav(&data, &filename))
        .await
        .map_err(|e| format!("Audio conversion task failed: {e}"))?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            ollama_check,
            ollama_list_models,
            ollama_chat,
            prepare_audio_for_ollama
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
