<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type {
    CommunityFeedbackSummary,
    CommunityFeedbackReceipt,
    CommunityImportConflictStrategy,
    CommunityImportHistoryEntry,
    CommunitySnippetPreview,
    PopularWorkflowFeedEntry,
  } from "$lib/types";

  let {
    communityFeedOptIn = false,
    onToggleFeedOptIn,
    onClose,
  }: {
    communityFeedOptIn: boolean;
    onToggleFeedOptIn: (next: boolean) => void;
    onClose: () => void;
  } = $props();

  let summary: CommunityFeedbackSummary | null = $state(null);
  let popularFeed: PopularWorkflowFeedEntry[] = $state([]);
  let busy = $state(false);
  let message = $state("");
  let contact = $state("");
  let roadmapTopic = $state("popular-workflows-feed");
  let snippetPayload = $state("");
  let snippetKind = $state<"workflow" | "profile" | "theme">("workflow");
  let snippetTargetId = $state("");
  let snippetConflictStrategy = $state<CommunityImportConflictStrategy>("replace");
  let snippetPreview: CommunitySnippetPreview | null = $state(null);
  let snippetPreviewSource = $state("");
  let importHistory: CommunityImportHistoryEntry[] = $state([]);
  let status = $state<string | null>(null);

  const topics = [
    { id: "shareable-snippets", label: "Shareable snippets" },
    { id: "popular-workflows-feed", label: "Popular workflows feed" },
    { id: "in-app-feedback-channel", label: "In-app feedback" },
    { id: "adaptive-intent-engine", label: "Adaptive intent engine" },
  ];

  function formatTimestamp(ms: number): string {
    const value = new Date(ms);
    return Number.isNaN(value.getTime()) ? String(ms) : value.toLocaleString();
  }

  async function refreshImportHistory() {
    importHistory = await invoke<CommunityImportHistoryEntry[]>("get_community_import_history");
  }

  async function refresh() {
    try {
      summary = await invoke<CommunityFeedbackSummary>("get_community_feedback_summary");
      if (communityFeedOptIn) {
        popularFeed = await invoke<PopularWorkflowFeedEntry[]>("get_popular_workflows_feed");
      } else {
        popularFeed = [];
      }
      await refreshImportHistory();
    } catch (e) {
      status = `Failed to refresh: ${String(e)}`;
    }
  }

  async function submitFeedback() {
    if (!message.trim()) {
      status = "Feedback message is required.";
      return;
    }
    busy = true;
    status = null;
    try {
      const receipt = await invoke<CommunityFeedbackReceipt>("submit_community_feedback", {
        message,
        contact: contact.trim() || null,
        roadmapTopic: roadmapTopic || null,
      });
      message = "";
      status = `Feedback received (${receipt.id}).`;
      await refresh();
    } catch (e) {
      status = `Submit failed: ${String(e)}`;
    } finally {
      busy = false;
    }
  }

  async function vote(topic: string) {
    busy = true;
    status = null;
    try {
      await invoke("vote_roadmap_item", { topic });
      roadmapTopic = topic;
      status = `Vote recorded for ${topic}.`;
      await refresh();
    } catch (e) {
      status = `Vote failed: ${String(e)}`;
    } finally {
      busy = false;
    }
  }

  async function installPopular(id: string) {
    busy = true;
    status = null;
    try {
      await invoke("install_popular_workflow", { workflowId: id });
      status = `Installed '${id}' into your workflows.`;
    } catch (e) {
      status = `Install failed: ${String(e)}`;
    } finally {
      busy = false;
    }
  }

  async function exportSnippet() {
    status = null;
    try {
      snippetPayload = await invoke<string>("export_community_snippet", {
        kind: snippetKind,
        targetId: snippetKind === "theme" ? null : (snippetTargetId.trim() || null),
      });
      snippetPreview = null;
      snippetPreviewSource = "";
      status = `Exported ${snippetKind} snippet.`;
    } catch (e) {
      status = `Export failed: ${String(e)}`;
    }
  }

  async function previewSnippet() {
    if (!snippetPayload.trim()) {
      status = "Paste snippet payload first.";
      snippetPreview = null;
      snippetPreviewSource = "";
      return null;
    }

    status = null;
    try {
      const preview = await invoke<CommunitySnippetPreview>("preview_community_snippet", {
        snippetJson: snippetPayload,
      });
      snippetPreview = preview;
      snippetPreviewSource = snippetPayload.trim();
      status = preview.compatible
        ? `Preview ready: ${preview.kind} '${preview.name}' is importable.`
        : `Preview blocked: ${preview.blockers.join(" ")}`;
      return preview;
    } catch (e) {
      snippetPreview = null;
      snippetPreviewSource = "";
      status = `Preview failed: ${String(e)}`;
      return null;
    }
  }

  async function importSnippet() {
    if (!snippetPayload.trim()) {
      status = "Paste snippet payload first.";
      return;
    }

    const preview =
      snippetPreview && snippetPreviewSource === snippetPayload.trim()
        ? snippetPreview
        : await previewSnippet();
    if (!preview) {
      return;
    }
    if (!preview.compatible) {
      status = `Import blocked: ${preview.blockers.join(" ")}`;
      return;
    }

    status = null;
    try {
      const msg = await invoke<string>("import_community_snippet", {
        snippetJson: snippetPayload,
        onConflict: snippetConflictStrategy,
      });
      snippetPreview = null;
      snippetPreviewSource = "";
      await refreshImportHistory();
      status = msg;
    } catch (e) {
      status = `Import failed: ${String(e)}`;
    }
  }

  async function rollbackImport(importId?: string) {
    busy = true;
    status = null;
    try {
      const msg = await invoke<string>("rollback_community_import", {
        importId: importId || null,
      });
      await refresh();
      status = msg;
    } catch (e) {
      status = `Rollback failed: ${String(e)}`;
    } finally {
      busy = false;
    }
  }

  $effect(() => {
    refresh();
  });
