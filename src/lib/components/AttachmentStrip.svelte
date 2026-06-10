<script lang="ts">
  import AudioPlayer from "./AudioPlayer.svelte";
  import ImagePreview from "./ImagePreview.svelte";
  import type { PendingAttachment } from "$lib/types";
  import { revokeBlobUrl } from "$lib/utils/audio";

  interface Props {
    items: PendingAttachment[];
    onRemove: (index: number) => void;
    disabled?: boolean;
  }

  let { items, onRemove, disabled = false }: Props = $props();
</script>

{#if items.length > 0}
  <div class="strip">
    {#each items as item, index (item.label + index)}
      <div class="card">
        {#if item.kind === "image" && item.previewUrl}
          <ImagePreview src={item.previewUrl} alt={item.label} size="sm" />
        {:else if item.kind === "audio" && item.previewUrl}
          <AudioPlayer src={item.previewUrl} label={item.label} compact />
        {/if}
        {#if item.note}
          <span class="note">{item.note}</span>
        {/if}
        <button
          type="button"
          class="remove"
          onclick={() => {
            revokeBlobUrl(item.previewUrl);
            onRemove(index);
          }}
          aria-label="Remove attachment"
          {disabled}
        >×</button>
      </div>
    {/each}
  </div>
{/if}

<style>
  .strip {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    padding-bottom: var(--space-3);
  }

  .card {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .note {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    padding-left: var(--space-1);
  }

  .remove {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-full);
    background: var(--color-bg-panel);
    color: var(--color-text);
    font-size: 14px;
    line-height: 1;
    z-index: 1;
  }

  .remove:hover:not(:disabled) {
    background: var(--color-error-bg);
    color: var(--color-error);
    border-color: var(--color-error);
  }
</style>
