import type { ToolCallDisplay } from "$lib/types";

function toolArgs(tool: ToolCallDisplay): Record<string, unknown> {
  if (typeof tool.arguments === "string") {
    try {
      return JSON.parse(tool.arguments) as Record<string, unknown>;
    } catch {
      return {};
    }
  }
  return tool.arguments ?? {};
}

function basename(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/").filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

function readRange(args: Record<string, unknown>): string | undefined {
  const offset = Number(args.offset ?? args.start_line ?? 1);
  const limit = args.limit ?? args.end_line;
  if (limit !== undefined && limit !== null) {
    const end = Number(limit);
    if (!Number.isNaN(offset) && !Number.isNaN(end)) {
      return end >= offset ? `L${offset}–${end}` : `L${offset}`;
    }
  }
  if (!Number.isNaN(offset) && offset > 1) return `L${offset}`;
  return undefined;
}

function writeDetail(tool: ToolCallDisplay, args: Record<string, unknown>): string | undefined {
  const content = args.content;
  if (typeof content === "string" && content.length > 0) {
    const lines = content.split("\n").length;
    return `+${lines}`;
  }
  if (tool.result) {
    try {
      const parsed = JSON.parse(tool.result) as Record<string, unknown>;
      if (parsed.created === true) {
        const bytes = Number(parsed.bytes_written ?? 0);
        return bytes > 0 ? `+${bytes}b` : "new";
      }
      if (parsed.bytes_written !== undefined) {
        return `${parsed.bytes_written}b`;
      }
    } catch {
      // ignore
    }
  }
  return undefined;
}

export interface ToolActivityLine {
  verb: string;
  target: string;
  detail?: string;
  status?: ToolCallDisplay["status"];
}

export function formatToolActivity(tool: ToolCallDisplay): ToolActivityLine {
  const args = toolArgs(tool);
  const path = String(args.path ?? args.file_path ?? args.directory ?? "");
  const name = path ? basename(path) : tool.name;

  switch (tool.name) {
    case "read_file":
      return { verb: "Read", target: name, detail: readRange(args), status: tool.status };
    case "write_file":
      return {
        verb: "Edited",
        target: name,
        detail: writeDetail(tool, args),
        status: tool.status,
      };
    case "create_file":
      return {
        verb: "Created",
        target: name,
        detail: writeDetail(tool, args),
        status: tool.status,
      };
    case "create_directory":
      return { verb: "Created folder", target: name, status: tool.status };
    case "delete_path":
      return { verb: "Deleted", target: name, status: tool.status };
    case "move_path":
      return { verb: "Moved", target: name, status: tool.status };
    case "copy_path":
      return { verb: "Copied", target: name, status: tool.status };
    case "list_directory":
      return { verb: "Listed", target: name || "directory", status: tool.status };
    case "search_files":
      return { verb: "Searched files", target: String(args.pattern ?? "…"), status: tool.status };
    case "search_content":
      return { verb: "Searched content", target: String(args.pattern ?? "…"), status: tool.status };
    case "stat_file":
      return { verb: "Stat", target: name, status: tool.status };
    case "run_terminal":
      return {
        verb: "Ran",
        target: String(args.command ?? "command").slice(0, 48),
        status: tool.status,
      };
    case "get_agent_capabilities":
      return { verb: "Checked", target: "capabilities", status: tool.status };
    default:
      if (tool.name.startsWith("git_")) {
        return { verb: tool.name.replace("git_", "Git "), target: name || "", status: tool.status };
      }
      return { verb: tool.name, target: name, status: tool.status };
  }
}

export function formatThoughtDuration(ms: number | undefined): string {
  if (!ms || ms < 1000) return "Thought";
  const secs = Math.max(1, Math.round(ms / 1000));
  return `Thought for ${secs}s`;
}

export function thinkingSummary(thinking: string, maxLen = 120): string {
  const line = thinking
    .split("\n")
    .map((l) => l.trim())
    .find(Boolean);
  if (!line) return "";
  return line.length > maxLen ? `${line.slice(0, maxLen)}…` : line;
}
