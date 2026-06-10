<script lang="ts">
  interface Props {
    ttsEnabled?: boolean;
    speaking?: boolean;
    tagged?: boolean;
    showTts?: boolean;
    onCopy?: () => void;
    onBranch?: () => void;
    onSpeak?: () => void;
    onStopSpeak?: () => void;
    onToggleTag?: () => void;
  }

  let {
    ttsEnabled = false,
    speaking = false,
    tagged = false,
    showTts = true,
    onCopy,
    onBranch,
    onSpeak,
    onStopSpeak,
    onToggleTag,
  }: Props = $props();
</script>

<div class="msg-actions">
  {#if onCopy}
    <button
      type="button"
      class="action-btn"
      onclick={onCopy}
      aria-label="Copy message"
      title="Copy"
    >
      ⧉
    </button>
  {/if}
  {#if onBranch}
    <button
      type="button"
      class="action-btn"
      onclick={onBranch}
      aria-label="Branch from here"
      title="Branch chat from here"
    >
      ⑂
    </button>
  {/if}
  {#if showTts && ttsEnabled}
    {#if speaking && onStopSpeak}
      <button
        type="button"
        class="action-btn stop"
        onclick={onStopSpeak}
        aria-label="Stop reading aloud"
        title="Stop"
      >
        ■
      </button>
    {:else if onSpeak}
      <button
        type="button"
        class="action-btn"
        onclick={onSpeak}
        aria-label="Read aloud"
        title="Read aloud"
      >
        🔊
      </button>
    {/if}
  {/if}
  {#if onToggleTag}
    <button
      type="button"
      class="action-btn"
      class:tagged
      onclick={onToggleTag}
      aria-label={tagged ? "Remove tag" : "Tag message"}
      title={tagged ? "Tagged — click to remove" : "Tag message"}
    >
      {tagged ? "★" : "☆"}
    </button>
  {/if}
</div>

<style>
  .msg-actions {
    margin-left: auto;
    display: flex;
    align-items: center;
    gap: var(--space-1);
  }

  .action-btn {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
    color: var(--color-text-secondary);
    font-size: var(--text-xs);
    padding: 2px 8px;
    line-height: 1.4;
  }

  .action-btn:hover:not(:disabled) {
    color: var(--color-text);
    border-color: var(--color-primary);
  }

  .action-btn.stop {
    color: var(--color-error);
    border-color: rgba(248, 113, 113, 0.45);
  }

  .action-btn.tagged {
    color: var(--color-warning);
    border-color: rgba(251, 191, 36, 0.45);
  }
</style>
