<script lang="ts">
  interface Props {
    open: boolean;
    onChooseFolder: () => void | Promise<void>;
    onDismiss: () => void;
  }

  let { open, onChooseFolder, onDismiss }: Props = $props();
</script>

{#if open}
  <div class="backdrop" role="presentation" onclick={onDismiss}></div>
  <div class="modal" role="dialog" aria-labelledby="workspace-title" aria-modal="true">
    <header class="head">
      <h3 id="workspace-title">Choose a workspace folder</h3>
      <p class="sub">
        Agent tools and plans need at least one allowed folder on your computer.
        Pick where the model can read and write files (e.g. your project directory).
      </p>
    </header>

    <footer class="actions">
      <button type="button" class="btn ghost" onclick={onDismiss}>Cancel</button>
      <button type="button" class="btn primary" onclick={onChooseFolder}>
        Choose folder…
      </button>
    </footer>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.45);
    z-index: 200;
  }

  .modal {
    position: fixed;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    z-index: 201;
    width: min(440px, calc(100vw - 2rem));
    padding: var(--space-5);
    border-radius: var(--radius-lg);
    border: 1px solid var(--color-border);
    background: var(--color-bg-panel);
    box-shadow: var(--shadow-lg);
  }

  .head h3 {
    margin: 0 0 var(--space-2);
    font-size: var(--text-lg);
  }

  .sub {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    margin-top: var(--space-5);
  }

  .btn {
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .ghost {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .primary {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }
</style>
