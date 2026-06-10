import type { AgentPlan } from "./agent-plan";
import type { ChatMessage, DisplayMessage } from "./types";

export type AppView = "chat" | "tagged" | "models" | "settings";
export type ChatMode = "ask" | "plan" | "debug" | "multi";

export interface SavedConversation {
  id: string;
  title: string;
  model: string;
  thinkEnabled: boolean;
  chatMode: ChatMode;
  createdAt: number;
  updatedAt: number;
  messages: DisplayMessage[];
  history: ChatMessage[];
  agentPlan?: AgentPlan | null;
}

const STORAGE_KEY = "local-llm-ui-conversations";

export function loadConversations(): SavedConversation[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as SavedConversation[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function saveConversations(conversations: SavedConversation[]): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(conversations));
}

export function deriveTitle(messages: DisplayMessage[]): string {
  const first = messages.find((m) => m.role === "user" && m.content.trim());
  if (!first) return "New chat";
  const text = first.content.trim();
  return text.length > 42 ? `${text.slice(0, 42)}…` : text;
}

export function applyModePrefix(text: string, mode: ChatMode): string {
  const trimmed = text.trim();
  if (!trimmed) return trimmed;
  switch (mode) {
    case "plan":
      return trimmed;
    case "debug":
      return `Help me debug the following issue:\n\n${trimmed}`;
    case "multi":
      return `Break this into subtasks and address each one:\n\n${trimmed}`;
    default:
      return trimmed;
  }
}

export function modeLabel(mode: ChatMode): string {
  switch (mode) {
    case "plan":
      return "Plan";
    case "debug":
      return "Debug";
    case "multi":
      return "Multi-task";
    default:
      return "Ask";
  }
}
