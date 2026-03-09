<script lang="ts">
  import type { MacroJobRecord } from "$lib/types";

  let {
    jobs = [],
    onRetry,
    onCancel,
  }: {
    jobs: MacroJobRecord[];
    onRetry: (jobId: string) => void;
    onCancel: (jobId: string) => void;
  } = $props();

  const visible = $derived(jobs.slice(0, 4));

  function ago(ts: number): string {
    const diff = Math.max(0, Math.floor((Date.now() - ts) / 1000));
    if (diff < 60) return `${diff}s ago`;
    const mins = Math.floor(diff / 60);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    return `${hrs}h ago`;
  }
</script>

<section class="jobs-panel" aria-label="Background jobs">
  <div class="jobs-header">
    <h3>Background Jobs</h3>
    <span class="jobs-count">{jobs.length}</span>
  </div>

  {#if visible.length === 0}
    <div class="jobs-empty">No jobs yet.</div>
  {:else}
    <div class="jobs-list">
      {#each visible as job (job.id)}
        <div class="job-row">
          <div class="job-main">
            <div class="job-title">{job.macro_name}</div>
            <div class="job-meta">{job.status} • {ago(job.updated_at)}</div>
            {#if job.error}
              <div class="job-error">{job.error}</div>
            {/if}
          </div>
          <div class="job-actions">
            {#if job.status === "running"}
              <button class="job-btn" onclick={() => onCancel(job.id)}>Cancel</button>
            {:else}
              <button class="job-btn" onclick={() => onRetry(job.id)}>Retry</button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</section>
