import { invoke } from "@tauri-apps/api/core";

const OLLAMA_HOST_KEY = "local-llm-ui-ollama-host";
const OLLAMA_SETUP_DISMISSED_KEY = "local-llm-ui-ollama-setup-dismissed";

export const DEFAULT_OLLAMA_HOST = "http://127.0.0.1:11434";

export function loadOllamaHost(): string {
  if (typeof localStorage === "undefined") return DEFAULT_OLLAMA_HOST;
  try {
    const raw = localStorage.getItem(OLLAMA_HOST_KEY);
    return raw?.trim() || DEFAULT_OLLAMA_HOST;
  } catch {
    return DEFAULT_OLLAMA_HOST;
  }
}

export function saveOllamaHost(host: string): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(OLLAMA_HOST_KEY, host.trim());
}

export function loadOllamaSetupDismissed(): boolean {
  if (typeof localStorage === "undefined") return false;
  try {
    return localStorage.getItem(OLLAMA_SETUP_DISMISSED_KEY) === "1";
  } catch {
    return false;
  }
}

export function saveOllamaSetupDismissed(dismissed: boolean): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(OLLAMA_SETUP_DISMISSED_KEY, dismissed ? "1" : "0");
}

export async function applyOllamaHost(host: string): Promise<string> {
  const normalized = await invoke<string>("ollama_normalize_host", { host });
  await invoke("ollama_set_host", { host: normalized });
  saveOllamaHost(normalized);
  return normalized;
}

export async function syncOllamaHostFromStorage(): Promise<string> {
  return applyOllamaHost(loadOllamaHost());
}
