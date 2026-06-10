<script lang="ts">
  import AttachmentStrip from "$lib/components/AttachmentStrip.svelte";
  import ChatBubble from "$lib/components/ChatBubble.svelte";
  import {
    checkOllama,
    formatAudioNote,
    formatBytes,
    hasCapability,
    listModels,
    prepareAudioForOllama,
    streamChat,
    supportsAudio,
  } from "$lib/ollama";
  import type {
    AttachmentDisplay,
    ChatMessage,
    DisplayMessage,
    ModelInfo,
    PendingAttachment,
    ToolCallDisplay,
  } from "$lib/types";
  import { wavBase64ToUrl } from "$lib/utils/audio";
  import { parseToolCalls } from "$lib/utils/tools";

  const DEFAULT_AUDIO_PROMPT = "Transcribe this audio";

  let models = $state<ModelInfo[]>([]);
  let selectedModel = $state("");
  let connected = $state(false);
  let statusMessage = $state("Checking Ollama…");
  let thinkEnabled = $state(true);
  let input = $state("");
  let pending = $state<PendingAttachment[]>([]);
  let messages = $state<DisplayMessage[]>([]);
  let history = $state<ChatMessage[]>([]);
  let isStreaming = $state(false);
  let isPreparingAudio = $state(false);
  let errorMessage = $state("");
  let messagesEnd = $state<HTMLDivElement | null>(null);

  const selectedModelInfo = $derived(
    models.find((m) => m.name === selectedModel),
  );
  const supportsThinking = $derived(
    hasCapability(selectedModelInfo, "thinking"),
  );
  const supportsVision = $derived(hasCapability(selectedModelInfo, "vision"));
  const supportsTools = $derived(hasCapability(selectedModelInfo, "tools"));
  const supportsAudioInput = $derived(supportsAudio(selectedModelInfo));
  const hasPendingAudio = $derived(
    pending.some((a) => a.kind === "audio"),
  );

  async function refreshModels() {
    try {
      await checkOllama();
      connected = true;
      statusMessage = "Connected to Ollama";
      models = await listModels();
      if (!selectedModel && models.length > 0) {
        selectedModel = models[0].name;
      }
      errorMessage = "";
    } catch (error) {
      connected = false;
      statusMessage = "Ollama not reachable";
      errorMessage = String(error);
      models = [];
    }
  }

  $effect(() => {
    refreshModels();
  });

  $effect(() => {
    if (messages.length > 0) {
      messagesEnd?.scrollIntoView({ behavior: "smooth" });
    }
  });

  function readFileAsBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result;
        if (typeof result !== "string") {
          reject(new Error("Failed to read image"));
          return;
        }
        resolve(result.split(",")[1] ?? result);
      };
      reader.onerror = () =>
        reject(reader.error ?? new Error("Failed to read image"));
      reader.readAsDataURL(file);
    });
  }

  async function onImageSelected(event: Event) {
    const target = event.target as HTMLInputElement;
    const files = target.files;
    if (!files?.length) return;

    for (const file of Array.from(files)) {
      const base64 = await readFileAsBase64(file);
      pending = [
        ...pending,
        {
          kind: "image",
          base64,
          label: file.name,
          previewUrl: URL.createObjectURL(file),
        },
      ];
    }
    target.value = "";
  }

  async function onAudioSelected(event: Event) {
    const target = event.target as HTMLInputElement;
    const files = target.files;
    if (!files?.length) return;

    isPreparingAudio = true;
    errorMessage = "";

    try {
      for (const file of Array.from(files)) {
        const result = await prepareAudioForOllama(file);
        pending = [
          ...pending,
          {
            kind: "audio",
            base64: result.base64,
            label: file.name,
            previewUrl: wavBase64ToUrl(result.base64),
            note: formatAudioNote(result),
            durationSecs: result.duration_secs,
          },
        ];
      }
    } catch (error) {
      errorMessage = String(error);
    } finally {
      isPreparingAudio = false;
      target.value = "";
    }
  }

  function removePending(index: number) {
    pending = pending.filter((_, i) => i !== index);
  }

  function attachmentsForOllama(items: PendingAttachment[]): string[] {
    return [...items]
      .sort((a, b) => (a.kind === "audio" ? -1 : b.kind === "audio" ? 1 : 0))
      .map((item) => item.base64);
  }

  function attachmentsForDisplay(
    items: PendingAttachment[],
  ): AttachmentDisplay[] {
    return items.map((item) => ({
      kind: item.kind,
      label: item.label,
      previewUrl: item.previewUrl,
      note: item.note,
      durationSecs: item.durationSecs,
    }));
  }

  async function sendMessage() {
    const trimmed = input.trim();
    if (
      (!trimmed && pending.length === 0) ||
      !selectedModel ||
      isStreaming ||
      isPreparingAudio
    ) {
      return;
    }

    const includesAudio = hasPendingAudio;
    const messageContent =
      trimmed || (includesAudio ? DEFAULT_AUDIO_PROMPT : "");

    const sentAttachments = attachmentsForDisplay(pending);

    messages = [
      ...messages,
      {
        id: crypto.randomUUID(),
        role: "user",
        content: messageContent,
        attachments: sentAttachments,
      },
    ];

    history = [
      ...history,
      {
        role: "user",
        content: messageContent,
        images:
          pending.length > 0 ? attachmentsForOllama(pending) : undefined,
      },
    ];

    input = "";
    pending = [];

    const assistantId = crypto.randomUUID();
    messages = [
      ...messages,
      {
        id: assistantId,
        role: "assistant",
        content: "",
        thinking: "",
        toolCalls: [],
        streaming: true,
      },
    ];

    isStreaming = true;
    errorMessage = "";

    let thinking = "";
    let content = "";
    let toolCalls: ToolCallDisplay[] = [];

    try {
      const unlisten = await streamChat(
        {
          model: selectedModel,
          messages: history,
          think: supportsThinking ? thinkEnabled : undefined,
          stream: true,
          options: includesAudio ? { num_ctx: 8192 } : undefined,
        },
        (chunk) => {
          if (chunk.error) errorMessage = chunk.error;
          if (chunk.thinking) thinking += chunk.thinking;
          if (chunk.content) content += chunk.content;
          if (chunk.tool_calls?.length) {
            toolCalls = parseToolCalls(chunk.tool_calls).map((t) => ({
              ...t,
              status: "running" as const,
            }));
          }

          messages = messages.map((m) =>
            m.id === assistantId
              ? {
                  ...m,
                  thinking,
                  content,
                  toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
                }
              : m,
          );

          if (chunk.done) isStreaming = false;
        },
      );

      if (toolCalls.length > 0) {
        toolCalls = toolCalls.map((t) => ({ ...t, status: "completed" as const }));
      }

      history = [
        ...history,
        {
          role: "assistant",
          content,
          thinking: thinking || undefined,
          tool_calls:
            toolCalls.length > 0
              ? toolCalls.map((t) => ({
                  function: { name: t.name, arguments: t.arguments },
                }))
              : undefined,
        },
      ];

      messages = messages.map((m) =>
        m.id === assistantId
          ? {
              ...m,
              streaming: false,
              toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
            }
          : m,
      );

      await unlisten();
    } catch (error) {
      isStreaming = false;
      errorMessage = String(error);
      messages = messages.map((m) =>
        m.id === assistantId
          ? {
              ...m,
              content: "Failed to get a response.",
              streaming: false,
            }
          : m,
      );
    }
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void sendMessage();
    }
  }
