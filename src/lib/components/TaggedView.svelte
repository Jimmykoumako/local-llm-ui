<script lang="ts">
  import type { TaggedMessage } from "$lib/tagged-messages";

  interface Props {
    tags: TaggedMessage[];
    onOpen: (tag: TaggedMessage) => void;
    onRemove: (id: string) => void;
  }

  let { tags, onOpen, onRemove }: Props = $props();

  const sorted = $derived(
    [...tags].sort((a, b) => b.taggedAt - a.taggedAt),
  );

  function formatWhen(ts: number): string {
    return new Date(ts).toLocaleString(undefined, {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

<div class="tagged-view">
  <header class="tagged-header">
    <h2>Tagged messages</h2>
    <p class="hint">Star a message in chat to save it here. Open any item to jump back to that conversation.</p>
  </header>

  {#if sorted.length === 0}
    <div class="empty">
      <p>No tagged messages yet.</p>
      <p class="sub">Use the ☆ button on any message to bookmark it.</p>
    </div>
  {:else}
    <ul class="tag-list">
      {#each sorted as tag (tag.id)}
        <li class="tag-item">
          <button type="button" class="tag-open" onclick={() => onOpen(tag)}>
            <span class="tag-meta">
              <span class="role">{tag.role === "user" ? "You" : "Assistant"}</span>
              <span class="chat">{tag.conversationTitle}</span>
              <span class="when">{formatWhen(tag.taggedAt)}</span>
            </span>
            <span class="preview">{tag.preview}</span>
          </button>
          <button
            type="button"
            class="tag-remove"
            onclick={() => onRemove(tag.id)}
            aria-label="Remove tag"
            title="Remove tag"
          >
            ×
          </button>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .tagged-view {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    margin: 0 auto;
    width: 100%;
    padding: var(--space-4);
  }

  .tagged-header {
    flex-shrink: 0;
    padding-bottom: var(--space-4);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .tagged-header h2 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-lg);
    font-weight: 600;
  }

  .hint {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--color-text-muted);
    line-height: 1.5;
  }

  .empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: var(--color-text-muted);
    gap: var(--space-2);
  }

  .empty .sub {
    font-size: var(--text-sm);
    margin: 0;
  }

  .tag-list {
    list-style: none;
    margin: var(--space-4) 0 0;
    padding: 0;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .tag-item {
    display: flex;
    align-items: stretch;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-md);
    background: var(--color-bg-inset);
    overflow: hidden;
  }

  .tag-open {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-4);
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .tag-open:hover {
    background: var(--color-bg-hover);
  }

  .tag-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    width: 100%;
  }

  .role {
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-secondary);
  }

  .chat {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .when {
    flex-shrink: 0;
  }

  .preview {
    font-size: var(--text-sm);
    color: var(--color-text);
    line-height: 1.5;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .tag-remove {
    width: 36px;
    border: none;
    border-left: 1px solid var(--color-border-subtle);
    background: transparent;
    color: var(--color-text-muted);
    font-size: 1.1rem;
    flex-shrink: 0;
  }

  .tag-remove:hover {
    color: var(--color-error);
    background: var(--color-bg-hover);
  }
</style>
