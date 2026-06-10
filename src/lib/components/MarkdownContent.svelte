<script lang="ts">
  import { marked } from "marked";
  import DOMPurify from "dompurify";

  interface Props {
    content: string;
    raw?: boolean;
  }

  let { content, raw = false }: Props = $props();

  const html = $derived.by(() => {
    if (raw || !content.trim()) return "";
    return DOMPurify.sanitize(
      marked.parse(content, { async: false }) as string,
    );
  });
</script>

{#if raw}
  <pre class="raw">{content}</pre>
{:else if html}
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  <div class="markdown">{@html html}</div>
{:else}
  <p class="empty">—</p>
{/if}

<style>
  .raw {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .empty {
    margin: 0;
    color: var(--color-text-muted);
  }

  .markdown :global(p) {
    margin: 0 0 0.75em;
  }
  .markdown :global(p:last-child) {
    margin-bottom: 0;
  }
  .markdown :global(h1),
  .markdown :global(h2),
  .markdown :global(h3) {
    margin: 1em 0 0.5em;
    font-size: 1em;
    font-weight: 600;
  }
  .markdown :global(ul),
  .markdown :global(ol) {
    margin: 0 0 0.75em;
    padding-left: 1.25em;
  }
  .markdown :global(code) {
    font-family: var(--font-mono);
    font-size: 0.9em;
    background: var(--color-bg-inset);
    padding: 0.1em 0.35em;
    border-radius: var(--radius-sm);
  }
  .markdown :global(pre) {
    margin: 0 0 0.75em;
    padding: var(--space-3);
    background: var(--color-bg-inset);
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-md);
    overflow-x: auto;
  }
  .markdown :global(pre code) {
    padding: 0;
    background: none;
  }
  .markdown :global(a) {
    color: var(--color-primary);
  }
  .markdown :global(blockquote) {
    margin: 0 0 0.75em;
    padding-left: var(--space-3);
    border-left: 3px solid var(--color-border);
    color: var(--color-text-secondary);
  }
</style>
