const LAST_MODEL_KEY = "local-llm-ui-last-model";
const APP_PREFS_KEY = "local-llm-ui-app-preferences";

export interface AppPreferences {
  maxConcurrentChats: number;
  sidebarCollapsed: boolean;
  lastActiveConversationId: string | null;
}

export const DEFAULT_APP_PREFERENCES: AppPreferences = {
  maxConcurrentChats: 3,
  sidebarCollapsed: false,
  lastActiveConversationId: null,
};

export function loadAppPreferences(): AppPreferences {
  if (typeof localStorage === "undefined") return { ...DEFAULT_APP_PREFERENCES };
  try {
    const raw = localStorage.getItem(APP_PREFS_KEY);
    if (!raw) return { ...DEFAULT_APP_PREFERENCES };
    return { ...DEFAULT_APP_PREFERENCES, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_APP_PREFERENCES };
  }
}

export function saveAppPreferences(prefs: AppPreferences): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(APP_PREFS_KEY, JSON.stringify(prefs));
}

export function loadLastModel(): string {
  if (typeof localStorage === "undefined") return "";
  try {
    const raw = localStorage.getItem(LAST_MODEL_KEY);
    return typeof raw === "string" ? raw : "";
  } catch {
    return "";
  }
}

export function saveLastModel(name: string): void {
  if (typeof localStorage === "undefined" || !name.trim()) return;
  localStorage.setItem(LAST_MODEL_KEY, name.trim());
}

export function pickAvailableModel(
  models: { name: string }[],
  current: string,
  fallback: string,
): string {
  if (current && models.some((m) => m.name === current)) return current;
  if (fallback && models.some((m) => m.name === fallback)) return fallback;
  return models[0]?.name ?? "";
}
