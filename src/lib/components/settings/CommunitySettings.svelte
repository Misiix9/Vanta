<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import type {
        VantaConfig,
        CommunityFeedbackSummary,
        CommunityFeedbackReceipt,
        PopularWorkflowFeedEntry,
        WorkflowMacro,
        ProfilesConfig,
    } from "$lib/types";

    let {
        config = $bindable(),
        onSave,
    }: {
        config: VantaConfig;
        onSave: () => void;
    } = $props();

    let communitySummary: CommunityFeedbackSummary | null = $state(null);
    let communityFeed: PopularWorkflowFeedEntry[] = $state([]);
    let availableWorkflowIds: string[] = $state([]);
    let availableProfileIds: string[] = $state([]);
    let feedbackMessage = $state("");
    let feedbackContact = $state("");
    let selectedRoadmapTopic = $state("popular-workflows-feed");
    let snippetKind = $state<"workflow" | "profile" | "theme">("workflow");
    let snippetTargetId = $state("");
    let snippetPayload = $state("");
    let snippetStatus = $state<string | null>(null);
    let feedbackStatus = $state<string | null>(null);
    let communityBusy = $state(false);

    const roadmapTopics = [
        { id: "shareable-snippets", label: "Shareable snippets" },
        { id: "popular-workflows-feed", label: "Popular workflows feed" },
        { id: "in-app-feedback-channel", label: "In-app feedback channel" },
        { id: "settings-ia-refresh", label: "Settings IA refresh (Phase 24)" },
    ];

    async function loadCommunitySummary() {
        try {
            communitySummary = await invoke<CommunityFeedbackSummary>("get_community_feedback_summary");
        } catch (e) {
            console.error("Failed to load community summary:", e);
        }
    }

    async function loadPopularWorkflowsFeed() {
        if (!config.general.community_feed_opt_in) { communityFeed = []; return; }
        try {
            communityFeed = await invoke<PopularWorkflowFeedEntry[]>("get_popular_workflows_feed");
        } catch (e) {
            feedbackStatus = `Failed to load popular feed: ${String(e)}`;
        }
    }

    async function loadWorkflowTargets() {
        try {
            const macros = await invoke<WorkflowMacro[]>("get_workflows");
            availableWorkflowIds = macros.map((m) => m.id).sort();
            if (!snippetTargetId && availableWorkflowIds.length > 0) snippetTargetId = availableWorkflowIds[0];
        } catch (e) { console.error("Failed to load workflow targets:", e); }
        try {
            const profiles = await invoke<ProfilesConfig>("get_profiles");
            availableProfileIds = profiles.entries.map((p) => p.id).sort();
            if (snippetKind === "profile" && availableProfileIds.length > 0 && !snippetTargetId)
                snippetTargetId = availableProfileIds[0];
        } catch (e) { console.error("Failed to load profile targets:", e); }
    }

    async function installPopularWorkflow(id: string) {
        communityBusy = true; feedbackStatus = null;
        try {
            await invoke("install_popular_workflow", { workflowId: id });
            feedbackStatus = `Installed popular workflow '${id}'.`;
            await loadWorkflowTargets();
        } catch (e) { feedbackStatus = `Failed to install workflow: ${String(e)}`; }
        finally { communityBusy = false; }
    }

    function selectedSnippetTarget(): string | null {
        if (snippetKind === "theme") return null;
        return snippetTargetId.trim() || null;
    }

    async function exportCommunitySnippet() {
        snippetStatus = null;
        try {
            snippetPayload = await invoke<string>("export_community_snippet", { kind: snippetKind, targetId: selectedSnippetTarget() });
            snippetStatus = `Exported ${snippetKind} snippet.`;
        } catch (e) { snippetStatus = `Export failed: ${String(e)}`; }
    }

    async function importCommunitySnippet() {
        if (!snippetPayload.trim()) { snippetStatus = "Paste a snippet payload first."; return; }
        snippetStatus = null;
        try {
            const message = await invoke<string>("import_community_snippet", { snippetJson: snippetPayload });
            snippetStatus = message;
            await loadWorkflowTargets();
        } catch (e) { snippetStatus = `Import failed: ${String(e)}`; }
    }

    function onSnippetKindChange(nextKind: "workflow" | "profile" | "theme") {
        snippetKind = nextKind;
        if (nextKind === "workflow") snippetTargetId = availableWorkflowIds[0] || "";
        else if (nextKind === "profile") snippetTargetId = availableProfileIds[0] || "";
        else snippetTargetId = "";
    }

    async function submitCommunityFeedback() {
        if (!feedbackMessage.trim()) { feedbackStatus = "Please add feedback before submitting."; return; }
        communityBusy = true; feedbackStatus = null;
        try {
            const receipt = await invoke<CommunityFeedbackReceipt>("submit_community_feedback", {
                message: feedbackMessage, contact: feedbackContact.trim() || null, roadmapTopic: selectedRoadmapTopic || null,
            });
            feedbackMessage = "";
            feedbackStatus = `Thanks, feedback recorded (${receipt.id}).`;
            await loadCommunitySummary();
        } catch (e) { feedbackStatus = `Failed to submit feedback: ${String(e)}`; }
        finally { communityBusy = false; }
    }

    async function voteRoadmapTopic(topic: string) {
        communityBusy = true; feedbackStatus = null;
        try {
            await invoke("vote_roadmap_item", { topic });
            selectedRoadmapTopic = topic;
            feedbackStatus = `Vote recorded for ${topic}.`;
            await loadCommunitySummary();
        } catch (e) { feedbackStatus = `Failed to vote: ${String(e)}`; }
        finally { communityBusy = false; }
    }

    onMount(() => {
        loadCommunitySummary();
        loadWorkflowTargets();
        loadPopularWorkflowsFeed();
    });
