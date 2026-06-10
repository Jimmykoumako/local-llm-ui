export interface TaggedMessage {
  id: string;
  conversationId: string;
  messageId: string;
  conversationTitle: string;
  role: "user" | "assistant";
  preview: string;
  content: string;
  taggedAt: number;
}

const STORAGE_KEY = "local-llm-ui-tagged-messages";

export function loadTaggedMessages(): TaggedMessage[] {
  if (typeof localStorage === "undefined") return [];
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw) as TaggedMessage[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function saveTaggedMessages(tags: TaggedMessage[]): void {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(tags));
}

export function messagePreview(content: string, max = 120): string {
  const line = content.split("\n").find((l) => l.trim())?.trim() ?? "";
  const text = line || content.trim();
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

export function isMessageTagged(
  tags: TaggedMessage[],
  conversationId: string,
  messageId: string,
): boolean {
  return tags.some(
    (t) => t.conversationId === conversationId && t.messageId === messageId,
  );
}

export function toggleTaggedMessage(
  tags: TaggedMessage[],
  entry: Omit<TaggedMessage, "id" | "taggedAt">,
): TaggedMessage[] {
  const existing = tags.find(
    (t) => t.conversationId === entry.conversationId && t.messageId === entry.messageId,
  );
  if (existing) {
    return tags.filter((t) => t.id !== existing.id);
  }
  const next: TaggedMessage = {
    ...entry,
    id: crypto.randomUUID(),
    taggedAt: Date.now(),
  };
  return [next, ...tags];
}

export function removeTaggedMessage(tags: TaggedMessage[], id: string): TaggedMessage[] {
  return tags.filter((t) => t.id !== id);
}
