import type { ToolCallDisplay } from "./types";

const STORAGE_KEY = "local-llm-ui-remembered-approvals";

function argsObject(
  args: Record<string, unknown> | string,
): Record<string, unknown> {
  if (typeof args === "string") {
    try {
      return JSON.parse(args) as Record<string, unknown>;
    } catch {
      return {};
    }
  }
  return args;
}

/** Stable per-file / per-repo key for remembered approvals. */
export function persistentApprovalKey(tool: ToolCallDisplay): string | null {
  const args = argsObject(tool.arguments);

  const path = args.path;
  if (
    typeof path === "string" &&
    (tool.name === "write_file" ||
      tool.name === "create_file" ||
      tool.name === "delete_path" ||
      tool.name === "create_directory")
  ) {
    return `${tool.name}:${path}`;
  }

  if (tool.name === "move_path") {
    const from = args.from;
    const to = args.to;
    if (typeof from === "string" && typeof to === "string") {
      return `move_path:${from}->${to}`;
    }
  }

  if (tool.name === "copy_path") {
    const from = args.from;
    const to = args.to;
    if (typeof from === "string" && typeof to === "string") {
      return `copy_path:${from}->${to}`;
    }
  }

  if (tool.name.startsWith("git_")) {
    const cwd = args.cwd ?? args.root ?? ".";
    if (typeof cwd === "string") return `${tool.name}:${cwd}`;
  }

  if (tool.name === "run_terminal") {
    const cwd = args.cwd ?? ".";
    const command = args.command;
    if (typeof cwd === "string" && typeof command === "string") {
      return `run_terminal:${cwd}:${command}`;
    }
  }

  return null;
}

export function loadRememberedApprovals(): string[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as unknown;
    return Array.isArray(parsed)
      ? parsed.filter((k): k is string => typeof k === "string")
      : [];
  } catch {
    return [];
  }
}

let rememberedCache: Set<string> | null = null;

function getRememberedSet(): Set<string> {
  if (!rememberedCache) {
    rememberedCache = new Set(loadRememberedApprovals());
  }
  return rememberedCache;
}

export function isRemembered(tool: ToolCallDisplay): boolean {
  const key = persistentApprovalKey(tool);
  if (!key) return false;
  return getRememberedSet().has(key);
}

export function rememberApproval(tool: ToolCallDisplay): void {
  const key = persistentApprovalKey(tool);
  if (!key) return;
  const set = getRememberedSet();
  set.add(key);
  localStorage.setItem(STORAGE_KEY, JSON.stringify([...set]));
}

export function forgetApproval(key: string): void {
  const set = getRememberedSet();
  set.delete(key);
  localStorage.setItem(STORAGE_KEY, JSON.stringify([...set]));
}

export function clearRememberedApprovals(): void {
  rememberedCache = new Set();
  localStorage.removeItem(STORAGE_KEY);
}

export function listRememberedApprovals(): string[] {
  return [...getRememberedSet()];
}
