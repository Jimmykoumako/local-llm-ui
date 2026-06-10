import type { ToolCallDisplay } from "$lib/types";

export interface SearchFileMatch {
  path: string;
  name: string;
  is_dir: boolean;
}

export interface SearchFilesPayload {
  root?: string;
  pattern?: string;
  count: number;
  matches: SearchFileMatch[];
}

export interface SearchContentHit {
  path: string;
  line: number;
  text: string;
}

export interface SearchContentPayload {
  root?: string;
  query?: string;
  count: number;
  hits: SearchContentHit[];
}

function escapeTableCell(value: string): string {
  return value.replace(/\|/g, "\\|").replace(/\n/g, " ");
}

function fileNameFromPath(path: string): string {
  return path.split("/").pop() || path || "—";
}

export function parseSearchFilesResult(raw: string): SearchFilesPayload | null {
  try {
    const data = JSON.parse(raw) as Record<string, unknown>;
    if (!Array.isArray(data.matches)) return null;

    const matches: SearchFileMatch[] = data.matches
      .filter((m): m is Record<string, unknown> => !!m && typeof m === "object")
      .map((m) => ({
        path: String(m.path ?? ""),
        name: String(m.name ?? ""),
        is_dir: Boolean(m.is_dir),
      }))
      .filter((m) => m.path || m.name);

    return {
      root: typeof data.root === "string" ? data.root : undefined,
      pattern: typeof data.pattern === "string" ? data.pattern : undefined,
      count:
        typeof data.count === "number" ? data.count : matches.length,
      matches,
    };
  } catch {
    return null;
  }
}

export function parseSearchContentResult(raw: string): SearchContentPayload | null {
  try {
    const data = JSON.parse(raw) as Record<string, unknown>;
    if (!Array.isArray(data.hits)) return null;

    const hits: SearchContentHit[] = data.hits
      .filter((h): h is Record<string, unknown> => !!h && typeof h === "object")
      .map((h) => ({
        path: String(h.path ?? ""),
        line: typeof h.line === "number" ? h.line : Number(h.line) || 0,
        text: String(h.text ?? ""),
      }))
      .filter((h) => h.path);

    return {
      root: typeof data.root === "string" ? data.root : undefined,
      query: typeof data.query === "string" ? data.query : undefined,
      count: typeof data.count === "number" ? data.count : hits.length,
      hits,
    };
  } catch {
    return null;
  }
}

export function searchFilesToMarkdown(matches: SearchFileMatch[]): string {
  if (matches.length === 0) {
    return "*No files found.*";
  }

  const lines = [
    "| File | Type | Path |",
    "| --- | --- | --- |",
    ...matches.map(
      (m) =>
        `| ${escapeTableCell(m.name || fileNameFromPath(m.path))} | ${m.is_dir ? "directory" : "file"} | ${escapeTableCell(m.path)} |`,
    ),
  ];
  return lines.join("\n");
}

export function searchContentToMarkdown(hits: SearchContentHit[]): string {
  if (hits.length === 0) {
    return "*No matches found.*";
  }

  const lines = [
    "| File | Line | Content |",
    "| --- | --- | --- |",
    ...hits.map(
      (h) =>
        `| ${escapeTableCell(fileNameFromPath(h.path))} | ${h.line || "—"} | ${escapeTableCell(h.text)} |`,
    ),
  ];
  return lines.join("\n");
}

function isTableLine(line: string): boolean {
  return line.trim().startsWith("|") && line.trim().endsWith("|");
}

function isSeparatorLine(line: string): boolean {
  const cells = line
    .trim()
    .slice(1, -1)
    .split("|")
    .map((c) => c.trim());
  return cells.length > 0 && cells.every((c) => /^:?-{1,}:?$/.test(c) || c === "");
}

function tableHeaderCells(content: string): string[] {
  const line = content.split("\n").find(isTableLine);
  if (!line) return [];
  return line
    .trim()
    .slice(1, -1)
    .split("|")
    .map((c) => c.trim().toLowerCase());
}

export function hasMarkdownTable(content: string): boolean {
  return content.split("\n").some(isTableLine);
}

