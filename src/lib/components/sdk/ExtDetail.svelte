<script lang="ts">
	type MetadataItem = { label: string; value: string };
	type ActionItem = { title: string; icon?: string; onAction: () => void };

	let {
		title = '',
		content = '',
		metadata = [],
		actions = [],
	}: {
		title: string;
		content?: string;
		metadata?: MetadataItem[];
		actions?: ActionItem[];
	} = $props();
</script>

<div class="ext-detail">
	<div class="ext-detail-main">
		<h1 class="ext-detail-title">{title}</h1>
		{#if content}
			<div class="ext-detail-content">{content}</div>
		{/if}
		{#if actions.length > 0}
			<div class="ext-detail-actions">
				{#each actions as action}
					<button
						type="button"
						class="ext-detail-action"
						onclick={() => action.onAction()}
					>
						{#if action.icon}
							<span class="ext-detail-action-icon">{action.icon}</span>
						{/if}
						{action.title}
					</button>
				{/each}
			</div>
		{/if}
	</div>
	{#if metadata.length > 0}
		<aside class="ext-detail-sidebar">
			<h3 class="ext-detail-sidebar-title">Details</h3>
			<dl class="ext-detail-meta">
				{#each metadata as item}
					<div class="ext-detail-meta-row">
						<dt class="ext-detail-meta-label">{item.label}</dt>
						<dd class="ext-detail-meta-value">{item.value}</dd>
					</div>
				{/each}
			</dl>
		</aside>
	{/if}
</div>

<style>
	.ext-detail {
		display: flex;
		gap: 24px;
		background: var(--vanta-surface, #0c0c0c);
		border-radius: var(--vanta-radius, 12px);
		padding: 20px;
	}

	.ext-detail-main {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.ext-detail-title {
		margin: 0;
		font-size: 20px;
		font-weight: 700;
		color: var(--vanta-text, #ebebeb);
	}

	.ext-detail-content {
		font-size: 14px;
		line-height: 1.6;
		color: var(--vanta-text, #ebebeb);
		white-space: pre-wrap;
		word-break: break-word;
	}

	.ext-detail-actions {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		margin-top: 8px;
	}

	.ext-detail-action {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		padding: 10px 16px;
		background: var(--vanta-accent, #7b35f0);
		color: #fff;
		border: none;
		border-radius: var(--vanta-radius, 8px);
		font-weight: 600;
		font-size: 13px;
		cursor: pointer;
		transition: background 140ms ease, box-shadow 140ms ease;
	}

	.ext-detail-action:hover {
		background: var(--vanta-accent-glow, #9b5ff8);
		box-shadow: 0 0 16px var(--vanta-accent-glow, rgba(123, 53, 240, 0.4));
	}

	.ext-detail-action-icon {
		font-size: 16px;
	}

	.ext-detail-sidebar {
		width: 200px;
		flex-shrink: 0;
		padding: 16px;
		background: var(--vanta-bg, #000);
		border: 1px solid var(--vanta-border, rgba(123, 53, 240, 0.14));
		border-radius: var(--vanta-radius, 8px);
	}

	.ext-detail-sidebar-title {
		margin: 0 0 12px 0;
		font-size: 12px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--vanta-text-dim, #555);
	}

	.ext-detail-meta {
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.ext-detail-meta-row {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.ext-detail-meta-label {
		margin: 0;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--vanta-text-dim, #555);
	}

	.ext-detail-meta-value {
		margin: 0;
		font-size: 13px;
		color: var(--vanta-text, #ebebeb);
	}
</style>
