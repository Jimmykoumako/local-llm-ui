<script lang="ts">
  import {
    createEmptyPlanStep,
    planWriteSteps,
    reindexPlanSteps,
    stepToolLabel,
    type AgentPlan,
    type PlanStep,
    type PlanStepKind,
  } from "$lib/agent-plan";

  interface Props {
    plan: AgentPlan | null;
    isStreaming: boolean;
    onRun: () => void;
    onDiscard: () => void;
    onPlanChange: (plan: AgentPlan) => void;
  }

  let { plan, isStreaming, onRun, onDiscard, onPlanChange }: Props = $props();

  let expanded = $state(true);

  const writeCount = $derived(plan ? planWriteSteps(plan).length : 0);
  const doneCount = $derived(
    plan?.steps.filter((s) => s.status === "done").length ?? 0,
  );
  const totalSteps = $derived(plan?.steps.length ?? 0);
  const isCompleted = $derived(plan?.status === "completed");
  const isFailed = $derived(plan?.status === "failed");
  const isExecuting = $derived(plan?.status === "executing");
  const isReady = $derived(plan?.status === "ready");
  const editable = $derived(isReady && !isStreaming);

  $effect(() => {
    if (!plan) return;
    if (plan.status === "completed" || plan.status === "failed") {
      expanded = false;
    } else if (plan.status === "ready" || plan.status === "executing") {
      expanded = true;
    }
  });

  function updateStep(stepId: string, patch: Partial<PlanStep>) {
    if (!plan || !editable) return;
    const steps = plan.steps.map((s) =>
      s.id === stepId ? { ...s, ...patch } : s,
    );
    onPlanChange({ ...plan, steps });
  }

  function removeStep(stepId: string) {
    if (!plan || !editable || plan.steps.length <= 1) return;
    const steps = reindexPlanSteps(plan.steps.filter((s) => s.id !== stepId));
    onPlanChange({ ...plan, steps });
  }

  function addStep() {
    if (!plan || !editable) return;
    const steps = reindexPlanSteps([
      ...plan.steps,
      createEmptyPlanStep(plan.steps.length + 1),
    ]);
    onPlanChange({ ...plan, steps });
  }

  function moveStep(stepId: string, direction: -1 | 1) {
    if (!plan || !editable) return;
    const idx = plan.steps.findIndex((s) => s.id === stepId);
    const target = idx + direction;
    if (idx < 0 || target < 0 || target >= plan.steps.length) return;
    const next = [...plan.steps];
    [next[idx], next[target]] = [next[target], next[idx]];
    onPlanChange({ ...plan, steps: reindexPlanSteps(next) });
  }
</script>

