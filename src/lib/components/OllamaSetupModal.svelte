<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";

  interface Props {
    open: boolean;
    host: string;
    error: string;
    busy: boolean;
    onHostChange: (value: string) => void;
    onRetry: () => void | Promise<void>;
    onOpenSettings: () => void;
    onDismiss: () => void;
  }

  let {
    open,
    host,
    error,
    busy,
    onHostChange,
    onRetry,
    onOpenSettings,
    onDismiss,
  }: Props = $props();

  async function openOllamaDownload() {
    await openUrl("https://ollama.com/download");
  }
</script>

{#if open}
  <div class="backdrop" role="presentation" onclick={onDismiss}></div>
  <div class="modal" role="dialog" aria-labelledby="ollama-setup-title" aria-modal="true">
    <header class="head">
      <h3 id="ollama-setup-title">Connect to Ollama</h3>
      <p class="sub">
        Install and run Ollama on your computer, then point this app at its API URL.
        The default is <code>http://127.0.0.1:11434</code>.
      </p>
    </header>

    <label class="field">
      <span>Ollama URL</span>
      <input
        type="url"
        value={host}
        placeholder="http://127.0.0.1:11434"
        disabled={busy}
        oninput={(e) => onHostChange(e.currentTarget.value)}
      />
    </label>

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <footer class="actions">
      <button type="button" class="btn ghost" onclick={onDismiss} disabled={busy}>
        Later
      </button>
      <button type="button" class="btn ghost" onclick={openOllamaDownload} disabled={busy}>
        Get Ollama
      </button>
      <button type="button" class="btn ghost" onclick={onOpenSettings} disabled={busy}>
        Settings
      </button>
      <button type="button" class="btn primary" onclick={onRetry} disabled={busy}>
        {busy ? "Connecting…" : "Retry"}
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

  .field {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
    margin-top: var(--space-4);
    font-size: var(--text-sm);
  }

  .field input {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .error {
    margin: var(--space-3) 0 0;
    font-size: var(--text-sm);
    color: var(--color-danger, #c0392b);
    line-height: 1.4;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
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
