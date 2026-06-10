import { invoke } from "@tauri-apps/api/core";
import { isRemembered } from "./agent-approvals";
import type { AgentSettings, ApprovalPolicy } from "./agent-settings";
import { levelLabel, toFsAuthConfig } from "./agent-settings";
import type { ToolCallDisplay } from "./types";
import { formatToolArguments } from "./utils/tools";

const FILE_TOOLS = new Set([
  "read_file",
  "write_file",
  "create_file",
  "create_directory",
  "list_directory",
  "delete_path",
  "move_path",
  "copy_path",
  "stat_file",
  "search_files",
  "search_content",
]);

const GIT_READ_TOOLS = new Set([
  "git_status",
  "git_diff",
  "git_log",
  "git_branch_list",
]);

const GIT_WRITE_TOOLS = new Set([
  "git_add",
  "git_commit",
  "git_checkout",
  "git_pull",
  "git_push",
]);

const SHELL_TOOLS = new Set(["run_terminal"]);

export const AGENT_TOOL_NAMES = new Set([
  ...FILE_TOOLS,
  ...GIT_READ_TOOLS,
  ...GIT_WRITE_TOOLS,
  ...SHELL_TOOLS,
]);

const WRITE_TOOLS = new Set([
  "write_file",
  "create_file",
  "create_directory",
  "move_path",
  "copy_path",
  "delete_path",
  "run_terminal",
  "git_add",
  "git_commit",
  "git_checkout",
  "git_pull",
  "git_push",
]);

const DESTRUCTIVE_TOOLS = new Set([
  "delete_path",
  "write_file",
  "move_path",
  "run_terminal",
  "git_commit",
  "git_checkout",
  "git_pull",
  "git_push",
]);

const ALWAYS_APPROVE_TOOLS = new Set(["run_terminal"]);

const CAPABILITIES_TOOL = "get_agent_capabilities";

export const AGENT_EFFICIENCY_PROMPT =
  "You have filesystem, git, and terminal tools. Be efficient: use the minimum number of tool calls. " +
  "For directory listings, use a single list_directory call with recursive:true — do not call it multiple times. " +
  "Never repeat the same tool with identical arguments. Answer from tool results when possible.";

export function getAllowedToolNames(settings: AgentSettings): string[] {
  return toolsForLevel(settings)
    .map((t) => (t as { function?: { name?: string } }).function?.name)
    .filter((n): n is string => Boolean(n));
}

export function buildAgentSystemPrompt(settings: AgentSettings): string {
  const allowed = getAllowedToolNames(settings);
  const roots =
    settings.allowedRoots.length > 0
      ? settings.allowedRoots.join(", ")
      : "none configured";

  const lines = [
    AGENT_EFFICIENCY_PROMPT,
    "",
    "## Authorization (always respect this)",
    `Permission level: ${levelLabel(settings.level)} (${settings.level})`,
    `Allowed tools: ${allowed.join(", ")}`,
    `Workspace roots: ${roots}`,
    `Approval policy: ${settings.approvalPolicy}`,
  ];

  if (settings.level === "read") {
    lines.push(
      "",
      "READ-ONLY MODE: You cannot create, modify, delete, move, or copy files. Shell and git writes are also unavailable.",
      "If the user asks to create, write, edit, delete, or modify anything:",
      "1. Do NOT call read/search tools repeatedly trying to work around this.",
      "2. Optionally call get_agent_capabilities once to confirm limits.",
      "3. Tell the user write access must be enabled in Settings → Agent (Read & write or Full access).",
    );
  } else if (settings.level === "write") {
    lines.push(
      "",
      "Write mode: create/edit/copy/move files is allowed. delete_path, shell, and most git writes require Full access.",
    );
  }

  lines.push(
    "",
    "For multi-step create/edit/delete requests, a plan will be built automatically (read-only first). The user approves once before execution.",
    "If any tool returns a permission or authorization error, STOP immediately — do not retry the same operation or call other tools hoping to bypass it. Explain what permission level is required.",
    "When unsure whether an operation is allowed, call get_agent_capabilities first instead of guessing.",
  );

  return lines.join("\n");
}

export const VISION_AGENT_SUPPLEMENT =
  "## Vision (attached images)\n" +
  "The user's message includes image(s) that you can see directly.\n" +
  "Describe, analyze, or answer questions about those images in your text reply.\n" +
  "Do NOT use filesystem, git, or agent tools for image understanding — the pixels are already in the message.\n" +
  "You cannot generate new images; only analyze what was attached.";

