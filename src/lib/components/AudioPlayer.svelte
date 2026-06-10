<script lang="ts">
  import { formatDuration } from "$lib/utils/audio";

  interface Props {
    src: string;
    label?: string;
    compact?: boolean;
  }

  let { src, label = "Audio", compact = false }: Props = $props();

  let audioEl = $state<HTMLAudioElement | null>(null);
  let playing = $state(false);
  let currentTime = $state(0);
  let duration = $state(0);

  function togglePlay() {
    if (!audioEl) return;
    if (playing) {
      audioEl.pause();
    } else {
      void audioEl.play();
    }
  }

  function onTimeUpdate() {
    if (audioEl) currentTime = audioEl.currentTime;
  }

  function onLoadedMetadata() {
    if (audioEl) duration = audioEl.duration;
  }

  function onEnded() {
    playing = false;
  }

  function onPlay() {
    playing = true;
  }

  function onPause() {
    playing = false;
  }

  function onSeek(event: Event) {
    const target = event.target as HTMLInputElement;
    const t = Number(target.value);
    if (audioEl) {
      audioEl.currentTime = t;
      currentTime = t;
    }
  }
</script>

<div class="player" class:compact>
  <!-- svelte-ignore a11y_media_has_caption -->
  <audio
    bind:this={audioEl}
    {src}
    ontimeupdate={onTimeUpdate}
    onloadedmetadata={onLoadedMetadata}
    onended={onEnded}
    onplay={onPlay}
    onpause={onPause}
    preload="metadata"
  ></audio>

  <button
    type="button"
    class="play-btn"
    onclick={togglePlay}
    aria-label={playing ? "Pause" : "Play"}
  >
    {#if playing}
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <rect x="3" y="2" width="4" height="12" rx="1" />
        <rect x="9" y="2" width="4" height="12" rx="1" />
      </svg>
    {:else}
      <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
        <path d="M4 2.5v11l9-5.5-9-5.5z" />
      </svg>
    {/if}
  </button>

  <div class="track">
    {#if !compact}
      <span class="label" title={label}>{label}</span>
    {/if}
    <input
      type="range"
      class="seek"
      min="0"
      max={duration || 0}
      step="0.1"
      value={currentTime}
      oninput={onSeek}
      aria-label="Seek"
    />
    <div class="times">
      <span>{formatDuration(currentTime)}</span>
      <span class="sep">/</span>
      <span>{formatDuration(duration)}</span>
    </div>
  </div>
</div>

<style>
  .player {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3);
    background: var(--color-bg-inset);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    min-width: 220px;
    max-width: 100%;
  }

  .player.compact {
    min-width: 180px;
    padding: var(--space-2) var(--space-3);
  }

  .play-btn {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border: none;
    border-radius: var(--radius-full);
    background: var(--color-primary);
    color: white;
  }

  .play-btn:hover:not(:disabled) {
    background: var(--color-primary-hover);
  }

  .track {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .label {
    font-size: var(--text-xs);
    color: var(--color-text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .seek {
    width: 100%;
    height: 4px;
    accent-color: var(--color-primary);
    cursor: pointer;
  }

  .times {
    display: flex;
    gap: var(--space-1);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
    color: var(--color-text-muted);
  }

  .sep {
    opacity: 0.5;
  }
</style>
