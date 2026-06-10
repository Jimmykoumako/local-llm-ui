<script lang="ts">
  import AudioPlayer from "./AudioPlayer.svelte";
  import ImagePreview from "./ImagePreview.svelte";
  import MarkdownContent from "./MarkdownContent.svelte";
  import ToolCallPanel from "./ToolCallPanel.svelte";
  import type { AttachmentDisplay, ToolCallDisplay } from "$lib/types";

  interface Props {
    role: "user" | "assistant";
    content: string;
    thinking?: string;
    attachments?: AttachmentDisplay[];
    toolCalls?: ToolCallDisplay[];
    streaming?: boolean;
    showThinking?: boolean;
  }

  let {
    role,
    content,
    thinking = "",
    attachments = [],
    toolCalls = [],
    streaming = false,
    showThinking = true,
  }: Props = $props();

  let thinkingOpen = $state(true);
  let contentMode = $state<"preview" | "raw">("preview");

  const isUser = $derived(role === "user");
  const hasThinking = $derived(Boolean(thinking?.trim()));
  const hasTools = $derived(toolCalls.length > 0);
</script>

<article class="bubble" class:user={isUser} class:assistant={!isUser}>
  <header class="meta">
    <span class="role">{isUser ? "You" : "Assistant"}</span>
    {#if streaming && !isUser}
      <span class="streaming-dot" aria-label="Streaming"></span>
    {/if}
    {#if !isUser && content.trim()}
      <div class="mode-toggle">
        <button
          type="button"
          class:active={contentMode === "preview"}
          onclick={() => (contentMode = "preview")}
        >Preview</button>
        <button
          type="button"
          class:active={contentMode === "raw"}
          onclick={() => (contentMode = "raw")}
        >Raw</button>
      </div>
    {/if}
  </header>

  {#if attachments.length > 0}
    <div class="attachments">
      {#each attachments as attachment (attachment.label + attachment.kind)}
        {#if attachment.kind === "image" && attachment.previewUrl}
          <ImagePreview
            src={attachment.previewUrl}
            alt={attachment.label}
            size={isUser ? "md" : "lg"}
          />
        {:else if attachment.kind === "audio" && attachment.previewUrl}
          <AudioPlayer
            src={attachment.previewUrl}
            label={attachment.label}
          />
        {/if}
      {/each}
    </div>
  {/if}

  {#if hasThinking && showThinking && !isUser}
    <details class="thinking" bind:open={thinkingOpen}>
      <summary>
        <span class="thinking-icon">◈</span> Thinking
        {#if streaming && !content.trim()}
          <span class="thinking-live">live</span>
        {/if}
      </summary>
      <div class="thinking-body">
        <MarkdownContent content={thinking} raw={false} />
      </div>
    </details>
  {/if}

  {#if hasTools}
    <ToolCallPanel tools={toolCalls} {streaming} />
  {/if}

  <div class="body">
    {#if content.trim()}
      {#if isUser}
        <p class="user-text">{content}</p>
      {:else}
        <MarkdownContent {content} raw={contentMode === "raw"} />
      {/if}
    {:else if streaming}
      <p class="placeholder">
        <span class="dots"><span></span><span></span><span></span></span>
        Generating…
      </p>
    {/if}
  </div>
</article>

<style>
  .bubble {
    width: 100%;
    max-width: var(--thread-max-width);
    padding: var(--space-4);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border-subtle);
    background: var(--color-assistant-bg);
  }

  .bubble.user {
    background: var(--color-user-bg);
    border-color: rgba(79, 106, 245, 0.2);
  }

  .meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-3);
  }

  .role {
    font-size: var(--text-xs);
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-secondary);
  }

  .streaming-dot {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--color-success);
    animation: blink 1s ease infinite;
  }

  @keyframes blink {
    50% { opacity: 0.3; }
  }

  .mode-toggle {
    margin-left: auto;
    display: flex;
    gap: 2px;
    padding: 2px;
    background: var(--color-bg-inset);
    border-radius: var(--radius-sm);
  }

  .mode-toggle button {
    padding: 2px 8px;
    border: none;
    border-radius: 4px;
    background: transparent;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .mode-toggle button.active {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .attachments {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .thinking {
    margin-bottom: var(--space-3);
    border: 1px solid rgba(183, 148, 246, 0.25);
    border-radius: var(--radius-md);
    background: rgba(183, 148, 246, 0.06);
    overflow: hidden;
  }

  .thinking summary {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    cursor: pointer;
    font-size: var(--text-sm);
    font-weight: 500;
    color: var(--color-thinking);
    list-style: none;
  }

  .thinking summary::-webkit-details-marker {
    display: none;
  }

  .thinking-icon {
    font-size: 0.75rem;
  }

  .thinking-live {
    margin-left: auto;
    font-size: var(--text-xs);
    color: var(--color-thinking);
    opacity: 0.8;
  }

  .thinking-body {
    padding: 0 var(--space-3) var(--space-3);
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    max-height: 280px;
    overflow-y: auto;
  }

  .body {
    font-size: var(--text-base);
    line-height: 1.6;
  }

  .user-text {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .placeholder {
    margin: 0;
    display: flex;
    align-items: center;
    gap: var(--space-2);
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .dots {
    display: flex;
    gap: 4px;
  }

  .dots span {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--color-text-muted);
    animation: bounce 1.2s ease infinite;
  }

  .dots span:nth-child(2) { animation-delay: 0.15s; }
  .dots span:nth-child(3) { animation-delay: 0.3s; }

  @keyframes bounce {
    0%, 80%, 100% { transform: translateY(0); opacity: 0.4; }
    40% { transform: translateY(-4px); opacity: 1; }
  }
</style>
