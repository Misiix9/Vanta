<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import type { SearchResult } from "$lib/types";

  interface FilePreview {
    path: string;
    name: string;
    size_bytes: number;
    modified: number | null;
    mime_hint: string;
    content: string | null;
    is_image: boolean;
    is_directory: boolean;
  }

  let {
    result,
    onClose,
  }: {
    result: SearchResult;
    onClose: () => void;
  } = $props();

  let preview = $state<FilePreview | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  function getFilePath(result: SearchResult): string | null {
    if (result.source === "File") {
      // exec is usually "open:/path/to/file" or just the path
      const exec = result.exec;
      if (exec.startsWith("open:")) return exec.slice(5);
      if (exec.startsWith("xdg-open ")) return exec.slice(9).replace(/^"|"$/g, "");
      if (exec.startsWith("/")) return exec;
      // subtitle often contains the path
      if (result.subtitle?.startsWith("/")) return result.subtitle;
    }
    if (result.subtitle?.startsWith("/")) return result.subtitle;
    return null;
  }

  $effect(() => {
    loading = true;
    error = null;
    preview = null;

    const filePath = getFilePath(result);
    if (filePath) {
      invoke<FilePreview>("preview_file", { path: filePath })
        .then((p) => { preview = p; loading = false; })
        .catch((e) => { error = String(e); loading = false; });
    } else {
      // Non-file result — show basic info
      loading = false;
    }
  });

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatDate(epoch: number | null): string {
    if (!epoch) return "Unknown";
    return new Date(epoch * 1000).toLocaleString();
  }

  function sourceLabel(source: SearchResult["source"]): string {
    if (typeof source === "object" && "Extension" in source) return `Extension: ${source.Extension.ext_id}`;
    return String(source);
  }
</script>

<div class="preview-panel" role="dialog" aria-label="Quick Look preview">
  <div class="preview-header">
    <h3 class="preview-title">{result.title}</h3>
    <button class="preview-close" onclick={onClose} aria-label="Close preview">✕</button>
  </div>

  <div class="preview-body">
    {#if loading}
      <div class="preview-loading">
        <span class="preview-spinner"></span>
        <span>Loading preview…</span>
      </div>
    {:else if error}
      <div class="preview-error">{error}</div>
    {:else if preview}
      <div class="preview-meta">
        <div class="preview-meta-row">
          <span class="preview-meta-label">Name</span>
          <span class="preview-meta-value">{preview.name}</span>
        </div>
        <div class="preview-meta-row">
          <span class="preview-meta-label">Size</span>
          <span class="preview-meta-value">{formatSize(preview.size_bytes)}</span>
        </div>
        <div class="preview-meta-row">
          <span class="preview-meta-label">Modified</span>
          <span class="preview-meta-value">{formatDate(preview.modified)}</span>
        </div>
        <div class="preview-meta-row">
          <span class="preview-meta-label">Type</span>
          <span class="preview-meta-value">{preview.mime_hint}</span>
        </div>
      </div>

      {#if preview.is_image}
        <div class="preview-image-wrap">
          <img src={convertFileSrc(preview.path)} alt={preview.name} class="preview-image" />
        </div>
      {:else if preview.content}
        <pre class="preview-content">{preview.content}</pre>
      {/if}
    {:else}
      <!-- Non-file result: show what we have -->
      <div class="preview-meta">
        <div class="preview-meta-row">
          <span class="preview-meta-label">Source</span>
          <span class="preview-meta-value">{sourceLabel(result.source)}</span>
        </div>
        {#if result.subtitle}
          <div class="preview-meta-row">
            <span class="preview-meta-label">Details</span>
            <span class="preview-meta-value">{result.subtitle}</span>
          </div>
        {/if}
        <div class="preview-meta-row">
          <span class="preview-meta-label">Score</span>
          <span class="preview-meta-value">{Math.round(result.score)}</span>
        </div>
        {#if result.exec}
          <div class="preview-meta-row">
            <span class="preview-meta-label">Action</span>
            <span class="preview-meta-value preview-exec">{result.exec}</span>
          </div>
        {/if}
      </div>

      {#if result.source === "Clipboard"}
        <pre class="preview-content">{result.subtitle || result.title}</pre>
      {/if}
    {/if}
  </div>
</div>
