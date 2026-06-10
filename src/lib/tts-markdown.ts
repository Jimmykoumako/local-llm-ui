/** Shared markdown → speakable text helpers for TTS. */

export function extractFencedBlocks(text: string): string[] {
  const blocks: string[] = [];
  const pattern = /```[^\n]*\n([\s\S]*?)```/g;
  for (const match of text.matchAll(pattern)) {
    const body = match[1]?.trim();
    if (body) blocks.push(body);
  }
  return blocks;
}

function isSubstantialBlock(text: string): boolean {
  const lines = text.split("\n").filter((line) => line.trim()).length;
  return text.length >= 60 || lines >= 3;
}

export function cleanPlainTextForTtsExport(text: string): string {
  return text
    .replace(/\r\n/g, "\n")
    .replace(/\[([^\]]+)\]/g, "$1")
    .replace(/\*\*([^*]+)\*\*/g, "$1")
    .replace(/\*([^*]+)\*/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/`([^`]+)`/g, "$1")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .join("\n")
    .trim();
}

function stripMarkdownBody(text: string): string {
  return text
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/`[^`]+`/g, " ")
    .replace(/!\[[^\]]*]\([^)]+\)/g, " ")
    .replace(/\[([^\]]+)]\([^)]+\)/g, "$1")
    .replace(/^#{1,6}\s+/gm, "")
    .replace(/[*_~>#|-]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

/** Pick speakable text; prefers fenced letter/draft blocks over commentary. */
export function textForTts(markdown: string): string {
  const fenced = extractFencedBlocks(markdown);

  if (fenced.length > 0) {
    const substantial = fenced.filter(isSubstantialBlock);
    const candidates = substantial.length > 0 ? substantial : fenced;
    const primary = [...candidates].sort((a, b) => b.length - a.length)[0];
    const cleaned = cleanPlainTextForTtsExport(primary);
    if (cleaned) return cleaned;

    const combined = cleanPlainTextForTtsExport(fenced.join("\n\n"));
    if (combined) return combined;
  }

  return stripMarkdownBody(markdown);
}
