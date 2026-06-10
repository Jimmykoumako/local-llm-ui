export interface TtsSettings {
  enabled: boolean;
  autoSpeak: boolean;
  autoDetectLanguage: boolean;
  kokoroPath: string;
  voice: string;
  speed: number;
}

const STORAGE_KEY = "local-llm-ui-tts-settings";

export const DEFAULT_TTS_SETTINGS: TtsSettings = {
  enabled: false,
  autoSpeak: false,
  autoDetectLanguage: true,
  kokoroPath: "",
  voice: "af_heart",
  speed: 1.0,
};

export const KOKORO_VOICE_SUGGESTIONS = [
  "af_heart",
  "af_bella",
  "af_sarah",
  "am_adam",
  "am_michael",
  "bf_emma",
  "bm_george",
];

export function loadTtsSettings(): TtsSettings {
  if (typeof localStorage === "undefined") return { ...DEFAULT_TTS_SETTINGS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_TTS_SETTINGS };
    const parsed = JSON.parse(raw) as Partial<TtsSettings>;
    return {
      ...DEFAULT_TTS_SETTINGS,
      ...parsed,
      autoDetectLanguage:
        typeof parsed.autoDetectLanguage === "boolean"
          ? parsed.autoDetectLanguage
          : DEFAULT_TTS_SETTINGS.autoDetectLanguage,
      speed:
        typeof parsed.speed === "number" && parsed.speed > 0
          ? parsed.speed
          : DEFAULT_TTS_SETTINGS.speed,
    };
  } catch {
    return { ...DEFAULT_TTS_SETTINGS };
  }
}

export function saveTtsSettings(settings: TtsSettings): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}
