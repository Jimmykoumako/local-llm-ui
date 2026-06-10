<script lang="ts">
  import { formatBytes, supportsAudio } from "$lib/ollama";
  import type { ModelInfo } from "$lib/types";

  interface Props {
    models: ModelInfo[];
    selectedModel: string;
    connected: boolean;
    onSelectModel: (name: string) => void;
    onRefresh: () => void;
    onStartChat: () => void;
  }

  let {
    models,
    selectedModel,
    connected,
    onSelectModel,
    onRefresh,
    onStartChat,
  }: Props = $props();
</script>

<div class="view">
  <header class="view-header">
    <h2>Models</h2>
    <button type="button" class="btn" onclick={onRefresh} disabled={!connected}>
      Refresh
    </button>
  </header>

  {#if !connected}
    <p class="muted">Connect to Ollama to list models.</p>
  {:else if models.length === 0}
    <p class="muted">No models installed. Run <code>ollama pull &lt;model&gt;</code>.</p>
  {:else}
    <ul class="model-list">
      {#each models as model (model.name)}
        <li class="model-card" class:selected={model.name === selectedModel}>
          <div class="model-info">
            <strong>{model.name}</strong>
            <span class="size">{formatBytes(model.size)}</span>
            <div class="badges">
              {#each model.capabilities as cap}
                <span class="badge">{cap}</span>
              {/each}
              {#if supportsAudio(model)}
                <span class="badge">audio</span>
              {/if}
            </div>
          </div>
          <div class="actions">
            <button
              type="button"
              class="btn btn-primary"
              onclick={() => { onSelectModel(model.name); onStartChat(); }}
            >
              Chat
            </button>
            <button
              type="button"
              class="btn"
              class:active={model.name === selectedModel}
              onclick={() => onSelectModel(model.name)}
            >
              {model.name === selectedModel ? "Selected" : "Select"}
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .view {
    max-width: 720px;
    margin: 0 auto;
    padding: var(--space-6) var(--space-4);
  }

  .view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: var(--space-6);
  }

  .view-header h2 {
    margin: 0;
    font-size: var(--text-lg);
  }

  .muted {
    color: var(--color-text-secondary);
    font-size: var(--text-sm);
  }

  .model-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .model-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-4);
    padding: var(--space-4);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-panel);
  }

  .model-card.selected {
    border-color: rgba(79, 106, 245, 0.4);
  }

  .model-info strong {
    display: block;
    margin-bottom: var(--space-1);
  }

  .size {
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-top: var(--space-2);
  }

  .badge {
    font-size: 0.65rem;
    padding: 2px 6px;
    border-radius: var(--radius-full);
    background: rgba(79, 106, 245, 0.12);
    color: #9ec5ff;
  }

  .actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .btn {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
  }

  .btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .btn-primary {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .btn.active {
    border-color: var(--color-primary);
    color: #9ec5ff;
  }

  code {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    background: var(--color-bg-inset);
    padding: 2px 4px;
    border-radius: 4px;
  }
</style>
