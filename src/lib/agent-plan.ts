import type { AgentSettings } from "./agent-settings";
import { levelLabel } from "./agent-settings";

export type PlanStepKind = "read" | "write";
export type PlanStepStatus = "pending" | "running" | "done" | "failed" | "skipped";
export type AgentPlanStatus = "ready" | "executing" | "completed" | "failed";

export interface PlanStep {
  id: string;
  index: number;
  description: string;
  kind: PlanStepKind;
  status: PlanStepStatus;
  tool?: string;
  path?: string;
  args?: Record<string, unknown>;
}

export interface AgentPlan {
  id: string;
  title: string;
  steps: PlanStep[];
  status: AgentPlanStatus;
  sourceMessage: string;
  createdAt: number;
}

const approvedPlanKeys = new Set<string>();

export function planApprovalKey(convId: string, planId: string): string {
  return `${convId}:${planId}`;
}

export function approvePlanExecution(convId: string, planId: string): void {
  approvedPlanKeys.add(planApprovalKey(convId, planId));
}

export function isPlanExecutionApproved(convId: string, planId: string): boolean {
  return approvedPlanKeys.has(planApprovalKey(convId, planId));
}

export function clearPlanApproval(convId: string, planId: string): void {
  approvedPlanKeys.delete(planApprovalKey(convId, planId));
}

export const PLAN_JSON_EXAMPLE = `{
  "steps": [
    {
      "index": 1,
      "kind": "read",
      "description": "List files in workspace",
      "tool": "list_directory",
      "path": "/path/to/workspace"
    },
    {
      "index": 2,
      "kind": "write",
      "description": "Create notes file",
      "tool": "create_file",
      "path": "/path/to/workspace/notes.md",
      "args": { "content": "# Notes\\n" }
    }
  ]
}`;

export function buildPlanningUserMessage(userText: string, roots: string[]): string {
  const rootsHint =
    roots.length > 0
      ? `Workspace roots: ${roots.join(", ")}`
      : "No workspace configured yet — use paths the user is likely to add.";

  return (
    "Create a step-by-step implementation plan for the request below. " +
    "You may use read-only tools to explore the workspace first. " +
    "Do NOT create, modify, or delete anything during planning. " +
    "Do not discuss permissions or authorization.\n\n" +
    `${rootsHint}\n\n` +
    "Output the final plan ONLY as JSON inside a fenced code block tagged `plan`. " +
    "Each step must include: index, kind (read|write), description, tool (agent tool name), " +
    "path (primary file/dir path), and args (tool arguments object when needed).\n" +
    "```plan\n" +
    PLAN_JSON_EXAMPLE +
    "\n```\n\n" +
    `Request:\n${userText}`
  );
}

export function buildPlanningSystemPrompt(settings: AgentSettings): string {
  const roots =
    settings.allowedRoots.length > 0
      ? settings.allowedRoots.join(", ")
      : "none configured";

  return [
    "You are in PLANNING mode.",
    "Explore with read-only tools when helpful. Never write, create, delete, or modify files.",
    "Do not mention permission levels or ask the user to change settings.",
    `Workspace roots: ${roots}`,
    "Output the plan as JSON in a ```plan block. Prefer concrete paths under workspace roots.",
    "Valid tools: list_directory, read_file, search_files, search_content, stat_file, " +
      "write_file, create_file, create_directory, move_path, copy_path, delete_path, git_*.",
  ].join("\n");
}

export const PLAN_COMPLETE_MARKER = "PLAN_COMPLETE";

export function isPlanCompleteMarker(content: string): boolean {
  return /\bPLAN_COMPLETE\b/i.test(content);
}