export function isToolAllowedAtLevel(
  name: string,
  settings: AgentSettings,
): boolean {
  if (name === CAPABILITIES_TOOL) return settings.enabled && settings.level !== "off";
  return getAllowedToolNames(settings).includes(name);
}

const PERMISSION_DENIED_RE =
  /not allowed at permission level|not available at permission level|agent file access is disabled|do not retry this operation/i;

export function isPermissionDeniedResult(result: string | undefined): boolean {
  return Boolean(result && PERMISSION_DENIED_RE.test(result));
}

function getAgentCapabilities(settings: AgentSettings): string {
  return JSON.stringify(
    {
      permission_level: settings.level,
      permission_label: levelLabel(settings.level),
      write_allowed: settings.level === "write" || settings.level === "full",
      delete_allowed: settings.level === "full",
      shell_allowed: settings.level === "full" && settings.shellEnabled,
      git_write_allowed:
        (settings.level === "write" || settings.level === "full") &&
        settings.gitEnabled,
      allowed_tools: getAllowedToolNames(settings),
      workspace_roots: settings.allowedRoots,
      approval_policy: settings.approvalPolicy,
    },
    null,
    2,
  );
}

export function isAgentTool(name: string): boolean {
  return AGENT_TOOL_NAMES.has(name) || name === CAPABILITIES_TOOL;
}

export function canRememberApproval(tool: ToolCallDisplay): boolean {
  return (
    WRITE_TOOLS.has(tool.name) &&
    tool.name !== "run_terminal" &&
    !ALWAYS_APPROVE_TOOLS.has(tool.name)
  );
}

export function agentToolDefinitions(settings: AgentSettings): unknown[] {
  const cwdParam = {
    cwd: {
      type: "string",
      description: "Repository or working directory (optional)",
    },
  };

  const tools: unknown[] = [
    toolDef(
      CAPABILITIES_TOOL,
      "Return current permission level, allowed tools, and workspace roots. Call before write/create/delete when authorization is unclear.",
      {},
      [],
    ),
    toolDef("read_file", "Read a text file within the workspace.", {
      path: { type: "string", description: "Absolute or relative file path" },
      offset: { type: "integer", description: "1-based line offset (optional)" },
      limit: { type: "integer", description: "Max lines to read (optional)" },
    }, ["path"]),
    toolDef("write_file", "Write or overwrite a text file.", {
      path: { type: "string" },
      content: { type: "string" },
    }, ["path", "content"]),
    toolDef("create_file", "Create a new file (fails if it exists).", {
      path: { type: "string" },
      content: { type: "string" },
    }, ["path"]),
    toolDef("create_directory", "Create a directory (and parents).", {
      path: { type: "string" },
    }, ["path"]),
    toolDef(
      "list_directory",
      "List files and folders. Use recursive:true once to get the full tree — avoid calling repeatedly.",
      {
        path: { type: "string" },
        recursive: {
          type: "boolean",
          description: "Set true to list all nested files in one call",
        },
      },
      ["path"],
    ),
    toolDef("delete_path", "Delete a file or directory.", {
      path: { type: "string" },
    }, ["path"]),
    toolDef("move_path", "Move or rename a file or directory.", {
      from: { type: "string" },
      to: { type: "string" },
    }, ["from", "to"]),
    toolDef("copy_path", "Copy a file or directory.", {
      from: { type: "string" },
      to: { type: "string" },
    }, ["from", "to"]),
    toolDef("stat_file", "Get file or directory metadata.", {
      path: { type: "string" },
    }, ["path"]),
    toolDef("search_files", "Find files by name pattern under a root.", {
      root: { type: "string", description: "Directory to search" },
      pattern: { type: "string", description: "Filename substring or glob" },
    }, ["root", "pattern"]),
    toolDef("search_content", "Search text file contents for a query.", {
      root: { type: "string" },
      query: { type: "string" },
      glob: { type: "string", description: "Optional filename filter e.g. *.ts" },
    }, ["root", "query"]),
  ];

  if (settings.gitEnabled) {
    tools.push(
      toolDef("git_status", "Show git working tree status.", cwdParam, []),
      toolDef("git_diff", "Show git diff.", {
        ...cwdParam,
        staged: { type: "boolean" },
        path: { type: "string", description: "Optional file path" },
      }, []),
      toolDef("git_log", "Show recent git commits.", {
        ...cwdParam,
        limit: { type: "integer" },
      }, []),
      toolDef("git_branch_list", "List git branches.", cwdParam, []),
      toolDef("git_add", "Stage files for commit.", {
        ...cwdParam,
        paths: { type: "array", items: { type: "string" } },
      }, ["paths"]),
      toolDef("git_commit", "Create a git commit.", {
        ...cwdParam,
        message: { type: "string" },
      }, ["message"]),
      toolDef("git_checkout", "Switch git branch.", {
        ...cwdParam,
        branch: { type: "string" },
      }, ["branch"]),
      toolDef("git_pull", "Pull from remote.", {
        ...cwdParam,
        remote: { type: "string" },
        branch: { type: "string" },
      }, []),
      toolDef("git_push", "Push to remote.", {
        ...cwdParam,
        remote: { type: "string" },
        branch: { type: "string" },
      }, []),
    );
  }

  if (settings.shellEnabled) {
    tools.push(
      toolDef(
        "run_terminal",
        "Run a shell command in the workspace (requires approval).",
        {
          command: { type: "string" },
          cwd: { type: "string", description: "Working directory" },
          timeout_secs: { type: "integer", description: "Max 120 seconds" },
        },
        ["command"],
      ),
    );
  }

  return tools;
}

