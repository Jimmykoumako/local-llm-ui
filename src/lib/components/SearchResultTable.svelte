<script lang="ts">
  import MarkdownContent from "./MarkdownContent.svelte";
  import {
    parseSearchContentResult,
    parseSearchFilesResult,
    searchContentToMarkdown,
    searchFilesToMarkdown,
  } from "$lib/utils/tool-results";

  interface Props {
    toolName: "search_files" | "search_content";
    result: string;
  }

  let { toolName, result }: Props = $props();

  const markdown = $derived.by(() => {
    if (toolName === "search_content") {
      const parsed = parseSearchContentResult(result);
      return parsed ? searchContentToMarkdown(parsed.hits) : "";
    }
    const parsed = parseSearchFilesResult(result);
    return parsed ? searchFilesToMarkdown(parsed.matches) : "";
  });
</script>

{#if markdown}
  <MarkdownContent content={markdown} />
{/if}