</script>

<div class="shell">
  <aside class="sidebar">
    <div class="sidebar-brand">
      <div class="logo">LLM</div>
      <div>
        <h1>Local LLM UI</h1>
        <p class="status" class:online={connected} class:offline={!connected}>
          <span class="status-dot"></span>
          {statusMessage}
        </p>
      </div>
    </div>

    <div class="sidebar-section">
      <label class="field">
        <span class="field-label">Model</span>
        <select
          bind:value={selectedModel}
          disabled={!connected || isStreaming}
        >
          {#each models as model}
            <option value={model.name}>
              {model.name} ({formatBytes(model.size)})
            </option>
          {/each}
        </select>
      </label>

      {#if selectedModelInfo?.capabilities.length}
        <div class="badges">
          {#each selectedModelInfo.capabilities as cap}
            <span class="badge">{cap}</span>
          {/each}
        </div>
      {/if}
    </div>

    <div class="sidebar-section">
      <label class="check">
        <input
          type="checkbox"
          bind:checked={thinkEnabled}
          disabled={!supportsThinking || isStreaming}
        />
        Show thinking
      </label>
      <button
        type="button"
        class="btn btn-ghost"
        onclick={refreshModels}
        disabled={isStreaming}
      >
        Refresh models
      </button>
    </div>

    <div class="sidebar-section tools-info">
      <h2 class="section-title">Capabilities</h2>
      <ul class="cap-list">
        <li class:ok={supportsVision}>
          <span>Vision</span>
          <span>{supportsVision ? "on" : "off"}</span>
        </li>
        <li class:ok={supportsAudioInput}>
          <span>Audio</span>
          <span>{supportsAudioInput ? "30s max" : "off"}</span>
        </li>
        <li class:ok={supportsTools}>
          <span>Tools / MCP</span>
          <span>{supportsTools ? "display" : "off"}</span>
        </li>
      </ul>
      {#if supportsTools}
        <p class="hint">
          Tool & MCP calls appear as structured cards in chat. Execution loop
          coming in v0.3.
        </p>
      {/if}
    </div>
  </aside>

  <div class="main">
    <div class="thread-scroll">
      <div class="thread">
        {#if messages.length === 0}
          <div class="empty">
            <h2>Chat with local models</h2>
            <p>
              Attach images for vision models or audio for Gemma 4. Audio
              includes playback controls; images open in a preview lightbox.
            </p>
          </div>
        {:else}
          {#each messages as message (message.id)}
            <div
              class="row"
              class:user={message.role === "user"}
              class:assistant={message.role === "assistant"}
            >
              <ChatBubble
                role={message.role}
                content={message.content}
                thinking={message.thinking}
                attachments={message.attachments}
                toolCalls={message.toolCalls}
                streaming={message.streaming}
                showThinking={thinkEnabled}
              />
            </div>
          {/each}
          <div bind:this={messagesEnd}></div>
        {/if}
      </div>
    </div>

    {#if errorMessage}
      <div class="error-banner" role="alert">{errorMessage}</div>
    {/if}

    <footer class="composer">
      <AttachmentStrip
        items={pending}
        onRemove={removePending}
        disabled={isStreaming || isPreparingAudio}
      />

      {#if isPreparingAudio}
        <p class="preparing">Converting audio to 16 kHz WAV…</p>
      {/if}

      <div class="composer-inner">
        <div class="composer-actions">
          {#if supportsVision}
            <label class="btn btn-ghost attach-btn">
              <input
                type="file"
                accept="image/*"
                multiple
                onchange={onImageSelected}
                disabled={isStreaming || isPreparingAudio}
              />
              <svg width="18" height="18" viewBox="0 0 18 18" fill="currentColor" aria-hidden="true">
                <path d="M2 4a2 2 0 012-2h8l4 4v8a2 2 0 01-2 2H4a2 2 0 01-2-2V4zm10 0v3h3M6 10l2.5-3 2 2.5L13 8" stroke="currentColor" stroke-width="1.2" fill="none"/>
              </svg>
              Image
            </label>
          {/if}
          {#if supportsAudioInput}
            <label class="btn btn-ghost attach-btn">
              <input
                type="file"
                accept="audio/*,.mp3,.wav,.ogg,.flac,.m4a,.aac,.webm"
                multiple
                onchange={onAudioSelected}
                disabled={isStreaming || isPreparingAudio}
              />
              <svg width="18" height="18" viewBox="0 0 18 18" fill="currentColor" aria-hidden="true">
                <path d="M9 1a3 3 0 00-3 3v5a3 3 0 006 0V4a3 3 0 00-3-3zm-5 8a5 5 0 0010 0h2a7 7 0 01-14 0h2zm3 0H6a3 3 0 006 0h-1a2 2 0 11-4 0H7z"/>
              </svg>
              Audio
            </label>
          {/if}
        </div>

        <textarea
          class="composer-input"
          bind:value={input}
          placeholder={hasPendingAudio
            ? "Optional prompt (defaults to “Transcribe this audio”)…"
            : "Message the model…"}
          rows="2"
          onkeydown={onKeydown}
          disabled={!connected || isStreaming || isPreparingAudio}
        ></textarea>

        <button
          type="button"
          class="btn btn-primary send-btn"
          onclick={sendMessage}
          disabled={!connected || isStreaming || isPreparingAudio}
        >
          {#if isStreaming}
            Streaming…
          {:else if isPreparingAudio}
            Converting…
          {:else}
            Send
          {/if}
        </button>
      </div>
    </footer>
  </div>
</div>

<style>
  .shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--color-bg);
  }

  /* Sidebar */
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
    padding: var(--space-5);
    background: var(--color-bg-elevated);
    border-right: 1px solid var(--color-border);
    overflow-y: auto;
  }

  .sidebar-brand {
    display: flex;
    gap: var(--space-3);
    align-items: center;
  }

  .logo {
    width: 40px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-md);
    background: var(--color-primary);
    font-weight: 700;
    font-size: var(--text-sm);
  }

  .sidebar-brand h1 {
    margin: 0;
    font-size: var(--text-base);
    font-weight: 600;
  }

  .status {
    margin: var(--space-1) 0 0;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .status-dot {
    width: 7px;
    height: 7px;
    border-radius: var(--radius-full);
    background: var(--color-text-muted);
  }

  .status.online .status-dot {
    background: var(--color-success);
    box-shadow: 0 0 6px var(--color-success);
  }

  .status.offline .status-dot {
    background: var(--color-error);
  }

  .sidebar-section {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .section-title {
    margin: 0;
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .field-label {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
  }

  select,
  textarea {
    background: var(--color-bg-inset);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    color: var(--color-text);
  }

  select:focus,
  textarea:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: 1px;
  }

  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .badge {
    font-size: var(--text-xs);
    padding: 2px 8px;
    border-radius: var(--radius-full);
    background: rgba(79, 106, 245, 0.12);
    color: #9ec5ff;
    border: 1px solid rgba(79, 106, 245, 0.2);
  }

  .check {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .cap-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .cap-list li {
    display: flex;
    justify-content: space-between;
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
  }

  .cap-list li.ok span:last-child {
    color: var(--color-success);
  }

  .tools-info {
    margin-top: auto;
  }

  .hint {
    margin: 0;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    line-height: 1.4;
  }

  /* Main chat */
  .main {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    background: var(--color-bg);
  }

  .thread-scroll {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .thread {
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    margin: 0 auto;
    padding: var(--space-6) var(--space-4);
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    width: 100%;
  }

  .row {
    display: flex;
    width: 100%;
  }

  .row.user {
    justify-content: flex-end;
  }

  .row.assistant {
    justify-content: flex-start;
  }

  .row :global(.bubble) {
    max-width: 100%;
  }

  .row.user :global(.bubble) {
    max-width: min(100%, 560px);
  }

  .empty {
    margin: auto;
    padding: var(--space-8) var(--space-4);
    text-align: center;
    max-width: 420px;
  }

  .empty h2 {
    margin: 0 0 var(--space-3);
    font-size: var(--text-lg);
    font-weight: 600;
  }

  .empty p {
    margin: 0;
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .error-banner {
    flex-shrink: 0;
    margin: 0 var(--space-4) var(--space-2);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--color-error-bg);
    border: 1px solid rgba(248, 113, 113, 0.35);
    color: var(--color-error);
    font-size: var(--text-sm);
  }

  /* Composer */
  .composer {
    flex-shrink: 0;
    padding: var(--space-4);
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
  }

  .composer-inner {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: var(--space-3);
    align-items: flex-end;
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    margin: 0 auto;
  }

  .composer-actions {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .composer-input {
    resize: none;
    min-height: 52px;
    max-height: 160px;
    line-height: 1.5;
  }

  .preparing {
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    margin: 0 auto var(--space-2);
    font-size: var(--text-sm);
    color: var(--color-tool);
  }

  /* Buttons */
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid transparent;
    font-size: var(--text-sm);
    font-weight: 500;
    white-space: nowrap;
  }

  .btn-primary {
    background: var(--color-primary);
    color: white;
    border-color: var(--color-primary);
    min-width: 88px;
    padding: var(--space-3) var(--space-5);
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .btn-ghost {
    background: var(--color-bg-inset);
    border-color: var(--color-border);
    color: var(--color-text-secondary);
  }

  .btn-ghost:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .attach-btn {
    cursor: pointer;
    position: relative;
  }

  .attach-btn input {
    position: absolute;
    width: 0;
    height: 0;
    opacity: 0;
  }

  .send-btn {
    align-self: stretch;
  }
</style>
