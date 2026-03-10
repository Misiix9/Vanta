<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type {
    CommunityFeedbackSummary,
    CommunityFeedbackReceipt,
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
  let status = $state<string | null>(null);

  const topics = [
    { id: "shareable-snippets", label: "Shareable snippets" },
    { id: "popular-workflows-feed", label: "Popular workflows feed" },
    { id: "in-app-feedback-channel", label: "In-app feedback" },
    { id: "adaptive-intent-engine", label: "Adaptive intent engine" },
  ];

  async function refresh() {
    try {
      summary = await invoke<CommunityFeedbackSummary>("get_community_feedback_summary");
      if (communityFeedOptIn) {
        popularFeed = await invoke<PopularWorkflowFeedEntry[]>("get_popular_workflows_feed");
      } else {
        popularFeed = [];
      }
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
      status = `Exported ${snippetKind} snippet.`;
    } catch (e) {
      status = `Export failed: ${String(e)}`;
    }
  }

  async function importSnippet() {
    if (!snippetPayload.trim()) {
      status = "Paste snippet payload first.";
      return;
    }
    status = null;
    try {
      const msg = await invoke<string>("import_community_snippet", { snippetJson: snippetPayload });
      status = msg;
    } catch (e) {
      status = `Import failed: ${String(e)}`;
    }
  }

  $effect(() => {
    refresh();
  });
</script>

<div class="hub-window-root">
  <div class="hub-window-header">
    <button class="link-btn" onclick={onClose}>Back</button>
    <h2>Community Hub</h2>
    <button class="link-btn" onclick={refresh}>Refresh</button>
  </div>

  <div class="hub-window-scroll">
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

    <div class="control-group" style="display:block;">
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

    <div class="control-group" style="display:block;">
      <h3>Shareable Snippets</h3>
      <div class="preset-row">
        <button class="preset-btn" onclick={() => (snippetKind = "workflow")}>Workflow</button>
        <button class="preset-btn" onclick={() => (snippetKind = "profile")}>Profile</button>
        <button class="preset-btn" onclick={() => (snippetKind = "theme")}>Theme</button>
      </div>
      {#if snippetKind !== "theme"}
        <label>
          Target ID
          <input type="text" bind:value={snippetTargetId} placeholder="workflow/profile id" />
        </label>
      {/if}
      <div class="preset-row">
        <button class="preset-btn" onclick={exportSnippet}>Export</button>
        <button class="preset-btn" onclick={importSnippet}>Import</button>
      </div>
      <label>
        Snippet Payload
        <textarea rows="8" bind:value={snippetPayload}></textarea>
      </label>
    </div>

    {#if communityFeedOptIn}
      <div class="control-group" style="display:block;">
        <h3>Popular Workflows</h3>
        <ul class="hint-list">
          {#each popularFeed as wf}
            <li>
              <strong>{wf.title}</strong> ({wf.id}) - {wf.description}
              <br />
              <span>Trust: {wf.trust.publisher} / {wf.trust.risk_level}</span>
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
