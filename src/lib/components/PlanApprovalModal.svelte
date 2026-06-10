<script lang="ts">
  import type { AgentPlan } from "$lib/agent-plan";
  import { planWriteSteps } from "$lib/agent-plan";

  interface Props {
    plan: AgentPlan | null;
    onApprove: () => void;
    onDeny: () => void;
  }

  let { plan, onApprove, onDeny }: Props = $props();

  const writeSteps = $derived(plan ? planWriteSteps(plan) : []);
</script>

{#if plan}
  <div class="backdrop" role="presentation" onclick={onDeny}></div>
  <div class="modal" role="dialog" aria-labelledby="plan-approval-title" aria-modal="true">
    <header class="head">
      <h3 id="plan-approval-title">Run plan?</h3>
      <p class="sub">
        This plan includes {writeSteps.length} step{writeSteps.length === 1 ? "" : "s"}
        that create, edit, or delete files. Approve once to execute the full plan.
      </p>
    </header>

    <ul class="list">
      {#each writeSteps as step (step.id)}
        <li>{step.index}. {step.description}</li>
      {/each}
    </ul>

    <footer class="actions">
      <button type="button" class="btn deny" onclick={onDeny}>Cancel</button>
      <button type="button" class="btn approve" onclick={onApprove}>Run plan</button>
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
    width: min(480px, calc(100vw - 2rem));
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

  .list {
    margin: var(--space-4) 0;
    padding-left: 1.25rem;
    max-height: 200px;
    overflow-y: auto;
    font-size: var(--text-sm);
    color: var(--color-text);
  }

  .list li {
    margin-bottom: var(--space-2);
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .btn {
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .deny {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .approve {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }
</style>
