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

function inferSource(name: string): "mcp" | "ollama" {
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