</script>

<div class="control-group">
    <label>Feedback Message
        <textarea rows="5" bind:value={feedbackMessage} placeholder="Tell us what to improve in Vanta"></textarea>
    </label>
</div>
<div class="control-group">
    <label>Contact (optional) <input type="text" bind:value={feedbackContact} placeholder="email or handle" /></label>
</div>
<div class="control-group">
    <label>Roadmap Topic
        <select class="vanta-select" bind:value={selectedRoadmapTopic}>
            {#each roadmapTopics as topic}
                <option value={topic.id}>{topic.label}</option>
            {/each}
        </select>
    </label>
</div>
<div class="preset-row">
    <button class="preset-btn" onclick={submitCommunityFeedback} disabled={communityBusy}>
        {communityBusy ? "Submitting..." : "Submit Feedback"}
    </button>
    {#each roadmapTopics as topic}
        <button class="preset-btn" disabled={communityBusy} onclick={() => voteRoadmapTopic(topic.id)}>Vote: {topic.label}</button>
    {/each}
</div>

<div class="control-group control-group-block">
    <h4>Popular Workflows Feed (Opt-in)</h4>
    {#if !config.general.community_feed_opt_in}
        <p>Enable opt-in under General to fetch curated popular workflows.</p>
    {:else if communityFeed.length === 0}
        <p>No popular workflows available right now.</p>
    {:else}
        <ul class="hint-list">
            {#each communityFeed as item}
                <li>
                    <strong>{item.title}</strong> ({item.id}) - {item.description}<br />
                    <span>Tags: {item.tags.join(", ")} | Trust: {item.trust.publisher} / {item.trust.risk_level}</span><br />
                    <button class="preset-btn" disabled={communityBusy} onclick={() => installPopularWorkflow(item.id)}>Install Workflow</button>
                </li>
            {/each}
        </ul>
    {/if}
</div>

<div class="control-group control-group-block">
    <h4>Shareable Snippets</h4>
    <div class="preset-row">
        <button class="preset-btn" onclick={() => onSnippetKindChange("workflow")}>Workflow</button>
        <button class="preset-btn" onclick={() => onSnippetKindChange("profile")}>Profile</button>
        <button class="preset-btn" onclick={() => onSnippetKindChange("theme")}>Theme</button>
    </div>
    {#if snippetKind !== "theme"}
        <label>Target ID
            <select class="vanta-select" bind:value={snippetTargetId}>
                {#if snippetKind === "workflow"}
                    {#each availableWorkflowIds as id}<option value={id}>{id}</option>{/each}
                {:else}
                    {#each availableProfileIds as id}<option value={id}>{id}</option>{/each}
                {/if}
            </select>
        </label>
    {/if}
    <div class="preset-row">
        <button class="preset-btn" onclick={exportCommunitySnippet}>Export Snippet</button>
        <button class="preset-btn" onclick={importCommunitySnippet}>Import Snippet</button>
    </div>
    <label>Snippet Payload JSON
        <textarea rows="8" bind:value={snippetPayload} placeholder="Paste or export snippet JSON"></textarea>
    </label>
    {#if snippetStatus}<div class="status-info">{snippetStatus}</div>{/if}
</div>

{#if communitySummary}
    <div class="control-group control-group-block">
        <h4>Community Snapshot</h4>
        <p>Total feedback: {communitySummary.total_feedback}</p>
        <ul class="hint-list">
            {#each communitySummary.top_votes as vote}
                <li><strong>{vote.topic}</strong>: {vote.votes}</li>
            {/each}
        </ul>
    </div>
{/if}

{#if feedbackStatus}<div class="status-info">{feedbackStatus}</div>{/if}
