import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AudioPrepareResult,
  ChatChunk,
  ChatRequest,
  ModelInfo,
} from "./types";

export async function checkOllama(): Promise<void> {
  await invoke("ollama_check");
}

export async function listModels(): Promise<ModelInfo[]> {
  return invoke<ModelInfo[]>("ollama_list_models");
}

export async function streamChat(
  request: ChatRequest,
  onChunk: (chunk: ChatChunk) => void,
): Promise<UnlistenFn> {
  const unlisten = await listen<ChatChunk>("chat-chunk", (event) => {
    onChunk(event.payload);
  });

  await invoke("ollama_chat", { request });
  return unlisten;
}

export function hasCapability(
  model: ModelInfo | undefined,
  capability: string,
): boolean {
  return model?.capabilities.includes(capability) ?? false;
}

/** Models that accept audio via Ollama's `images` field (16 kHz mono WAV). */
export function supportsAudio(model: ModelInfo | undefined): boolean {
  if (!model) return false;
  if (hasCapability(model, "audio")) return true;

  const name = model.name.toLowerCase();
  return (
    name.includes("gemma4") ||
    name.includes("gemma-4") ||
    name.includes("qwen2-audio")
  );
}

export async function prepareAudioForOllama(
  file: File,
): Promise<AudioPrepareResult> {
  const data = Array.from(new Uint8Array(await file.arrayBuffer()));
  return invoke<AudioPrepareResult>("prepare_audio_for_ollama", {
    data,
    filename: file.name,
  });
}

export function formatAudioNote(result: AudioPrepareResult): string | undefined {
  const duration = `${result.duration_secs.toFixed(1)}s`;
  const method = result.method === "ffmpeg" ? "ffmpeg" : "built-in";
  if (result.trimmed) {
    return `${duration} · trimmed to 30s · ${method}`;
  }
  return `${duration} · ${method}`;
}

export function formatBytes(bytes: number): string {
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  if (bytes < 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
