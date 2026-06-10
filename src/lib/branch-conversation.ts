import type { ChatMessage, DisplayMessage } from "./types";

export function truncateHistoryToMessages(
  history: ChatMessage[],
  messages: DisplayMessage[],
): ChatMessage[] {
  if (messages.length === 0) return [];

  const systems = history.filter((m) => m.role === "system");
  const rest = history.filter((m) => m.role !== "system");

  const userLimit = messages.filter((m) => m.role === "user").length;
  const assistantLimit = messages.filter((m) => m.role === "assistant").length;

  const result: ChatMessage[] = [...systems];
  let users = 0;
  let assistants = 0;

  for (let i = 0; i < rest.length; i++) {
    const msg = rest[i];
    if (msg.role === "user") {
      if (users >= userLimit) break;
      users++;
      result.push(msg);
      continue;
    }

    if (msg.role === "assistant") {
      const hasTools = Boolean(
        msg.tool_calls && Array.isArray(msg.tool_calls) && msg.tool_calls.length > 0,
      );
      if (!hasTools && assistants >= assistantLimit) break;
      result.push(msg);
      let j = i + 1;
      while (j < rest.length && rest[j].role === "tool") {
        result.push(rest[j]);
        j++;
      }
      i = j - 1;
      if (!hasTools) assistants++;
      continue;
    }

    if (msg.role === "tool") {
      result.push(msg);
    }
  }

  return result;
}