function toolDef(
  name: string,
  description: string,
  properties: Record<string, unknown>,
  required: string[],
) {
  return {
    type: "function",
    function: {
      name,
      description,
      parameters: {
        type: "object",
        properties,
        required,
      },
    },
  };
}

function approvalKey(tool: ToolCallDisplay): string {
  return `${tool.name}:${formatToolArguments(tool.arguments)}`;
}

const sessionApprovals = new Set<string>();

export function clearSessionApprovals(): void {
  sessionApprovals.clear();
}

export function needsApproval(
  tool: ToolCallDisplay,
  policy: ApprovalPolicy,
): boolean {
  if (!WRITE_TOOLS.has(tool.name)) return false;
  if (isRemembered(tool)) return false;
  if (ALWAYS_APPROVE_TOOLS.has(tool.name)) return true;

  switch (policy) {
    case "auto":
      return false;
    case "always":
      return true;
    case "destructive":
      return DESTRUCTIVE_TOOLS.has(tool.name);
    case "session":
      return !sessionApprovals.has(approvalKey(tool));
    default:
      return true;
  }
}

export function recordSessionApproval(tool: ToolCallDisplay): void {
  sessionApprovals.add(approvalKey(tool));
}

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

export async function executeAgentTool(
  tool: ToolCallDisplay,
  settings: AgentSettings,
): Promise<string> {
  if (tool.name === CAPABILITIES_TOOL) {
    return getAgentCapabilities(settings);
  }

  if (!isToolAllowedAtLevel(tool.name, settings)) {
    const allowed = getAllowedToolNames(settings).join(", ");
    return (
      `Operation '${tool.name}' is not available at permission level '${settings.level}' (${levelLabel(settings.level)}). ` +
      `Allowed tools: ${allowed}. ` +
      "Do not retry this operation — tell the user to raise the permission level in Settings → Agent if needed."
    );
  }

  const args = argsObject(tool.arguments);
  return invoke<string>("agent_execute_tool", {
    name: tool.name,
    args,
    config: toFsAuthConfig(settings),
  });
}

/** Read-only tools for the planning phase (no writes regardless of settings level). */
export function toolsForPlanning(settings: AgentSettings): unknown[] {
  if (!settings.enabled) return [];
  return toolsForLevel({ ...settings, level: "read" });
}

export function toolsForLevel(settings: AgentSettings): unknown[] {
  if (!settings.enabled || settings.level === "off") return [];

  const all = agentToolDefinitions(settings);
  if (settings.level === "full") return all;

  const allowed = new Set<string>([CAPABILITIES_TOOL]);

  if (settings.level === "read") {
    [
      "read_file",
      "list_directory",
      "search_files",
      "search_content",
      "stat_file",
    ].forEach((n) => allowed.add(n));
    if (settings.gitEnabled) {
      GIT_READ_TOOLS.forEach((n) => allowed.add(n));
    }
  } else if (settings.level === "write") {
    [
      "read_file",
      "list_directory",
      "search_files",
      "search_content",
      "stat_file",
      "write_file",
      "create_file",
      "create_directory",
      "move_path",
      "copy_path",
    ].forEach((n) => allowed.add(n));
    if (settings.gitEnabled) {
      GIT_READ_TOOLS.forEach((n) => allowed.add(n));
      allowed.add("git_add");
    }
  }

  return all.filter((t) => {
    const fn = (t as { function?: { name?: string } }).function;
    return fn?.name && allowed.has(fn.name);
  });
}
