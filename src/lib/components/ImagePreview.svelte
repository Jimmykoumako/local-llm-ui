<script lang="ts">
  interface Props {
    src: string;
    alt?: string;
    size?: "sm" | "md" | "lg";
  }

  let { src, alt = "Image", size = "md" }: Props = $props();

  let open = $state(false);

  const sizes = { sm: 80, md: 120, lg: 200 };
  const px = $derived(sizes[size]);

  function openPreview() {
    open = true;
  }

  function closePreview() {
    open = false;
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") closePreview();
  }
</script>

<svelte:window onkeydown={onKeydown} />

<button type="button" class="thumb" style="--size: {px}px" onclick={openPreview}>
  <img {src} {alt} />
  <span class="overlay">Preview</span>
</button>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="lightbox" onclick={closePreview} role="presentation">
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="lightbox-inner" onclick={(e) => e.stopPropagation()} role="presentation">
      <button type="button" class="close" onclick={closePreview} aria-label="Close">×</button>
      <img {src} {alt} class="full" />
      {#if alt}
        <p class="caption">{alt}</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .thumb {
    position: relative;
    padding: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-bg-inset);
    cursor: zoom-in;
    width: var(--size);
    height: var(--size);
  }

  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.45);
    color: white;
    font-size: var(--text-xs);
    opacity: 0;
    transition: opacity 0.15s;
  }

  .thumb:hover .overlay {
    opacity: 1;
  }

  .lightbox {
    position: fixed;
    inset: 0;
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.85);
    padding: var(--space-6);
  }

  .lightbox-inner {
    position: relative;
    max-width: min(90vw, 900px);
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
  }

  .full {
    max-width: 100%;
    max-height: calc(90vh - 48px);
    object-fit: contain;
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
  }

  .close {
    position: absolute;
    top: -40px;
    right: 0;
    width: 32px;
    height: 32px;
    border: none;
    border-radius: var(--radius-full);
    background: var(--color-bg-panel);
    color: var(--color-text);
    font-size: 1.25rem;
    line-height: 1;
  }

  .caption {
    margin: 0;
    font-size: var(--text-sm);
    color: var(--color-text-secondary);
    text-align: center;
  }
</style>
