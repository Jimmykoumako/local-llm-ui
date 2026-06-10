<script lang="ts">
  import "$lib/styles/global.css";

  let { children } = $props();
</script>

<svelte:head>
  <title>Local LLM UI</title>
</svelte:head>

{@render children()}
