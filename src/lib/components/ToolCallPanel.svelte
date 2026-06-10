<script lang="ts">
  import type { ToolCallDisplay } from "$lib/types";
  import { formatToolArguments } from "$lib/utils/tools";

  interface Props {
    tools: ToolCallDisplay[];
    streaming?: boolean;
  }

  let { tools, streaming = false }: Props = $props();
</script>

<div class="panel">
  <header class="header">
    <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
      <path d="M2 4h12v1.5H2V4zm0 3.5h8v1.5H2V7.5zm0 3.5h10v1.5H2V11z" />
    </svg>
    <span>Tool calls</span>
    {#if streaming}
      <span class="live">running…</span>
    {/if}
  </header>

  <ul class="list">
    {#each tools as tool, i (tool.id ?? `${tool.name}-${i}`)}
      <li class="item">
        <div class="item-head">
          <code class="name">{tool.name}</code>
          {#if tool.source === "mcp"}
            <span class="badge mcp">MCP</span>
          {:else}
            <span class="badge ollama">Ollama</span>
          {/if}
          <span class="status" data-status={tool.status ?? "requested"}>
            {tool.status ?? "requested"}
          </span>
        </div>
        <pre class="args">{formatToolArguments(tool.arguments)}</pre>
        {#if tool.result}
          <div class="result">
            <span class="result-label">Result</span>
            <pre>{tool.result}</pre>
          </div>
        {/if}
      </li>
    {/each}
  </ul>
</div>

<style>
  .panel {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-bg-inset);
    overflow: hidden;
  }

  .header {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: rgba(56, 189, 248, 0.08);
    border-bottom: 1px solid var(--color-border-subtle);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--color-tool);
  }

  .live {
    margin-left: auto;
    font-size: var(--text-xs);
    font-weight: 400;
    color: var(--color-warning);
    animation: pulse 1.5s ease infinite;
  }

  @keyframes pulse {
    50% { opacity: 0.5; }
  }

  .list {
    list-style: none;
    margin: 0;
    padding: var(--space-2);
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .item {
    padding: var(--space-3);
    border-radius: var(--radius-sm);
    background: var(--color-bg-panel);
    border: 1px solid var(--color-border-subtle);
  }

  .item-head {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: var(--space-2);
  }

  .name {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--color-text);
  }

  .badge {
    font-size: 0.65rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 2px 6px;
    border-radius: var(--radius-full);
  }

  .badge.mcp {
    background: rgba(244, 114, 182, 0.15);
    color: var(--color-mcp);
  }

  .badge.ollama {
    background: rgba(56, 189, 248, 0.12);
    color: var(--color-tool);
  }

  .status {
    margin-left: auto;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
    text-transform: capitalize;
  }

  .status[data-status="running"] { color: var(--color-warning); }
  .status[data-status="completed"] { color: var(--color-success); }
  .status[data-status="failed"] { color: var(--color-error); }

  .args,
  .result pre {
    margin: 0;
    padding: var(--space-2);
    background: var(--color-bg);
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--color-text-secondary);
    max-height: 160px;
    overflow-y: auto;
  }

  .result {
    margin-top: var(--space-2);
  }

  .result-label {
    display: block;
    font-size: var(--text-xs);
    color: var(--color-success);
    margin-bottom: var(--space-1);
  }
</style>
