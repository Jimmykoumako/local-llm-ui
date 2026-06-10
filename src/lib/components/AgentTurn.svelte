<script lang="ts">
  import MarkdownContent from "./MarkdownContent.svelte";
  import MessageActions from "./MessageActions.svelte";
  import type { ToolCallDisplay } from "$lib/types";
  import { enrichAssistantContentWithSearchFiles } from "$lib/utils/tool-results";
  import {
    formatThoughtDuration,
    formatToolActivity,
    thinkingSummary,
  } from "$lib/utils/tool-labels";

  interface Props {
    content: string;
    thinking?: string;
    toolCalls?: ToolCallDisplay[];
    contextToolCalls?: ToolCallDisplay[];
    streaming?: boolean;
    showThinking?: boolean;
    thinkingDurationMs?: number;
    ttsEnabled?: boolean;
    onSpeak?: () => void;
    onStopSpeak?: () => void;
    onSpeakBlock?: (text: string, blockKey: string) => void;
    speaking?: boolean;
    speakingBlockKey?: string | null;
    allowBlockDownload?: boolean;
    tagged?: boolean;
    onCopy?: () => void;
    onBranch?: () => void;
    onToggleTag?: () => void;
  }

  let {
    content,
    thinking = "",
    toolCalls = [],
    contextToolCalls = [],
    streaming = false,
    showThinking = true,
    thinkingDurationMs,
    ttsEnabled = false,
    onSpeak,
    onStopSpeak,
    onSpeakBlock,
    speaking = false,
    speakingBlockKey = null,
    allowBlockDownload = false,
    tagged = false,
    onCopy,
    onBranch,
    onToggleTag,
  }: Props = $props();

  let editingOpen = $state(true);
  let thinkingOpen = $state(true);
  let liveThinkingMs = $state(0);
  let liveTimer: ReturnType<typeof setInterval> | null = null;
  let liveStartedAt = $state<number | null>(null);

  const hasThinking = $derived(Boolean(thinking?.trim()));
  const allTools = $derived([...contextToolCalls, ...toolCalls]);
  const uniqueTools = $derived.by(() => {
    const seen = new Set<string>();
    return allTools.filter((tool) => {
      const key = `${tool.name}:${JSON.stringify(tool.arguments)}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
  });
  const hasTools = $derived(uniqueTools.length > 0);
  const fileTools = $derived(
    uniqueTools.filter((t) => t.source === "agent" || t.name !== "get_agent_capabilities"),
  );
  const thoughtLabel = $derived(
    formatThoughtDuration(
      streaming && liveThinkingMs > 0
        ? liveThinkingMs
        : thinkingDurationMs,
    ),
  );
  const thoughtPreview = $derived(thinkingSummary(thinking));
  const displayContent = $derived.by(() => {
    if (streaming || !content.trim()) return content;
    return enrichAssistantContentWithSearchFiles(content, allTools);
  });

  $effect(() => {
    if (streaming && hasThinking && liveStartedAt === null) {
      liveStartedAt = Date.now();
      liveTimer = setInterval(() => {
        if (liveStartedAt) liveThinkingMs = Date.now() - liveStartedAt;
      }, 500);
    }
    if (!streaming && liveTimer) {
      clearInterval(liveTimer);
      liveTimer = null;
    }
    return () => {
      if (liveTimer) clearInterval(liveTimer);
    };
  });
</script>

<article class="agent-turn">
  {#if hasThinking && showThinking}
    <details class="block thought" bind:open={thinkingOpen}>
      <summary>
        <span class="chev">›</span>
        <span class="label">{thoughtLabel}</span>
        {#if streaming && !content.trim()}
          <span class="live">live</span>
        {/if}
      </summary>
      <div class="block-body">
        {#if thoughtPreview && !streaming}
          <p class="preview">{thoughtPreview}</p>
        {/if}
        {#if streaming}
          <pre class="thinking-stream">{thinking}</pre>
        {:else}
          <MarkdownContent content={thinking} raw={false} />
        {/if}
      </div>
    </details>
  {/if}

  {#if hasTools}
    <details class="block editing" bind:open={editingOpen}>
      <summary>
        <span class="chev">›</span>
        <span class="label">Editing</span>
        <span class="count">{fileTools.length}</span>
      </summary>
      <ul class="activity">
        {#each fileTools as tool, i (tool.id ?? `${tool.name}-${i}`)}
          {@const line = formatToolActivity(tool)}
          <li class="activity-line" data-status={line.status ?? "requested"}>
            <span class="verb">{line.verb}</span>
            <span class="target">{line.target}</span>
            {#if line.detail}
              <span class="detail">{line.detail}</span>
            {/if}
            {#if line.status === "running"}
              <span class="spinner" aria-hidden="true"></span>
            {/if}
          </li>
        {/each}
      </ul>
    </details>
  {/if}

  {#if displayContent.trim()}
    <div class="response">
      {#if !streaming}
        <div class="speak-row">
          <MessageActions
            {ttsEnabled}
            {speaking}
            {tagged}
            {onCopy}
            {onBranch}
            {onSpeak}
            {onStopSpeak}
            {onToggleTag}
          />
        </div>
      {/if}
      <MarkdownContent
        content={displayContent}
        raw={false}
        {ttsEnabled}
        allowDownload={allowBlockDownload}
        {onSpeakBlock}
        {onStopSpeak}
        {speaking}
        {speakingBlockKey}
      />
    </div>
  {:else if streaming && !hasThinking && !hasTools}
    <p class="waiting">
      <span class="dots"><span></span><span></span><span></span></span>
      Working…
    </p>
  {/if}
</article>

<style>
  .agent-turn {
    width: 100%;
    max-width: var(--thread-max-width);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-1) 0;
  }

  .block {
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-md);
    background: var(--color-bg-inset);
    overflow: hidden;
  }

  .block summary {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    cursor: pointer;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    list-style: none;
    user-select: none;
  }

  .block summary::-webkit-details-marker {
    display: none;
  }

  .block[open] summary .chev {
    transform: rotate(90deg);
  }

  .chev {
    display: inline-block;
    font-size: 0.85rem;
    color: var(--color-text-muted);
    transition: transform 0.12s ease;
  }

  .label {
    font-weight: 500;
    color: var(--color-text);
  }

  .count {
    margin-left: auto;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .live {
    margin-left: auto;
    font-size: var(--text-xs);
    color: var(--color-thinking);
  }

  .block-body {
    padding: 0 var(--space-3) var(--space-3);
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    border-top: 1px solid var(--color-border-subtle);
  }

  .preview {
    margin: var(--space-2) 0 var(--space-3);
    color: var(--color-text);
    line-height: 1.5;
  }

  .thinking-stream {
    margin: var(--space-2) 0 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--font-sans);
    font-size: var(--text-sm);
    line-height: 1.6;
    max-height: 240px;
    overflow-y: auto;
  }

  .activity {
    list-style: none;
    margin: 0;
    padding: var(--space-2) var(--space-3) var(--space-3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .activity-line {
    display: flex;
    align-items: baseline;
    gap: var(--space-2);
    font-size: var(--text-sm);
    font-family: var(--font-mono);
    line-height: 1.5;
  }

  .verb {
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .target {
    color: var(--color-text);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .detail {
    color: var(--color-success);
    flex-shrink: 0;
    font-size: var(--text-xs);
  }

  .activity-line[data-status="failed"] .target {
    color: var(--color-error);
  }

  .spinner {
    width: 8px;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--color-warning);
    animation: blink 1s ease infinite;
    flex-shrink: 0;
  }

  @keyframes blink {
    50% { opacity: 0.3; }
  }

  .response {
    font-size: var(--text-base);
    line-height: 1.6;
    padding: var(--space-1) 0;
  }

  .speak-row {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--space-2);
  }

  .waiting {
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
