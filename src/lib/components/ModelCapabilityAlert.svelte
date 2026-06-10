<script lang="ts">
  interface Props {
    title: string;
    message: string;
    suggestions?: string[];
    currentModel?: string;
    onBrowseModels: () => void;
    onSelectModel?: (name: string) => void;
  }

  let {
    title,
    message,
    suggestions = [],
    currentModel = "",
    onBrowseModels,
    onSelectModel,
  }: Props = $props();
</script>

<div class="alert" role="alert">
  <div class="icon">!</div>
  <div class="body">
    <strong>{title}</strong>
    <p>{message}</p>
    {#if currentModel}
      <p class="current">Current model: <code>{currentModel}</code></p>
    {/if}
    {#if suggestions.length > 0}
      <div class="suggestions">
        <span>Try a capable model:</span>
        <div class="chips">
          {#each suggestions as name}
            {#if onSelectModel}
              <button type="button" class="chip" onclick={() => onSelectModel(name)}>
                {name}
              </button>
            {:else}
              <span class="chip static">{name}</span>
            {/if}
          {/each}
        </div>
      </div>
    {/if}
    <button type="button" class="browse" onclick={onBrowseModels}>
      Open Models view
    </button>
  </div>
</div>

<style>
  .alert {
    display: flex;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid rgba(248, 113, 113, 0.45);
    background: var(--color-error-bg);
    color: var(--color-error);
  }

  .icon {
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-full);
    background: rgba(248, 113, 113, 0.2);
    font-weight: 700;
    font-size: var(--text-sm);
  }

  .body {
    flex: 1;
    min-width: 0;
  }

  .body strong {
    display: block;
    margin-bottom: var(--space-1);
    font-size: var(--text-sm);
  }

  .body p {
    margin: 0 0 var(--space-2);
    font-size: var(--text-sm);
    line-height: 1.5;
    color: #fca5a5;
  }

  .current {
    font-size: var(--text-xs) !important;
  }

  code {
    font-family: var(--font-mono);
    color: #fecaca;
  }

  .suggestions {
    margin-bottom: var(--space-2);
    font-size: var(--text-xs);
    color: #fca5a5;
  }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-top: var(--space-1);
  }

  .chip {
    padding: 2px 8px;
    border-radius: var(--radius-full);
    border: 1px solid rgba(248, 113, 113, 0.4);
    background: rgba(0, 0, 0, 0.2);
    font-size: var(--text-xs);
    color: #fecaca;
  }

  .chip.static {
    display: inline-block;
  }

  .browse {
    padding: var(--space-1) var(--space-3);
    border-radius: var(--radius-sm);
    border: 1px solid rgba(248, 113, 113, 0.5);
    background: transparent;
    font-size: var(--text-xs);
    color: #fecaca;
  }

  .browse:hover {
    background: rgba(248, 113, 113, 0.15);
  }
</style>