export function buildSingleStepExecutionPrompt(
  settings: AgentSettings,
  plan: AgentPlan,
  stepIndex: number,
): string {
  const step = plan.steps[stepIndex];
  const toolHint = step.tool
    ? `Preferred tool: ${step.tool}`
    : "Pick the best agent tool for this step.";
  const pathHint = step.path ? `Primary path: ${step.path}` : "";
  const argsHint =
    step.args && Object.keys(step.args).length > 0
      ? `Suggested arguments: ${JSON.stringify(step.args)}`
      : "";

  return [
    "You are executing ONE step of an approved plan.",
    `Step ${step.index} of ${plan.steps.length} — [${step.kind}] ${step.description}`,
    toolHint,
    pathHint,
    argsHint,
    `Permission level: ${levelLabel(settings.level)} (${settings.level})`,
    "Complete only this step using tools, then summarize what you did.",
    "Do not start later steps.",
    `When this step is finished, end with ${PLAN_COMPLETE_MARKER} on its own line.`,
  ]
    .filter(Boolean)
    .join("\n");
}

export function buildSingleStepUserMessage(step: PlanStep): string {
  const parts = [
    `Execute plan step ${step.index} only:`,
    `[${step.kind}] ${step.description}`,
  ];
  if (step.tool) parts.push(`Tool: ${step.tool}`);
  if (step.path) parts.push(`Path: ${step.path}`);
  if (step.args && Object.keys(step.args).length > 0) {
    parts.push(`Args: ${JSON.stringify(step.args)}`);
  }
  return parts.join("\n");
}

export function buildExecutionSystemPrompt(
  settings: AgentSettings,
  plan: AgentPlan,
): string {
  const stepList = plan.steps
    .map((s) => {
      const meta = [s.tool, s.path].filter(Boolean).join(" · ");
      return `${s.index}. [${s.kind}] ${s.description}${meta ? ` (${meta})` : ""}`;
    })
    .join("\n");

  return [
    "You are EXECUTING an approved plan. Follow the steps in order.",
    `Permission level: ${levelLabel(settings.level)} (${settings.level})`,
    "",
    "## Approved plan",
    stepList,
  ].join("\n");
}

export function planContinueNudge(): string {
  return (
    "You have not finished the plan yet. Continue with the next pending step using tools. " +
    `Do not stop until every step is done, then say ${PLAN_COMPLETE_MARKER}.`
  );
}

export function allPlanStepsDone(plan: AgentPlan): boolean {
  return plan.steps.length > 0 && plan.steps.every((s) => s.status === "done");
}

export function reindexPlanSteps(steps: PlanStep[]): PlanStep[] {
  return steps.map((s, i) => ({ ...s, index: i + 1, id: s.id || `step-${i + 1}` }));
}

export function createEmptyPlanStep(index: number): PlanStep {
  return {
    id: `step-${crypto.randomUUID().slice(0, 8)}`,
    index,
    description: "New step",
    kind: "read",
    status: "pending",
    tool: "list_directory",
    path: "",
  };
}

const WRITE_HINT =
  /\b(create|write|edit|modify|delete|remove|move|copy|mkdir|overwrite|git commit|git push|git add|run terminal|shell)\b/i;

function inferStepKind(
  description: string,
  explicit?: string,
): PlanStepKind {
  if (explicit === "read" || explicit === "write") return explicit;
  return WRITE_HINT.test(description) ? "write" : "read";
}

function normalizeStep(raw: Record<string, unknown>, fallbackIndex: number): PlanStep | null {
  const description = String(raw.description ?? "").trim();
  if (!description) return null;

  const index =
    typeof raw.index === "number" ? raw.index : Number(raw.index) || fallbackIndex;
  const kind = inferStepKind(
    description,
    typeof raw.kind === "string" ? raw.kind.toLowerCase() : undefined,
  );

  const args =
    raw.args && typeof raw.args === "object" && !Array.isArray(raw.args)
      ? (raw.args as Record<string, unknown>)
      : undefined;

  const content =
    typeof raw.content === "string" ? raw.content : undefined;
  const mergedArgs =
    content !== undefined
      ? { ...(args ?? {}), content }
      : args;

  return {
    id: `step-${index}`,
    index,
    description,
    kind,
    status: "pending",
    tool: typeof raw.tool === "string" ? raw.tool : undefined,
    path:
      typeof raw.path === "string"
        ? raw.path
        : typeof raw.file === "string"
          ? raw.file
          : undefined,
    args: mergedArgs,
  };
}