export function hasPopulatedMarkdownTable(content: string): boolean {
  const lines = content.split("\n");
  let sawHeader = false;
  let sawSeparator = false;

  for (const line of lines) {
    if (!isTableLine(line)) {
      if (sawSeparator) break;
      sawHeader = false;
      sawSeparator = false;
      continue;
    }
    if (!sawHeader) {
      sawHeader = true;
      continue;
    }
    if (isSeparatorLine(line)) {
      sawSeparator = true;
      continue;
    }
    if (sawSeparator && line.trim().length > 2) {
      return true;
    }
  }
  return false;
}

export function isEmptyMarkdownTable(content: string): boolean {
  return hasMarkdownTable(content) && !hasPopulatedMarkdownTable(content);
}

export function fillEmptyMarkdownTable(
  content: string,
  table: string,
): string {
  const lines = content.split("\n");
  let start = -1;
  let end = -1;

  for (let i = 0; i < lines.length; i++) {
    if (!isTableLine(lines[i])) continue;
    if (start < 0) start = i;
    end = i;
    if (isSeparatorLine(lines[i])) {
      for (let j = i + 1; j < lines.length; j++) {
        if (isTableLine(lines[j]) && !isSeparatorLine(lines[j])) {
          end = j;
          i = j;
        } else if (!isTableLine(lines[j])) {
          break;
        }
      }
    }
  }

  if (start < 0) {
    return `${content.trimEnd()}\n\n${table}`;
  }

  const before = lines.slice(0, start);
  const after = lines.slice(end + 1);
  return [...before, table, ...after].join("\n").trimEnd();
}

function collectSearchFileMatches(
  toolCalls: ToolCallDisplay[],
): SearchFileMatch[] {
  const matches: SearchFileMatch[] = [];

  for (const tool of toolCalls) {
    if (tool.name !== "search_files" || tool.status !== "completed" || !tool.result) {
      continue;
    }
    const parsed = parseSearchFilesResult(tool.result);
    if (parsed) matches.push(...parsed.matches);
  }

  return matches;
}

function collectSearchContentHits(
  toolCalls: ToolCallDisplay[],
): SearchContentHit[] {
  const hits: SearchContentHit[] = [];

  for (const tool of toolCalls) {
    if (tool.name !== "search_content" || tool.status !== "completed" || !tool.result) {
      continue;
    }
    const parsed = parseSearchContentResult(tool.result);
    if (parsed) hits.push(...parsed.hits);
  }

  return hits;
}

function prefersContentTable(headers: string[]): boolean {
  return headers.some((h) => h === "line" || h === "content" || h === "text");
}

function pickTableMarkdown(
  content: string,
  fileMatches: SearchFileMatch[],
  contentHits: SearchContentHit[],
): string | null {
  if (contentHits.length > 0 && fileMatches.length === 0) {
    return searchContentToMarkdown(contentHits);
  }
  if (fileMatches.length > 0 && contentHits.length === 0) {
    return searchFilesToMarkdown(fileMatches);
  }

  if (contentHits.length === 0 && fileMatches.length === 0) {
    return null;
  }

  const headers = tableHeaderCells(content);
  if (prefersContentTable(headers)) {
    return searchContentToMarkdown(contentHits);
  }
  return searchFilesToMarkdown(fileMatches);
}

function looksLikeSearchResponse(content: string): boolean {
  if (hasMarkdownTable(content)) return false;
  return /\b(files?|found|match(?:es)?|hit(?:s)?|search(?:ed)?|results?)\b/i.test(
    content,
  );
}

/** Fill or append a markdown table from search_files / search_content tool results. */
export function enrichAssistantContentWithSearchFiles(
  content: string,
  toolCalls: ToolCallDisplay[],
): string {
  const trimmed = content.trim();
  if (!trimmed) return content;

  const fileMatches = collectSearchFileMatches(toolCalls);
  const contentHits = collectSearchContentHits(toolCalls);
  if (fileMatches.length === 0 && contentHits.length === 0) return content;
  if (hasPopulatedMarkdownTable(content)) return content;

  const table = pickTableMarkdown(content, fileMatches, contentHits);
  if (!table) return content;

  if (isEmptyMarkdownTable(content)) {
    return fillEmptyMarkdownTable(content, table);
  }

  if (looksLikeSearchResponse(content)) {
    return `${content.trimEnd()}\n\n${table}`;
  }

  return content;
}
