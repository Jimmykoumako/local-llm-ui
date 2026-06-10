export interface ModelInfo {
  name: string;
  size: number;
  capabilities: string[];
}

export interface ChatMessage {
  role: "user" | "assistant" | "system" | "tool";
  content?: string;
  thinking?: string;
  images?: string[];
  tool_calls?: unknown[];
  tool_name?: string;
}

export interface AttachmentDisplay {
  kind: "image" | "audio";
  label: string;
  previewUrl?: string;
  note?: string;
  durationSecs?: number;
}

export interface PendingAttachment {
  kind: "image" | "audio";
  base64: string;
  label: string;
  previewUrl?: string;
  note?: string;
  durationSecs?: number;
}

export interface ToolCallDisplay {
  id?: string;
  name: string;
  arguments: Record<string, unknown> | string;
  status?: "requested" | "running" | "completed" | "failed";
  result?: string;
  source?: "ollama" | "mcp" | "agent";
}

export interface AudioPrepareResult {
  base64: string;
  method: "ffmpeg" | "rust";
  duration_secs: number;
  trimmed: boolean;
}

export interface DisplayMessage {
  id: string;
  role: "user" | "assistant";
  content: string;
  thinking?: string;
  thinkingDurationMs?: number;
  attachments?: AttachmentDisplay[];
  toolCalls?: ToolCallDisplay[];
  streaming?: boolean;
}

export interface ChatChunk {
  thinking?: string;
  content?: string;
  tool_calls?: unknown[];
  done: boolean;
  cancelled?: boolean;
  error?: string;
}

export interface ChatRequest {
  sessionId: string;
  model: string;
  messages: ChatMessage[];
  think?: boolean | string;
  tools?: unknown[];
  stream: boolean;
  options?: Record<string, unknown>;
}