function parsePlanJson(body: string): PlanStep[] {
  try {
    const data = JSON.parse(body) as { steps?: unknown[] } | unknown[];
    const list = Array.isArray(data) ? data : data.steps;
    if (!Array.isArray(list)) return [];

    const steps: PlanStep[] = [];
    for (let i = 0; i < list.length; i++) {
      const item = list[i];
      if (!item || typeof item !== "object") continue;
      const step = normalizeStep(item as Record<string, unknown>, i + 1);
      if (step) steps.push(step);
    }
    return reindexPlanSteps(steps);
  } catch {
    return [];
  }
}

function parsePlanBodyMarkdown(body: string): PlanStep[] {
  const steps: PlanStep[] = [];

  for (const line of body.split("\n")) {
    const match = line.match(
      /^\s*(\d+)[.)]\s*(?:\[(read|write)\]\s*)?(.+?)\s*$/i,
    );
    if (!match) continue;

    const index = Number(match[1]);
    const explicit = match[2]?.toLowerCase();
    const description = match[3].trim();
    if (!description) continue;

    steps.push({
      id: `step-${index}`,
      index,
      description,
      kind: inferStepKind(description, explicit),
      status: "pending",
    });
  }

  return steps;
}

export function parsePlanFromContent(
  content: string,
  sourceMessage: string,
): AgentPlan | null {
  const blockMatch = content.match(/```plan\s*([\s\S]*?)```/i);
  const body = blockMatch?.[1]?.trim() ?? content.trim();

  let steps = parsePlanJson(body);
  if (steps.length === 0) {
    steps = parsePlanBodyMarkdown(body);
  }
  if (steps.length === 0) return null;

  const title =
    sourceMessage.length > 60
      ? `${sourceMessage.slice(0, 60)}…`
      : sourceMessage;

  return {
    id: crypto.randomUUID(),
    title,
    steps,
    status: "ready",
    sourceMessage,
    createdAt: Date.now(),
  };
}

export function planWriteSteps(plan: AgentPlan): PlanStep[] {
  return plan.steps.filter((s) => s.kind === "write");
}

export function planReadySummary(): string {
  return "I've prepared a plan for your request. Review and edit steps in the **Plan** panel, then click **Run plan** when ready.";
}

const AUTO_PLAN_RE =
  /\b(create|write|edit|modify|delete|remove|add|implement|build|refactor|rename|move|copy|mkdir|scaffold|fix|update|generate)\b.{0,40}\b(file|files|folder|directory|project|code|script|module|component|function|class|repo|repository)\b/i;

const AUTO_PLAN_VERB_RE =
  /\b(create|write|edit|modify|delete|remove|implement|build|refactor|scaffold)\s+(a |an |the |my |this )?/i;

export function shouldAutoPlan(message: string): boolean {
  const text = message.trim();
  if (text.length < 8) return false;
  return AUTO_PLAN_RE.test(text) || AUTO_PLAN_VERB_RE.test(text);
}

export function autoPlanNotice(): string {
  return "I'll build a plan first (read-only), then you can approve and run it.";
}

export function buildExecutionUserMessage(plan: AgentPlan): string {
  return `Execute the approved plan step by step:\n\n${plan.steps
    .map((s) => {
      const meta = [s.tool, s.path].filter(Boolean).join(" · ");
      return `${s.index}. [${s.kind}] ${s.description}${meta ? ` (${meta})` : ""}`;
    })
    .join("\n")}`;
}

export function stepToolLabel(step: PlanStep): string {
  const parts = [step.tool, step.path].filter(Boolean);
  return parts.length > 0 ? parts.join(" → ") : "—";
}
