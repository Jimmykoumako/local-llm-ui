<script lang="ts">
  let { children } = $props();
</script>

<svelte:head>
  <title>Local LLM UI</title>
</svelte:head>

{@render children()}

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    font-family:
      Inter,
      system-ui,
      -apple-system,
      BlinkMacSystemFont,
      "Segoe UI",
      sans-serif;
    background: #0f1117;
    color: #e8eaed;
  }
</style>
