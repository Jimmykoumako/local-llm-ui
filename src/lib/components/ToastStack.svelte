<script lang="ts">
  import {
    dismissToast,
    getToastVersion,
    getToasts,
    subscribeToasts,
  } from "$lib/toast";

  let tick = $state(0);

  $effect(() => {
    return subscribeToasts(() => {
      tick = getToastVersion();
    });
  });

  const toasts = $derived.by(() => {
    tick;
    return getToasts();
  });
</script>

<div class="stack" aria-live="polite" aria-relevant="additions">
  {#each toasts as toast (toast.id)}
    <div class="toast" data-kind={toast.kind} role="status">
      <p>{toast.message}</p>
      <button
        type="button"
        class="dismiss"
        onclick={() => dismissToast(toast.id)}
        aria-label="Dismiss"
      >×</button>
    </div>
  {/each}
</div>

<style>
  .stack {
    position: fixed;
    right: var(--space-4);
    bottom: var(--space-4);
    z-index: 300;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    width: min(380px, calc(100vw - 2rem));
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg-panel);
    box-shadow: var(--shadow-lg);
    animation: slide-in 0.18s ease;
  }

  .toast[data-kind="warning"] {
    border-color: rgba(251, 191, 36, 0.45);
    background: rgba(42, 34, 16, 0.95);
  }

  .toast[data-kind="error"] {
    border-color: rgba(248, 113, 113, 0.45);
    background: rgba(42, 21, 24, 0.95);
  }

  .toast p {
    margin: 0;
    flex: 1;
    font-size: var(--text-sm);
    line-height: 1.5;
    color: var(--color-text);
  }

  .dismiss {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 1.1rem;
    line-height: 1;
    padding: 0;
    cursor: pointer;
  }

  .dismiss:hover {
    color: var(--color-text);
  }

  @keyframes slide-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
