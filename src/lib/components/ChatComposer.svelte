<script lang="ts">
  import AttachmentStrip from "./AttachmentStrip.svelte";
  import type { ChatMode } from "$lib/conversations";
  import { modeLabel } from "$lib/conversations";
  import type { ModelInfo, PendingAttachment } from "$lib/types";

  interface Props {
    input: string;
    pending: PendingAttachment[];
    models: ModelInfo[];
    selectedModel: string;
    chatMode: ChatMode;
    connected: boolean;
    isStreaming: boolean;
    isPreparingAudio: boolean;
    supportsVision: boolean;
    supportsAudio: boolean;
    supportsTools: boolean;
    hasPendingAudio: boolean;
    onInputChange: (value: string) => void;
    onSend: () => void;
    onRemovePending: (index: number) => void;
    onImageFiles: (files: FileList) => void;
    onAudioFiles: (files: FileList) => void;
    onModelChange: (model: string) => void;
    onModeChange: (mode: ChatMode) => void;
    onOpenModelsView: () => void;
    onOpenSettingsView: () => void;
    onStop?: () => void;
  }

  let {
    input,
    pending,
    models,
    selectedModel,
    chatMode,
    connected,
    isStreaming,
    isPreparingAudio,
    supportsVision,
    supportsAudio,
    supportsTools,
    hasPendingAudio,
    onInputChange,
    onSend,
    onRemovePending,
    onImageFiles,
    onAudioFiles,
    onModelChange,
    onModeChange,
    onOpenModelsView,
    onOpenSettingsView,
    onStop,
  }: Props = $props();

  let plusOpen = $state(false);
  let modelOpen = $state(false);
  let imageInput = $state<HTMLInputElement | null>(null);
  let audioInput = $state<HTMLInputElement | null>(null);
  let textareaEl = $state<HTMLTextAreaElement | null>(null);

  const disabled = $derived(
    !connected || isStreaming || isPreparingAudio,
  );

  function closeMenus() {
    plusOpen = false;
    modelOpen = false;
  }

  function togglePlus() {
    plusOpen = !plusOpen;
    modelOpen = false;
  }

  function toggleModel() {
    modelOpen = !modelOpen;
    plusOpen = false;
  }

  function pickMode(mode: ChatMode) {
    onModeChange(mode);
    closeMenus();
    textareaEl?.focus();
  }

  function pickImage() {
    imageInput?.click();
    closeMenus();
  }

  function pickAudio() {
    audioInput?.click();
    closeMenus();
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      onSend();
    }
    if (event.key === "Escape") closeMenus();
  }

  const MAX_INPUT_LINES = 7;

  function autoResize(node: HTMLTextAreaElement) {
    const resize = () => {
      const style = getComputedStyle(node);
      const lineHeight = parseFloat(style.lineHeight) || 22;
      const maxHeight = lineHeight * MAX_INPUT_LINES;
      node.style.height = "auto";
      const next = Math.min(node.scrollHeight, maxHeight);
      node.style.height = `${next}px`;
      node.style.overflowY = node.scrollHeight > maxHeight ? "auto" : "hidden";
    };
    resize();
    return { update: resize };
  }
</script>

<svelte:window onclick={closeMenus} />

