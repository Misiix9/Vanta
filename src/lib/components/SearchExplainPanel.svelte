<script lang="ts">
  import type { SearchConfig, SearchResult } from "$lib/types";

  let {
    query,
    results,
    searchConfig,
    maxItems = 5,
  }: {
    query: string;
    results: SearchResult[];
    searchConfig: SearchConfig;
    maxItems?: number;
  } = $props();

  const visible = $derived.by(() => {
    const trimmed = query.trim();
    if (!trimmed) return [] as SearchResult[];
    return [...results].slice(0, maxItems);
  });

  function sourceWeight(source: SearchResult["source"]): number {
    if (source === "Application") return searchConfig.applications.weight;
    if (source === "Window") return searchConfig.windows.weight;
    if (source === "Calculator") return searchConfig.calculator.weight;
    if (source === "File") return searchConfig.files.weight;
    return 100;
  }

  function sourceLabel(source: SearchResult["source"]): string {
    if (typeof source === "object") return `Extension:${source.Extension.ext_id}`;
    return source;
  }

  function reason(result: SearchResult): string {
    const matched = result.match_indices?.length ?? 0;
    const parts = [
      `source=${sourceLabel(result.source)}`,
      `weight=${sourceWeight(result.source)}%`,
      `score=${Math.round(result.score)}`,
    ];

    if (matched > 0) {
      parts.push(`matched_chars=${matched}`);
    }

    if (result.section) {
      parts.push(`section=${result.section}`);
    }

    return parts.join(" · ");
  }
</script>

{#if visible.length > 0}
  <section class="search-explain" aria-label="Search ranking explainability">
    <div class="search-explain-header">
      <h4>Why These Results?</h4>
      <span class="search-explain-meta">{visible.length} shown</span>
    </div>
    <div class="search-explain-list">
      {#each visible as result, i}
        <div class="search-explain-row">
          <span class="search-explain-rank">#{i + 1}</span>
          <span class="search-explain-title">{result.title}</span>
          <span class="search-explain-reason">{reason(result)}</span>
        </div>
      {/each}
    </div>
  </section>
{/if}
