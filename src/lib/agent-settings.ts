import { invoke } from "@tauri-apps/api/core";

export type AgentPermissionLevel = "off" | "read" | "write" | "full";
export type ApprovalPolicy = "always" | "destructive" | "session" | "auto";

export interface AgentSettings {
  enabled: boolean;
  level: AgentPermissionLevel;
  approvalPolicy: ApprovalPolicy;
  allowedRoots: string[];
  maxReadBytes: number;
  maxSearchResults: number;
  shellEnabled: boolean;
  gitEnabled: boolean;
}

const STORAGE_KEY = "local-llm-ui-agent-settings";

export const DEFAULT_AGENT_SETTINGS: AgentSettings = {
  enabled: false,
  level: "write",
  approvalPolicy: "destructive",
  allowedRoots: [],
  maxReadBytes: 512_000,
  maxSearchResults: 50,
  shellEnabled: false,
  gitEnabled: true,
};

export function loadAgentSettings(): AgentSettings {
  if (typeof localStorage === "undefined") return { ...DEFAULT_AGENT_SETTINGS };
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULT_AGENT_SETTINGS };
    return { ...DEFAULT_AGENT_SETTINGS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_AGENT_SETTINGS };
  }
}

export function saveAgentSettings(settings: AgentSettings): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
}

export async function ensureAgentRoots(
  settings: AgentSettings,
): Promise<AgentSettings> {
  if (settings.allowedRoots.length > 0) return settings;
  try {
    const roots = await invoke<string[]>("agent_get_default_roots");
    return { ...settings, allowedRoots: roots };
  } catch {
    return settings;
  }
}

export function levelLabel(level: AgentPermissionLevel): string {
  switch (level) {
    case "off":
      return "Disabled";
    case "read":
      return "Read only";
    case "write":
      return "Read & write";
    case "full":
      return "Full access";
  }
}

export function approvalLabel(policy: ApprovalPolicy): string {
  switch (policy) {
    case "always":
      return "Ask for every change";
    case "destructive":
      return "Ask for destructive changes";
    case "session":
      return "Ask once per session";
    case "auto":
      return "Auto-approve in workspace";
  }
}

export function toFsAuthConfig(settings: AgentSettings) {
  return {
    level: settings.enabled ? settings.level : "off",
    allowedRoots: settings.allowedRoots,
    maxReadBytes: settings.maxReadBytes,
    maxSearchResults: settings.maxSearchResults,
    shellEnabled: settings.shellEnabled,
    gitEnabled: settings.gitEnabled,
  };
}
