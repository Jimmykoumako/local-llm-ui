import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { pushToast } from "$lib/toast";

export function sanitizeDownloadName(text: string, fallback = "message"): string {
  const slug = text
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "")
    .slice(0, 48);
  return slug || fallback;
}

export function markdownFilename(label: string, index?: number): string {
  const base = sanitizeDownloadName(label, index !== undefined ? `message-${index + 1}` : "message");
  return base.endsWith(".md") ? base : `${base}.md`;
}

function isTauriApp(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function browserDownload(name: string, content: string): void {
  const blob = new Blob([content], { type: "text/markdown;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = name;
  anchor.style.display = "none";
  document.body.appendChild(anchor);
  anchor.click();
  document.body.removeChild(anchor);
  URL.revokeObjectURL(url);
}

export async function downloadMarkdown(filename: string, content: string): Promise<void> {
  const name = filename.endsWith(".md") ? filename : `${filename}.md`;

  if (isTauriApp()) {
    const path = await save({
      filters: [{ name: "Markdown", extensions: ["md"] }],
      defaultPath: name,
    });
    if (!path) return;
    try {
      await invoke("save_text_file", { path, content });
      pushToast(`Saved ${name}`);
    } catch (error) {
      pushToast(String(error), "error");
    }
    return;
  }

  try {
    browserDownload(name, content);
  } catch (error) {
    pushToast(String(error), "error");
  }
}

export interface ChatExportMessage {
  role: "user" | "assistant";
  content: string;
}

export function buildChatMarkdown(messages: ChatExportMessage[]): string {
  const parts: string[] = [];
  for (const message of messages) {
    if (!message.content.trim()) continue;
    const heading = message.role === "user" ? "You" : "Assistant";
    parts.push(`## ${heading}\n\n${message.content.trim()}`);
  }
  return parts.join("\n\n---\n\n");
}
