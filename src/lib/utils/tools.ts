import type { ToolCallDisplay } from "$lib/types";

function parseArguments(raw: unknown): Record<string, unknown> | string {
  if (typeof raw === "string") {
    try {
      return JSON.parse(raw) as Record<string, unknown>;
    } catch {
      return raw;
    }
  }
  if (raw && typeof raw === "object") {
    return raw as Record<string, unknown>;
  }
  return {};
}

const AGENT_TOOL_NAMES = new Set([
  "read_file", "write_file", "create_file", "create_directory",
  "list_directory", "delete_path", "move_path", "copy_path", "stat_file",
  "search_files", "search_content", "run_terminal", "get_agent_capabilities",
  "git_status", "git_diff", "git_log", "git_branch_list",
  "git_add", "git_commit", "git_checkout", "git_pull", "git_push",
]);

function inferSource(name: string): "mcp" | "ollama" | "agent" {
  if (AGENT_TOOL_NAMES.has(name)) return "agent";
  const lower = name.toLowerCase();
  if (
    lower.startsWith("mcp_") ||
    lower.includes("mcp") ||
    lower.startsWith("server_")
  ) {
    return "mcp";
  }
  return "ollama";
}

/** Normalize Ollama / OpenAI-style tool_calls into display objects. */
export function parseToolCalls(raw: unknown[]): ToolCallDisplay[] {
  const results: ToolCallDisplay[] = [];

  for (const item of raw) {
    if (!item || typeof item !== "object") continue;
    const obj = item as Record<string, unknown>;

    const fn = obj.function as Record<string, unknown> | undefined;
    if (fn?.name) {
      const name = String(fn.name);
      results.push({
        id: obj.id ? String(obj.id) : undefined,
        name,
        arguments: parseArguments(fn.arguments),
        status: "requested",
        source: inferSource(name),
      });
      continue;
    }

    const name = String(obj.name ?? obj.tool_name ?? "unknown_tool");
    results.push({
      id: obj.id ? String(obj.id) : undefined,
      name,
      arguments: parseArguments(obj.arguments ?? obj.input),
      status: "requested",
      source: inferSource(name),
    });
  }

  return results;
}

export function formatToolArguments(
  args: Record<string, unknown> | string,
): string {
  if (typeof args === "string") return args;
  return JSON.stringify(args, null, 2);
}

/** Ollama expects tool call arguments as objects, not JSON strings. */
export function normalizeToolArguments(
  args: Record<string, unknown> | string,
): Record<string, unknown> {
  if (args && typeof args === "object" && !Array.isArray(args)) {
    return args;
  }
  if (typeof args === "string") {
    try {
      const parsed: unknown = JSON.parse(args);
      if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
        return parsed as Record<string, unknown>;
      }
    } catch {
      /* use empty object below */
    }
  }
  return {};
}

/** Format tool calls for Ollama /api/chat follow-up requests. */
export function toolCallsForOllamaHistory(
  tools: ToolCallDisplay[],
): Record<string, unknown>[] {
  return tools.map((tool, index) => ({
    type: "function",
    ...(tool.id ? { id: tool.id } : {}),
    function: {
      index,
      name: tool.name,
      arguments: normalizeToolArguments(tool.arguments),
    },
  }));
}
