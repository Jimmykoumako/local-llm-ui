<script lang="ts">
  import { marked } from "marked";
  import DOMPurify from "dompurify";
  import { cleanPlainTextForTtsExport } from "$lib/tts-markdown";
  import { copyToClipboard } from "$lib/utils/clipboard";
  import { downloadMarkdown, markdownFilename } from "$lib/utils/download";

  interface Props {
    content: string;
    raw?: boolean;
    ttsEnabled?: boolean;
    onSpeakBlock?: (text: string, blockKey: string) => void;
    onStopSpeak?: () => void;
    speaking?: boolean;
    speakingBlockKey?: string | null;
    allowDownload?: boolean;
  }

  let {
    content,
    raw = false,
    ttsEnabled = false,
    onSpeakBlock,
    onStopSpeak,
    speaking = false,
    speakingBlockKey = null,
    allowDownload = false,
  }: Props = $props();

  type Segment =
    | { kind: "markdown"; text: string }
    | { kind: "fenced"; text: string; key: string; asCode: boolean };

  const CODE_FENCE_LANGS = new Set([
    "bash",
    "c",
    "cpp",
    "css",
    "go",
    "html",
    "java",
    "javascript",
    "js",
    "json",
    "kotlin",
    "php",
    "python",
    "py",
    "rb",
    "ruby",
    "rust",
    "sh",
    "sql",
    "swift",
    "toml",
    "ts",
    "typescript",
    "xml",
    "yaml",
    "yml",
  ]);

  function isCodeFence(lang: string): boolean {
    const normalized = lang.trim().toLowerCase();
    if (!normalized) return false;
    return CODE_FENCE_LANGS.has(normalized);
  }

  marked.use({
    gfm: true,
    breaks: false,
  });

  function renderMarkdown(text: string): string {
    if (!text.trim()) return "";
    const parsed = marked.parse(text, { async: false }) as string;
    return DOMPurify.sanitize(parsed, {
      ADD_TAGS: ["table", "thead", "tbody", "tr", "th", "td"],
    });
  }

  const segments = $derived.by((): Segment[] => {
    if (raw || !content.trim()) return [];

    const result: Segment[] = [];
    const pattern = /```([^\n]*)\n([\s\S]*?)```/g;
    let lastIndex = 0;
    let blockIndex = 0;

    for (const match of content.matchAll(pattern)) {
      const start = match.index ?? 0;
      if (start > lastIndex) {
        result.push({
          kind: "markdown",
          text: content.slice(lastIndex, start),
        });
      }
      const lang = match[1] ?? "";
      const body = match[2]?.trim() ?? "";
      if (body) {
        result.push({
          kind: "fenced",
          text: body,
          key: `block-${blockIndex++}`,
          asCode: isCodeFence(lang),
        });
      }
      lastIndex = start + match[0].length;
    }

    if (lastIndex < content.length) {
      result.push({ kind: "markdown", text: content.slice(lastIndex) });
    }

    if (result.length === 0) {
      result.push({ kind: "markdown", text: content });
    }

    return result;
  });
</script>