{#if plan}
  <div
    class="plan-panel"
    class:collapsed={!expanded}
    class:completed={isCompleted}
    role="region"
    aria-label="Agent plan"
  >
    <button
      type="button"
      class="accordion-head"
      onclick={() => (expanded = !expanded)}
      aria-expanded={expanded}
    >
      <div class="title-wrap">
        <span class="icon">◎</span>
        <div class="title-text">
          <span class="title-row">
            <strong>Plan</strong>
            <span class="status" data-status={plan.status}>{plan.status}</span>
          </span>
          <span class="sub">
            {#if isCompleted}
              {totalSteps} steps · finished
            {:else if isExecuting}
              {doneCount}/{totalSteps} steps done
            {:else}
              {plan.title}
            {/if}
          </span>
        </div>
      </div>
      <span class="chev" class:open={expanded}>▾</span>
    </button>

    {#if expanded}
      <div class="body">
        <ol class="steps">
          {#each plan.steps as step (step.id)}
            <li class="step" data-kind={step.kind} data-status={step.status}>
              <div class="step-head">
                <span class="num">{step.index}</span>
                {#if editable}
                  <select
                    class="kind-select"
                    value={step.kind}
                    onchange={(e) =>
                      updateStep(step.id, {
                        kind: e.currentTarget.value as PlanStepKind,
                      })}
                  >
                    <option value="read">read</option>
                    <option value="write">write</option>
                  </select>
                {:else}
                  <span class="kind">{step.kind}</span>
                {/if}
                <span class="step-status">{step.status}</span>
              </div>

              {#if editable}
                <input
                  class="field"
                  type="text"
                  value={step.description}
                  placeholder="Step description"
                  oninput={(e) =>
                    updateStep(step.id, { description: e.currentTarget.value })}
                />
                <div class="field-row">
                  <input
                    class="field mono"
                    type="text"
                    value={step.tool ?? ""}
                    placeholder="tool e.g. create_file"
                    oninput={(e) =>
                      updateStep(step.id, {
                        tool: e.currentTarget.value || undefined,
                      })}
                  />
                  <input
                    class="field mono"
                    type="text"
                    value={step.path ?? ""}
                    placeholder="path"
                    oninput={(e) =>
                      updateStep(step.id, {
                        path: e.currentTarget.value || undefined,
                      })}
                  />
                </div>
                <div class="step-actions">
                  <button
                    type="button"
                    class="icon-btn"
                    title="Move up"
                    disabled={step.index <= 1}
                    onclick={() => moveStep(step.id, -1)}
                  >↑</button>
                  <button
                    type="button"
                    class="icon-btn"
                    title="Move down"
                    disabled={step.index >= totalSteps}
                    onclick={() => moveStep(step.id, 1)}
                  >↓</button>
                  <button
                    type="button"
                    class="icon-btn danger"
                    title="Remove step"
                    disabled={totalSteps <= 1}
                    onclick={() => removeStep(step.id)}
                  >×</button>
                </div>
              {:else}
                <p class="desc">{step.description}</p>
                <p class="tool-meta">{stepToolLabel(step)}</p>
              {/if}
            </li>
          {/each}
        </ol>

        {#if editable}
          <button type="button" class="add-step" onclick={addStep}>+ Add step</button>
        {/if}

        {#if isReady}
          <p class="hint">
            {#if writeCount > 0}
              {writeCount} step{writeCount === 1 ? "" : "s"} will modify files.
              You'll be asked once before running.
            {:else}
              This plan only uses read operations.
            {/if}
          </p>
          <div class="actions">
            <button type="button" class="btn ghost" onclick={onDiscard} disabled={isStreaming}>
              Discard
            </button>
            <button type="button" class="btn primary" onclick={onRun} disabled={isStreaming}>
              Run plan
            </button>
          </div>
        {:else if isExecuting}
          <p class="hint">Executing step {doneCount + 1} of {totalSteps}…</p>
        {:else if isCompleted}
          <p class="hint success">Plan completed.</p>
          <div class="actions">
            <button type="button" class="btn ghost" onclick={onDiscard}>Clear plan</button>
          </div>
        {:else if isFailed}
          <p class="hint error">Stopped at step {doneCount + 1}. Edit steps and retry.</p>
          <div class="actions">
            <button type="button" class="btn ghost" onclick={onDiscard}>Clear plan</button>
            <button type="button" class="btn primary" onclick={onRun} disabled={isStreaming}>
              Retry from failed step
            </button>
          </div>
        {/if}
      </div>
    {:else if isCompleted || isFailed}
      <div class="collapsed-actions">
        <button type="button" class="btn-link" onclick={onDiscard}>Clear</button>
        {#if isFailed}
          <button type="button" class="btn-link" onclick={onRun} disabled={isStreaming}>
            Retry
          </button>
        {/if}
      </div>
    {/if}
  </div>
{/if}

<style>
  .plan-panel {
    flex-shrink: 0;
    margin: 0 var(--space-4) var(--space-2);
    border: 1px solid rgba(79, 106, 245, 0.25);
    border-radius: var(--radius-lg);
    background: rgba(79, 106, 245, 0.06);
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    width: calc(100% - var(--space-8));
    margin-left: auto;
    margin-right: auto;
    overflow: hidden;
  }

  .plan-panel.completed {
    border-color: rgba(109, 212, 160, 0.2);
    background: rgba(109, 212, 160, 0.04);
  }

  .plan-panel.collapsed {
    background: var(--color-bg-inset);
  }

  .accordion-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-3) var(--space-4);
    border: none;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: inherit;
  }

  .accordion-head:hover {
    background: rgba(255, 255, 255, 0.02);
  }

  .title-wrap {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    min-width: 0;
    flex: 1;
  }

  .icon {
    color: var(--color-primary);
    font-size: 1.1rem;
    line-height: 1;
    margin-top: 2px;
    flex-shrink: 0;
  }

  .completed .icon {
    color: var(--color-success);
  }

  .title-text {
    min-width: 0;
    flex: 1;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .title-row strong {
    font-size: var(--text-sm);
    font-weight: 600;
  }

  .sub {
    display: block;
    margin-top: 2px;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .status {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 2px 6px;
    border-radius: var(--radius-full);
    background: var(--color-bg-panel);
    color: var(--color-text-muted);
  }

  .status[data-status="executing"] {
    color: var(--color-warning);
    background: rgba(251, 191, 36, 0.12);
  }

  .status[data-status="completed"] {
    color: var(--color-success);
    background: rgba(109, 212, 160, 0.12);
  }

  .status[data-status="failed"] {
    color: var(--color-error);
    background: rgba(248, 113, 113, 0.12);
  }

  .chev {
    flex-shrink: 0;
    color: var(--color-text-muted);
    transition: transform 0.15s ease;
    font-size: 0.85rem;
  }

  .chev.open {
    transform: rotate(180deg);
  }

  .body {
    padding: 0 var(--space-4) var(--space-4);
  }

  .steps {
    list-style: none;
    margin: 0 0 var(--space-3);
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    max-height: 320px;
    overflow-y: auto;
  }

  .step {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
    border: 1px solid var(--color-border-subtle);
    font-size: var(--text-sm);
  }

  .step-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .num {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    min-width: 1.25rem;
  }

  .kind,
  .kind-select {
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 2px 6px;
    border-radius: var(--radius-full);
    background: rgba(56, 189, 248, 0.12);
    color: var(--color-tool);
    border: none;
  }

  .step[data-kind="write"] .kind,
  .step[data-kind="write"] .kind-select {
    background: rgba(251, 191, 36, 0.12);
    color: var(--color-warning);
  }

  .step-status {
    margin-left: auto;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    text-transform: capitalize;
  }

  .step[data-status="running"] .step-status { color: var(--color-warning); }
  .step[data-status="done"] .step-status { color: var(--color-success); }
  .step[data-status="failed"] .step-status { color: var(--color-error); }

  .desc {
    margin: 0;
    color: var(--color-text);
    line-height: 1.45;
  }

  .tool-meta {
    margin: 0;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .field {
    width: 100%;
    padding: var(--space-2);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
    background: var(--color-bg-panel);
    color: var(--color-text);
    font-size: var(--text-sm);
  }

  .field.mono {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
  }

  .field-row {
    display: grid;
    grid-template-columns: 1fr 1.4fr;
    gap: var(--space-2);
  }

  .step-actions {
    display: flex;
    gap: var(--space-1);
    justify-content: flex-end;
  }

  .icon-btn {
    width: 28px;
    height: 28px;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-sm);
    background: var(--color-bg-panel);
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .icon-btn.danger:hover:not(:disabled) {
    color: var(--color-error);
    border-color: rgba(248, 113, 113, 0.4);
  }

  .add-step {
    width: 100%;
    margin-bottom: var(--space-3);
    padding: var(--space-2);
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--color-text-muted);
    font-size: var(--text-sm);
  }

  .add-step:hover {
    border-color: var(--color-primary);
    color: var(--color-text);
  }

  .hint {
    margin: 0 0 var(--space-3);
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .hint.success { color: var(--color-success); }
  .hint.error { color: var(--color-error); }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-2);
  }

  .collapsed-actions {
    display: flex;
    justify-content: flex-end;
    gap: var(--space-3);
    padding: 0 var(--space-4) var(--space-2);
  }

  .btn-link {
    border: none;
    background: none;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    padding: 0;
  }

  .btn-link:hover:not(:disabled) {
    color: var(--color-text);
  }

  .btn {
    padding: var(--space-2) var(--space-4);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    font-size: var(--text-sm);
    font-weight: 500;
  }

  .btn.ghost {
    background: transparent;
    color: var(--color-text-secondary);
  }

  .btn.ghost:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .btn.primary {
    background: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .btn.primary:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .btn:disabled,
  .btn-link:disabled,
  .icon-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
