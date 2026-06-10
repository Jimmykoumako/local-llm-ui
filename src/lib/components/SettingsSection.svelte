<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    id: string;
    title: string;
    open: boolean;
    onToggle: () => void;
    highlight?: boolean;
    blocked?: boolean;
    hidden?: boolean;
    children: Snippet;
  }

  let {
    id,
    title,
    open,
    onToggle,
    highlight = false,
    blocked = false,
    hidden = false,
    children,
  }: Props = $props();
</script>

{#if !hidden}
  <section
    {id}
    class="card"
    class:highlight
    class:blocked
    class:collapsed={!open}
    data-settings-section={id}
  >
    <button
      type="button"
      class="section-head"
      onclick={onToggle}
      aria-expanded={open}
      aria-controls={`${id}-panel`}
    >
      <span class="chev" class:open>›</span>
      <h3>{title}</h3>
    </button>

    {#if open}
      <div class="section-body" id={`${id}-panel`}>
        {@render children()}
      </div>
    {/if}
  </section>
{/if}

<style>
  .card {
    padding: 0;
    margin-bottom: var(--space-3);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-panel);
    scroll-margin-top: var(--space-4);
  }

  .card.highlight {
    border-color: rgba(79, 106, 245, 0.35);
  }

  .card.blocked {
    border-color: rgba(248, 113, 113, 0.4);
    background: rgba(42, 21, 24, 0.35);
  }

  .section-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-3) var(--space-4);
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: var(--color-text);
  }

  .section-head:hover {
    background: var(--color-bg-hover);
  }

  .chev {
    display: inline-block;
    font-size: 0.9rem;
    color: var(--color-text-muted);
    transition: transform 0.12s ease;
  }

  .chev.open {
    transform: rotate(90deg);
  }

  .section-head h3 {
    margin: 0;
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .section-body {
    padding: 0 var(--space-4) var(--space-4);
    border-top: 1px solid var(--color-border-subtle);
  }

  .collapsed .section-head {
    border-radius: var(--radius-md);
  }
</style>
