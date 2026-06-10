import { invoke } from "@tauri-apps/api/core";
import type { TtsSettings } from "./tts-settings";
import { textForTts } from "./tts-markdown";
import { wavBase64ToUrl, revokeBlobUrl } from "./utils/audio";

interface TtsResult {
  base64: string;
  mime: string;
  detected_lang?: string | null;
  kokoro_lang?: string | null;
}

interface KokoroDetectResult {
  found: boolean;
  path?: string | null;
}

let activeAudio: HTMLAudioElement | null = null;
let activeUrl: string | null = null;
let aborted = false;
let speaking = false;
const listeners = new Set<(value: boolean) => void>();

function setSpeaking(value: boolean) {
  speaking = value;
  for (const listener of listeners) listener(value);
}

export function getIsSpeaking(): boolean {
  return speaking;
}

export function subscribeSpeaking(listener: (value: boolean) => void): () => void {
  listeners.add(listener);
  listener(speaking);
  return () => listeners.delete(listener);
}

export async function detectKokoro(customPath = ""): Promise<KokoroDetectResult> {
  return invoke<KokoroDetectResult>("tts_detect_kokoro", { customPath });
}

export { extractFencedBlocks, textForTts } from "./tts-markdown";

/** @deprecated Use textForTts from tts-markdown. */
export function stripMarkdownForTts(text: string): string {
  return textForTts(text);
}

function clearPlayback(): void {
  if (activeAudio) {
    activeAudio.pause();
    activeAudio.onended = null;
    activeAudio.onerror = null;
    activeAudio = null;
  }
  if (activeUrl) {
    revokeBlobUrl(activeUrl);
    activeUrl = null;
  }
}

export function stopSpeech(): void {
  aborted = true;
  clearPlayback();
  void invoke("tts_cancel").catch(() => {});
  setSpeaking(false);
}

export async function synthesizeSpeech(
  text: string,
  settings: TtsSettings,
  options?: { raw?: boolean },
): Promise<{ url: string; detectedLang?: string; kokoroLang?: string }> {
  const spoken = options?.raw ? text.trim() : textForTts(text);
  if (!spoken) {
    throw new Error("Nothing to speak after removing formatting.");
  }

  if (aborted) {
    throw new Error("TTS cancelled");
  }

  const result = await invoke<TtsResult>("tts_synthesize", {
    text: spoken,
    kokoroPath: settings.kokoroPath,
    voice: settings.voice,
    speed: settings.speed,
    autoDetectLanguage: settings.autoDetectLanguage,
  });

  if (aborted) {
    throw new Error("TTS cancelled");
  }

  return {
    url: wavBase64ToUrl(result.base64),
    detectedLang: result.detected_lang ?? undefined,
    kokoroLang: result.kokoro_lang ?? undefined,
  };
}

export async function speakText(
  text: string,
  settings: TtsSettings,
  options?: { raw?: boolean },
): Promise<void> {
  clearPlayback();
  void invoke("tts_cancel").catch(() => {});
  aborted = false;
  setSpeaking(true);

  try {
    const { url } = await synthesizeSpeech(text, settings, options);
    if (aborted) return;

    activeUrl = url;
    const audio = new Audio(url);
    activeAudio = audio;

    await new Promise<void>((resolve, reject) => {
      audio.onended = () => resolve();
      audio.onerror = () => reject(new Error("Audio playback failed"));
      void audio.play().catch(reject);
    });
  } catch (error) {
    if (aborted || String(error).includes("cancelled")) return;
    throw error;
  } finally {
    clearPlayback();
    setSpeaking(false);
  }
}