</script>

<div class="v2-shell v2-fill hub-window-root">
  <div class="v2-header hub-window-header">
    <div class="window-header-main">
      <span class="window-breadcrumb">Feature Hub</span>
      <h2>Community Hub</h2>
    </div>
    <div class="window-actions">
      <button class="link-btn btn-ghost" onclick={onClose}>Back</button>
      <button class="link-btn btn-secondary" onclick={refresh}>Refresh</button>
    </div>
  </div>

  <div class="v2-scroll window-scroll">
    <div class="control-group">
      <label>
        Popular Workflows Feed (Opt-in)
        <input
          type="checkbox"
          checked={communityFeedOptIn}
          onchange={(e) => onToggleFeedOptIn((e.target as HTMLInputElement).checked)}
        />
      </label>
    </div>

    <div class="control-group control-group-block">
      <h3>Feedback And Votes</h3>
      <label>
        Feedback Message
        <textarea rows="4" bind:value={message} placeholder="What should be improved?"></textarea>
      </label>
      <label>
        Contact (optional)
        <input type="text" bind:value={contact} placeholder="email or handle" />
      </label>
      <label>
        Roadmap Topic
        <select class="vanta-select" bind:value={roadmapTopic}>
          {#each topics as t}
            <option value={t.id}>{t.label}</option>
          {/each}
        </select>
      </label>
      <div class="preset-row">
        <button class="preset-btn" disabled={busy} onclick={submitFeedback}>Submit Feedback</button>
        {#each topics as t}
          <button class="preset-btn" disabled={busy} onclick={() => vote(t.id)}>Vote: {t.label}</button>
        {/each}
      </div>
      {#if summary}
        <div class="status-info">Total feedback: {summary.total_feedback}</div>
      {/if}
    </div>

    <div class="control-group control-group-block">
      <h3>Shareable Snippets</h3>
      <div class="preset-row">
        <button
          class="preset-btn"
          onclick={() => {
            snippetKind = "workflow";
            snippetPreview = null;
            snippetPreviewSource = "";
          }}>Workflow</button
        >
        <button
          class="preset-btn"
          onclick={() => {
            snippetKind = "profile";
            snippetPreview = null;
            snippetPreviewSource = "";
          }}>Profile</button
        >
        <button
          class="preset-btn"
          onclick={() => {
            snippetKind = "theme";
            snippetPreview = null;
            snippetPreviewSource = "";
          }}>Theme</button
        >
      </div>
      {#if snippetKind !== "theme"}
        <label>
          Target ID
          <input type="text" bind:value={snippetTargetId} placeholder="workflow/profile id" />
        </label>
      {/if}
      <div class="preset-row">
        <button class="preset-btn" onclick={exportSnippet}>Export</button>
        <button class="preset-btn" onclick={previewSnippet}>Preview</button>
        <button class="preset-btn" onclick={importSnippet}>Import</button>
      </div>
      <label>
        Snippet Payload
        <textarea rows="8" bind:value={snippetPayload}></textarea>
      </label>
      {#if snippetPreview}
        <div class="status-info">
          <div>
            Preview: {snippetPreview.kind} "{snippetPreview.name}" ({snippetPreview.compatible
              ? "compatible"
              : "blocked"})
          </div>
          {#if snippetPreview.target_id}
            <div>Target: {snippetPreview.target_id}</div>
          {/if}
          <div>
            Trust: {snippetPreview.trust.publisher} / {snippetPreview.trust.risk_level} / {snippetPreview
              .trust.verified
              ? "verified"
              : "unverified"}
          </div>
          <div>Source: {snippetPreview.trust.source}</div>
          {#if snippetPreview.trust.compatibility_hint}
            <div>Compatibility: {snippetPreview.trust.compatibility_hint}</div>
          {/if}
          {#if snippetPreview.trust.publisher_url}
            <div>Publisher: {snippetPreview.trust.publisher_url}</div>
          {/if}
          {#if snippetPreview.badges.length > 0}
            <div>Badges: {snippetPreview.badges.join(", ")}</div>
          {/if}
          {#if snippetPreview.warnings.length > 0}
            <div>Warnings: {snippetPreview.warnings.join(" | ")}</div>
          {/if}
          {#if snippetPreview.blockers.length > 0}
            <div>Blockers: {snippetPreview.blockers.join(" | ")}</div>
          {/if}
          {#if snippetPreview.will_replace_existing}
            <div class="control-group">
              <label>
                Conflict Strategy
                <select class="vanta-select" bind:value={snippetConflictStrategy}>
                  <option value="replace">Replace local artifact</option>
                  <option value="keep_local">Keep local and skip import</option>
                </select>
              </label>
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="control-group control-group-block">
      <h3>Import Rollback</h3>
      <div class="preset-row">
        <button class="preset-btn" disabled={busy || importHistory.length === 0} onclick={() => rollbackImport()}>
          Undo Latest Import
        </button>
      </div>
      {#if importHistory.length === 0}
        <div class="status-info">No import history yet.</div>
      {:else}
        <ul class="hint-list">
          {#each importHistory as item}
            <li>
              <strong>{item.snippet_kind}</strong> "{item.snippet_name}"
              {#if item.target_id}
                ({item.target_id})
              {/if}
              <br />
              <span>{formatTimestamp(item.created_at)} • strategy: {item.conflict_strategy}</span>
              <br />
              <button class="preset-btn" disabled={busy} onclick={() => rollbackImport(item.id)}>
                Rollback
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>

    {#if communityFeedOptIn}
      <div class="control-group control-group-block">
        <h3>Popular Workflows</h3>
        <ul class="hint-list">
          {#each popularFeed as wf}
            <li>
              <strong>{wf.title}</strong> ({wf.id}) - {wf.description}
              <br />
              <span>
                Trust: {wf.trust.publisher} / {wf.trust.risk_level} / {wf.trust.verified
                  ? "verified"
                  : "unverified"}
              </span>
              <br />
              <span>Source: {wf.trust.source}</span>
              {#if wf.trust.compatibility_hint}
                <br />
                <span>Compatibility: {wf.trust.compatibility_hint}</span>
              {/if}
              {#if wf.trust.rating != null}
                <br />
                <span>
                  Community rating: {wf.trust.rating.toFixed(1)}
                  {#if wf.trust.vote_count != null}
                    ({wf.trust.vote_count} votes)
                  {/if}
                </span>
              {/if}
              <br />
              <button class="preset-btn" disabled={busy} onclick={() => installPopular(wf.id)}>Install</button>
            </li>
          {/each}
        </ul>
      </div>
    {/if}

    {#if status}
      <div class="status-info">{status}</div>
    {/if}
  </div>
</div>
