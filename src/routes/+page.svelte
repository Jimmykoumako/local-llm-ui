<script lang="ts">
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
  } from "$lib/types";

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

  const selectedModelInfo = $derived(
    models.find((model) => model.name === selectedModel),
  );
  const supportsThinking = $derived(
    hasCapability(selectedModelInfo, "thinking"),
  );
  const supportsVision = $derived(hasCapability(selectedModelInfo, "vision"));
  const supportsTools = $derived(hasCapability(selectedModelInfo, "tools"));
  const supportsAudioInput = $derived(supportsAudio(selectedModelInfo));
  const hasPendingAudio = $derived(
    pending.some((attachment) => attachment.kind === "audio"),
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

  function readFileAsBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result;
        if (typeof result !== "string") {
          reject(new Error("Failed to read image"));
          return;
        }
        const base64 = result.split(",")[1] ?? result;
        resolve(base64);
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
            note: formatAudioNote(result),
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
    const attachment = pending[index];
    if (attachment.previewUrl) {
      URL.revokeObjectURL(attachment.previewUrl);
    }
    pending = pending.filter((_, i) => i !== index);
  }

  function attachmentsForOllama(items: PendingAttachment[]): string[] {
    return [...items]
      .sort((a, b) => {
        if (a.kind === b.kind) return 0;
        return a.kind === "audio" ? -1 : 1;
      })
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

    const userMessage: DisplayMessage = {
      id: crypto.randomUUID(),
      role: "user",
      content: messageContent,
      attachments: attachmentsForDisplay(pending),
    };

    const userHistory: ChatMessage = {
      role: "user",
      content: messageContent,
      images:
        pending.length > 0 ? attachmentsForOllama(pending) : undefined,
    };

    messages = [...messages, userMessage];
    history = [...history, userHistory];

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
    let toolCalls: unknown[] = [];

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
          if (chunk.error) {
            errorMessage = chunk.error;
          }
          if (chunk.thinking) {
            thinking += chunk.thinking;
          }
          if (chunk.content) {
            content += chunk.content;
          }
          if (chunk.tool_calls?.length) {
            toolCalls = chunk.tool_calls;
          }

          messages = messages.map((message) =>
            message.id === assistantId
              ? {
                  ...message,
                  thinking,
                  content,
                  toolCalls: toolCalls.length > 0 ? toolCalls : undefined,
                }
              : message,
          );

          if (chunk.done) {
            isStreaming = false;
          }
        },
      );

      history = [
        ...history,
        {
          role: "assistant",
          content,
          thinking: thinking || undefined,
          tool_calls: toolCalls.length > 0 ? toolCalls : undefined,
        },
      ];

      messages = messages.map((message) =>
        message.id === assistantId
          ? { ...message, streaming: false }
          : message,
      );

      await unlisten();
    } catch (error) {
      isStreaming = false;
      errorMessage = String(error);
      messages = messages.map((message) =>
        message.id === assistantId
          ? {
              ...message,
              content: "Failed to get a response.",
              streaming: false,
            }
          : message,
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

<div class="app">
  <aside class="sidebar">
    <header>
      <h1>Local LLM UI</h1>
      <p class:online={connected} class:offline={!connected}>{statusMessage}</p>
    </header>

    <label>
      Model
      <select bind:value={selectedModel} disabled={!connected || isStreaming}>
        {#each models as model}
          <option value={model.name}>
            {model.name} ({formatBytes(model.size)})
          </option>
        {/each}
      </select>
    </label>

    {#if selectedModelInfo}
      <div class="capabilities">
        {#each selectedModelInfo.capabilities as capability}
          <span class="badge">{capability}</span>
        {/each}
      </div>
    {/if}

    <label class="toggle">
      <input
        type="checkbox"
        bind:checked={thinkEnabled}
        disabled={!supportsThinking || isStreaming}
      />
      Show thinking
    </label>

    <button class="secondary" onclick={refreshModels} disabled={isStreaming}>
      Refresh models
    </button>

    <div class="hints">
      <p>Vision: {supportsVision ? "supported" : "not available"}</p>
      <p>Audio: {supportsAudioInput ? "supported · max 30s clip" : "not available"}</p>
      <p>Tools: {supportsTools ? "display only (execution next)" : "not available"}</p>
    </div>
  </aside>

  <main class="chat">
    <section class="messages">
      {#if messages.length === 0}
        <div class="empty">
          <h2>Chat with your local models</h2>
          <p>
            Attach images for vision models, or audio for Gemma 4-style models.
            Audio is converted to 16 kHz mono WAV (ffmpeg or built-in fallback) and trimmed to 30 seconds.
          </p>
        </div>
      {/if}

      {#each messages as message (message.id)}
        <article class="message" class:user={message.role === "user"}>
          <header>{message.role === "user" ? "You" : "Assistant"}</header>

          {#if message.attachments?.length}
            <div class="attachments">
              {#each message.attachments as attachment}
                {#if attachment.kind === "image" && attachment.previewUrl}
                  <img src={attachment.previewUrl} alt={attachment.label} />
                {:else}
                  <div class="audio-attachment">
                    <span class="audio-chip" title={attachment.label}>
                      🎤 {attachment.label}
                    </span>
                    {#if attachment.note}
                      <span class="attachment-note">{attachment.note}</span>
                    {/if}
                  </div>
                {/if}
              {/each}
            </div>
          {/if}

          {#if message.thinking}
            <details class="thinking" open={message.streaming}>
              <summary>Thinking</summary>
              <pre>{message.thinking}</pre>
            </details>
          {/if}

          {#if message.toolCalls?.length}
            <div class="tools">
              <strong>Tool calls</strong>
              <pre>{JSON.stringify(message.toolCalls, null, 2)}</pre>
            </div>
          {/if}

          {#if message.content}
            <p class="content">{message.content}</p>
          {:else if message.streaming}
            <p class="content muted">Waiting for response…</p>
          {/if}
        </article>
      {/each}
    </section>

    {#if errorMessage}
      <div class="error">{errorMessage}</div>
    {/if}

    <footer class="composer">
      {#if pending.length > 0}
        <div class="pending-attachments">
          {#each pending as attachment, index}
            <div class="pending-item">
              {#if attachment.kind === "image" && attachment.previewUrl}
                <img src={attachment.previewUrl} alt={attachment.label} />
              {:else}
                <div class="audio-attachment">
                  <span class="audio-chip">🎤 {attachment.label}</span>
                  {#if attachment.note}
                    <span class="attachment-note">{attachment.note}</span>
                  {/if}
                </div>
              {/if}
              <button
                type="button"
                onclick={() => removePending(index)}
                aria-label="Remove attachment"
                disabled={isStreaming || isPreparingAudio}
              >
                ×
              </button>
            </div>
          {/each}
        </div>
      {/if}

      {#if isPreparingAudio}
        <p class="preparing">Converting audio to 16 kHz WAV…</p>
      {/if}

      <div class="composer-row">
        <div class="attach-group">
          {#if supportsVision}
            <label class="attach">
              <input
                type="file"
                accept="image/*"
                multiple
                onchange={onImageSelected}
                disabled={isStreaming || isPreparingAudio}
              />
              Image
            </label>
          {/if}

          {#if supportsAudioInput}
            <label class="attach">
              <input
                type="file"
                accept="audio/*,.mp3,.wav,.ogg,.flac,.m4a,.aac,.webm"
                multiple
                onchange={onAudioSelected}
                disabled={isStreaming || isPreparingAudio}
              />
              Audio
            </label>
          {/if}
        </div>

        <textarea
          bind:value={input}
          placeholder={hasPendingAudio
            ? "Optional prompt (defaults to “Transcribe this audio”)…"
            : "Message the model…"}
          rows="3"
          onkeydown={onKeydown}
          disabled={!connected || isStreaming || isPreparingAudio}
        ></textarea>

        <button
          onclick={sendMessage}
          disabled={!connected || isStreaming || isPreparingAudio}
        >
          {isStreaming ? "Streaming…" : isPreparingAudio ? "Converting…" : "Send"}
        </button>
      </div>
    </footer>
  </main>
</div>

<style>
  .app {
    display: grid;
    grid-template-columns: 280px 1fr;
    height: 100vh;
  }

  .sidebar {
    padding: 1.25rem;
    border-right: 1px solid #2a2f3a;
    background: #151821;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .sidebar h1 {
    margin: 0;
    font-size: 1.1rem;
  }

  .online {
    color: #7dcea0;
  }

  .offline {
    color: #f1948a;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.85rem;
    color: #b8bec8;
  }

  select,
  textarea,
  button,
  .attach {
    border-radius: 10px;
    border: 1px solid #2f3642;
    background: #0f1117;
    color: inherit;
    font: inherit;
  }

  select,
  textarea {
    padding: 0.75rem;
  }

  button {
    padding: 0.75rem 1rem;
    cursor: pointer;
    background: #3d5afe;
    border-color: #3d5afe;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  button.secondary {
    background: #1f2430;
    border-color: #2f3642;
  }

  .toggle {
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
  }

  .capabilities {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .badge {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    border-radius: 999px;
    background: #243042;
    color: #9ec5ff;
  }

  .hints {
    margin-top: auto;
    font-size: 0.8rem;
    color: #8b93a1;
  }

  .chat {
    display: grid;
    grid-template-rows: 1fr auto;
    min-width: 0;
  }

  .messages {
    overflow-y: auto;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .empty {
    margin: auto;
    text-align: center;
    color: #8b93a1;
    max-width: 480px;
  }

  .message {
    max-width: 820px;
    padding: 1rem;
    border-radius: 14px;
    background: #171b24;
    border: 1px solid #2a2f3a;
  }

  .message.user {
    align-self: flex-end;
    background: #1d2a44;
  }

  .message header {
    font-size: 0.8rem;
    color: #9aa3b2;
    margin-bottom: 0.5rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .content {
    margin: 0;
    white-space: pre-wrap;
    line-height: 1.5;
  }

  .muted {
    color: #8b93a1;
  }

  .thinking,
  .tools {
    margin-bottom: 0.75rem;
    padding: 0.75rem;
    border-radius: 10px;
    background: #10141d;
    border: 1px solid #2a2f3a;
  }

  .thinking summary,
  .tools strong {
    cursor: pointer;
    color: #c7b3ff;
  }

  pre {
    margin: 0.5rem 0 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-size: 0.85rem;
    color: #d6d9df;
  }

  .attachments,
  .pending-attachments {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
  }

  .attachments img,
  .pending-item img {
    width: 96px;
    height: 96px;
    object-fit: cover;
    border-radius: 10px;
    border: 1px solid #2f3642;
  }

  .audio-attachment {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    max-width: 260px;
  }

  .audio-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.5rem 0.75rem;
    border-radius: 10px;
    background: #1a2333;
    border: 1px solid #2f3642;
    font-size: 0.85rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .attachment-note {
    font-size: 0.75rem;
    color: #8b93a1;
  }

  .pending-item {
    position: relative;
  }

  .pending-item button {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 24px;
    height: 24px;
    padding: 0;
    border-radius: 999px;
    background: #11151d;
  }

  .preparing {
    margin: 0 0 0.5rem;
    font-size: 0.85rem;
    color: #9ec5ff;
  }

  .error {
    margin: 0 1.5rem;
    padding: 0.75rem 1rem;
    border-radius: 10px;
    background: #3b1f24;
    color: #ffb4b4;
    border: 1px solid #6d2d36;
  }

  .composer {
    padding: 1rem 1.5rem 1.5rem;
    border-top: 1px solid #2a2f3a;
    background: #12151c;
  }

  .composer-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    gap: 0.75rem;
    align-items: end;
  }

  .attach-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .attach {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.75rem 1rem;
    cursor: pointer;
    white-space: nowrap;
  }

  .attach input {
    display: none;
  }

  textarea {
    resize: vertical;
    min-height: 72px;
  }
</style>