{#if raw}
  <pre class="raw">{content}</pre>
{:else if !content.trim()}
  <p class="empty">—</p>
{:else}
  <div class="markdown-root">
    {#each segments as segment (segment.kind === "fenced" ? segment.key : segment.text)}
      {#if segment.kind === "markdown"}
        {@const html = renderMarkdown(segment.text)}
        {#if html}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          <div class="markdown">{@html html}</div>
        {/if}
      {:else if segment.asCode}
        <div class="fenced-block">
          <pre class="fenced-pre"><code>{segment.text}</code></pre>
        </div>
      {:else}
        {@const blockHtml = renderMarkdown(segment.text)}
        {@const blockSpeaking = speakingBlockKey === segment.key}
        <div class="fenced-block prose" class:toolbar-visible={blockSpeaking}>
          <div class="fenced-toolbar">
            <button
              type="button"
              class="block-action-btn"
              onclick={() => void copyToClipboard(segment.text)}
              aria-label="Copy letter"
              title="Copy"
            >
              ⧉
            </button>
            {#if allowDownload}
              <button
                type="button"
                class="block-action-btn"
                onclick={() =>
                  void downloadMarkdown(
                    markdownFilename(
                      segment.text.split("\n").find((l) => l.trim()) ?? "letter",
                    ),
                    segment.text,
                  )}
                aria-label="Download letter as Markdown"
                title="Download .md"
              >
                ↓ .md
              </button>
            {/if}
            {#if ttsEnabled}
              {#if blockSpeaking && onStopSpeak}
                <button
                  type="button"
                  class="block-action-btn stop"
                  onclick={onStopSpeak}
                  aria-label="Stop reading aloud"
                  title="Stop"
                >
                  ■ Stop
                </button>
              {:else if onSpeakBlock}
                <button
                  type="button"
                  class="block-action-btn"
                  onclick={() =>
                    onSpeakBlock(
                      cleanPlainTextForTtsExport(segment.text),
                      segment.key,
                    )}
                  aria-label="Read letter aloud"
                  title="Read aloud"
                >
                  🔊
                </button>
              {/if}
            {/if}
          </div>
          {#if blockHtml}
            <!-- eslint-disable-next-line svelte/no-at-html-tags -->
            <div class="markdown fenced-preview">{@html blockHtml}</div>
          {/if}
        </div>
      {/if}
    {/each}
  </div>
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

  .markdown-root {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .fenced-block {
    margin: 0 0 0.75em;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-md);
    background: var(--color-bg-inset);
    overflow: hidden;
  }

  .fenced-block.prose {
    background: var(--color-bg-panel);
    position: relative;
  }

  .fenced-preview {
    padding: var(--space-3);
  }

  .fenced-toolbar {
    position: absolute;
    top: var(--space-2);
    right: var(--space-2);
    z-index: 1;
    display: flex;
    justify-content: flex-end;
    gap: var(--space-1);
    padding: 2px;
    border-radius: var(--radius-sm);
    background: var(--color-bg-elevated);
    border: 1px solid var(--color-border-subtle);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.18);
    opacity: 0;
    pointer-events: none;
    transition: opacity 0.12s ease;
  }

  .fenced-block.prose:hover .fenced-toolbar,
  .fenced-block.prose:focus-within .fenced-toolbar,
  .fenced-block.prose.toolbar-visible .fenced-toolbar {
    opacity: 1;
    pointer-events: auto;
  }

  .block-action-btn {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    background: var(--color-bg-inset);
    color: var(--color-text-secondary);
    font-size: var(--text-xs);
    padding: 2px 7px;
    line-height: 1.4;
  }

  .block-action-btn:hover:not(:disabled) {
    color: var(--color-text);
    border-color: var(--color-primary);
  }

  .block-action-btn.stop {
    color: var(--color-error);
    border-color: rgba(248, 113, 113, 0.45);
  }

  .fenced-pre {
    margin: 0;
    padding: var(--space-3);
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-word;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    line-height: 1.6;
  }

  .fenced-pre code {
    font-family: inherit;
    background: none;
    padding: 0;
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
  .markdown :global(table) {
    width: 100%;
    margin: 0 0 0.75em;
    border-collapse: collapse;
    font-size: var(--text-sm);
  }
  .markdown :global(th),
  .markdown :global(td) {
    padding: var(--space-2) var(--space-3);
    border: 1px solid var(--color-border-subtle);
    text-align: left;
    vertical-align: top;
  }
  .markdown :global(th) {
    background: var(--color-bg-inset);
    font-weight: 600;
    color: var(--color-text);
  }
  .markdown :global(tr:nth-child(even) td) {
    background: rgba(255, 255, 255, 0.02);
  }
</style>