<div class="composer" role="region" aria-label="Message composer" onclick={(e) => e.stopPropagation()}>
  <div class="composer-wrap">
    <AttachmentStrip
      items={pending}
      onRemove={onRemovePending}
      disabled={isStreaming || isPreparingAudio}
    />

    {#if isPreparingAudio}
      <p class="preparing">Converting audio to 16 kHz WAV…</p>
    {/if}

    {#if plusOpen}
      <div class="menu plus-menu" role="menu">
        <button type="button" class="menu-item" onclick={() => pickMode("plan")}>
          <span class="menu-icon">◎</span> Plan
        </button>
        <button type="button" class="menu-item" onclick={() => pickMode("debug")}>
          <span class="menu-icon">⎈</span> Debug
        </button>
        <button type="button" class="menu-item" onclick={() => pickMode("multi")}>
          <span class="menu-icon">⊞</span> Multi-task
        </button>
        <button type="button" class="menu-item" onclick={() => pickMode("ask")}>
          <span class="menu-icon">?</span> Ask
        </button>
        <div class="menu-divider"></div>
        {#if supportsVision}
          <button type="button" class="menu-item" onclick={pickImage}>
            <span class="menu-icon">🖼</span> Image
          </button>
        {:else}
          <button type="button" class="menu-item blocked" disabled title="Switch to a vision-capable model">
            <span class="menu-icon">🖼</span> Image
            <span class="blocked-tag">No vision</span>
          </button>
        {/if}
        {#if supportsAudio}
          <button type="button" class="menu-item" onclick={pickAudio}>
            <span class="menu-icon">🎤</span> Audio
          </button>
        {:else}
          <button type="button" class="menu-item blocked" disabled title="Switch to an audio-capable model">
            <span class="menu-icon">🎤</span> Audio
            <span class="blocked-tag">No audio</span>
          </button>
        {/if}
        <div class="menu-divider"></div>
        <button type="button" class="menu-item" onclick={() => { onOpenModelsView(); closeMenus(); }}>
          <span class="menu-icon">◇</span> Models
          <span class="menu-chevron">›</span>
        </button>
        <button type="button" class="menu-item" disabled title="Coming soon">
          <span class="menu-icon">✦</span> Skills
          <span class="menu-badge">Soon</span>
        </button>
        {#if supportsTools}
          <button type="button" class="menu-item" onclick={() => { onOpenSettingsView(); closeMenus(); }}>
            <span class="menu-icon">⬡</span> Agent / MCP
            <span class="menu-chevron">›</span>
          </button>
        {:else}
          <button type="button" class="menu-item blocked" disabled title="Switch to a tools-capable model">
            <span class="menu-icon">⬡</span> Agent / MCP
            <span class="blocked-tag">No tools</span>
          </button>
        {/if}
      </div>
    {/if}

    <div class="input-shell">
      <button
        type="button"
        class="icon-btn plus-btn"
        onclick={(e) => { e.stopPropagation(); togglePlus(); }}
        aria-label="Add attachment or mode"
        {disabled}
      >+</button>

      <textarea
        bind:this={textareaEl}
        class="input"
        value={input}
        oninput={(e) => onInputChange(e.currentTarget.value)}
        placeholder={hasPendingAudio
          ? "Optional prompt (defaults to “Transcribe this audio”)…"
          : `${modeLabel(chatMode)} the model…`}
        rows="1"
        onkeydown={onKeydown}
        use:autoResize
        {disabled}
      ></textarea>

      <div class="trailing">
        <div class="model-picker">
          <button
            type="button"
            class="model-btn"
            onclick={(e) => { e.stopPropagation(); toggleModel(); }}
            {disabled}
          >
            {selectedModel || "Auto"}
            <span class="chev">▾</span>
          </button>
          {#if modelOpen}
            <div class="menu model-menu" role="listbox">
              {#each models as model}
                <button
                  type="button"
                  class="menu-item"
                  class:active={model.name === selectedModel}
                  onclick={() => { onModelChange(model.name); closeMenus(); }}
                >
                  {model.name}
                </button>
              {/each}
            </div>
          {/if}
        </div>

        {#if supportsAudio}
          <button
            type="button"
            class="icon-btn"
            onclick={pickAudio}
            aria-label="Attach audio"
            {disabled}
          >
            <svg width="18" height="18" viewBox="0 0 18 18" fill="currentColor" aria-hidden="true">
              <path d="M9 1a3 3 0 00-3 3v5a3 3 0 006 0V4a3 3 0 00-3-3zm-5 8a5 5 0 0010 0h2a7 7 0 01-14 0h2zm3 0H6a3 3 0 006 0h-1a2 2 0 11-4 0H7z"/>
            </svg>
          </button>
        {/if}

        {#if isStreaming && onStop}
          <button
            type="button"
            class="send-btn stop-btn"
            onclick={onStop}
            aria-label="Stop generating"
            title="Stop"
          >
            <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor" aria-hidden="true">
              <rect x="2" y="2" width="10" height="10" rx="1"/>
            </svg>
          </button>
        {:else}
          <button
            type="button"
            class="send-btn"
            onclick={onSend}
            aria-label="Send message"
            disabled={disabled}
          >
            <svg width="18" height="18" viewBox="0 0 18 18" fill="none" aria-hidden="true">
              <path d="M9 15V4M9 4L5 8M9 4l4 4" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        {/if}
      </div>
    </div>

    {#if chatMode !== "ask"}
      <p class="mode-hint">Mode: <strong>{modeLabel(chatMode)}</strong></p>
    {/if}
  </div>

  <input
    bind:this={imageInput}
    type="file"
    accept="image/*"
    multiple
    class="hidden"
    onchange={(e) => {
      const files = e.currentTarget.files;
      if (files?.length) onImageFiles(files);
      e.currentTarget.value = "";
    }}
  />
  <input
    bind:this={audioInput}
    type="file"
    accept="audio/*,.mp3,.wav,.ogg,.flac,.m4a,.aac,.webm"
    multiple
    class="hidden"
    onchange={(e) => {
      const files = e.currentTarget.files;
      if (files?.length) onAudioFiles(files);
      e.currentTarget.value = "";
    }}
  />
</div>

<style>
  .composer {
    flex-shrink: 0;
    padding: var(--space-4);
    border-top: 1px solid var(--color-border);
    background: var(--color-bg-elevated);
  }

  .composer-wrap {
    position: relative;
    max-width: calc(var(--thread-max-width) + var(--space-8) * 2);
    margin: 0 auto;
  }

  .preparing {
    margin: 0 0 var(--space-2);
    font-size: var(--text-sm);
    color: var(--color-tool);
  }

  .mode-hint {
    margin: var(--space-2) 0 0;
    font-size: var(--text-xs);
    color: var(--color-text-muted);
  }

  .input-shell {
    display: flex;
    align-items: flex-end;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--color-bg-inset);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
  }

  .input-shell:focus-within {
    border-color: var(--color-primary);
    outline: 1px solid rgba(79, 106, 245, 0.35);
  }

  .plus-btn {
    font-size: 1.25rem;
    font-weight: 300;
    line-height: 1;
  }

  .input {
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    resize: none;
    padding: var(--space-2) 0;
    line-height: 1.5;
    overflow-y: hidden;
  }

  .input:focus {
    outline: none;
  }

  .trailing {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
  }

  .icon-btn {
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    border-radius: var(--radius-full);
    background: transparent;
    color: var(--color-text-secondary);
  }

  .icon-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .model-picker {
    position: relative;
  }

  .model-btn {
    display: flex;
    align-items: center;
    gap: var(--space-1);
    padding: var(--space-1) var(--space-2);
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    max-width: 120px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .model-btn:hover:not(:disabled) {
    background: var(--color-bg-hover);
    color: var(--color-text);
  }

  .chev {
    font-size: 0.65rem;
    opacity: 0.7;
  }

  .send-btn {
    width: 34px;
    height: 34px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    border-radius: var(--radius-full);
    background: var(--color-primary);
    color: white;
    flex-shrink: 0;
  }

  .send-btn:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .stop-btn {
    background: var(--color-error);
  }

  .stop-btn:hover {
    background: #ef4444;
  }

  .menu {
    position: absolute;
    bottom: calc(100% + var(--space-2));
    left: 0;
    min-width: 200px;
    padding: var(--space-2);
    background: var(--color-bg-panel);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    z-index: 50;
  }

  .model-menu {
    left: auto;
    right: 0;
    min-width: 180px;
    max-height: 240px;
    overflow-y: auto;
  }

  .menu-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    border: none;
    border-radius: var(--radius-sm);
    background: transparent;
    font-size: var(--text-sm);
    color: var(--color-text);
    text-align: left;
  }

  .menu-item:hover:not(:disabled) {
    background: var(--color-bg-hover);
  }

  .menu-item.active {
    background: rgba(79, 106, 245, 0.15);
    color: #9ec5ff;
  }

  .menu-item:disabled {
    opacity: 0.55;
  }

  .menu-item.blocked {
    cursor: not-allowed;
  }

  .blocked-tag {
    margin-left: auto;
    font-size: 0.65rem;
    font-weight: 600;
    color: var(--color-error);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .menu-icon {
    width: 18px;
    text-align: center;
    opacity: 0.85;
  }

  .menu-chevron {
    margin-left: auto;
    color: var(--color-text-muted);
  }

  .menu-badge {
    margin-left: auto;
    font-size: 0.65rem;
    color: var(--color-text-muted);
    text-transform: uppercase;
  }

  .menu-divider {
    height: 1px;
    margin: var(--space-1) 0;
    background: var(--color-border-subtle);
  }

  .hidden {
    display: none;
  }
</style>
