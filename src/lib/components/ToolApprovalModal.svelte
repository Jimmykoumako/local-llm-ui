<script lang="ts">
  import { canRememberApproval } from "$lib/agent-tools";
  import type { ToolCallDisplay } from "$lib/types";
  import { formatToolArguments } from "$lib/utils/tools";

  interface Props {
    tool: ToolCallDisplay | null;
    onApprove: (remember: boolean) => void;
    onDeny: () => void;
  }

  let { tool, onApprove, onDeny }: Props = $props();
  let remember = $state(false);

  const showRemember = $derived(tool ? canRememberApproval(tool) : false);
  const title = $derived(
    tool?.name === "run_terminal"
      ? "Approve terminal command?"
      : tool?.name.startsWith("git_")
        ? "Approve git operation?"
        : "Approve file operation?",
  );
</script>

{#if tool}
  <div class="backdrop" role="presentation" onclick={onDeny}></div>
  <div class="modal" role="dialog" aria-labelledby="approval-title" aria-modal="true">
    <header class="head">
      <h3 id="approval-title">{title}</h3>
      <p class="sub">The model wants to run a potentially destructive action.</p>
    </header>

    <div class="body">
      <code class="tool-name">{tool.name}</code>
      <pre class="args">{formatToolArguments(tool.arguments)}</pre>

      {#if showRemember}
        <label class="remember">
          <input type="checkbox" bind:checked={remember} />
          Always allow for this file or repo
        </label>
      {/if}
    </div>

    <footer class="actions">
      <button type="button" class="btn deny" onclick={onDeny}>Deny</button>
      <button type="button" class="btn approve" onclick={() => onApprove(remember)}>
        Allow
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
    width: min(480px, calc(100vw - var(--space-8)));
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-bg-elevated);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.35);
    overflow: hidden;
  }

  .head {
    padding: var(--space-4) var(--space-5);
    border-bottom: 1px solid var(--color-border-subtle);
  }

  .head h3 {
    margin: 0 0 var(--space-1);
    font-size: var(--text-base);
  }

  .sub {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
  }

  .body {
    padding: var(--space-4) var(--space-5);
  }

  .tool-name {
    display: inline-block;
    margin-bottom: var(--space-2);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--color-warning);
  }

  .args {
    margin: 0;
    padding: var(--space-3);
    background: var(--color-bg-inset);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: pre-wrap;
    word-break: break-word;
    max-height: 200px;
    overflow-y: auto;
    color: var(--color-text-secondary);
  }

  .remember {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-3);
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    cursor: pointer;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
    padding: var(--space-3) var(--space-5) var(--space-4);
  }

  .btn {
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-sm);
    font-size: var(--text-sm);
    border: 1px solid var(--color-border);
  }

  .deny {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .approve {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: #fff;
  }
</style>
