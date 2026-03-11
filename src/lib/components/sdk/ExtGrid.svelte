<script lang="ts">
	type GridItem = {
		id: string;
		title: string;
		subtitle?: string;
		icon?: string;
		image?: string;
	};

	let {
		items = [],
		columns = 3,
		onItemSelect,
	}: {
		items: GridItem[];
		columns?: number;
		onItemSelect?: (item: GridItem) => void;
	} = $props();
</script>

<div class="ext-grid" style="--ext-grid-cols: {columns}">
	{#each items as item (item.id)}
		<button
			type="button"
			class="ext-grid-item"
			onclick={() => onItemSelect?.(item)}
		>
			{#if item.image}
				<div class="ext-grid-media ext-grid-image">
					<img src={item.image} alt={item.title} />
				</div>
			{:else if item.icon}
				<div class="ext-grid-media ext-grid-icon">{item.icon}</div>
			{:else}
				<div class="ext-grid-media ext-grid-placeholder">
					<span class="ext-grid-placeholder-text">{item.title.slice(0, 1).toUpperCase()}</span>
				</div>
			{/if}
			<div class="ext-grid-content">
				<span class="ext-grid-title">{item.title}</span>
				{#if item.subtitle}
					<span class="ext-grid-subtitle">{item.subtitle}</span>
				{/if}
			</div>
		</button>
	{/each}
</div>

<style>
	.ext-grid {
		display: grid;
		grid-template-columns: repeat(var(--ext-grid-cols, 3), 1fr);
		gap: 12px;
		background: var(--vanta-surface, #0c0c0c);
		border-radius: var(--vanta-radius, 12px);
		padding: 12px;
	}

	.ext-grid-item {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		padding: 16px 12px;
		background: var(--vanta-bg, #000);
		border: 1px solid transparent;
		border-radius: var(--vanta-radius, 8px);
		color: var(--vanta-text, #ebebeb);
		cursor: pointer;
		text-align: center;
		transition: background 140ms ease, border-color 140ms ease, transform 140ms ease;
	}

	.ext-grid-item:hover {
		background: rgba(123, 53, 240, 0.08);
		border-color: var(--vanta-border, rgba(123, 53, 240, 0.14));
		transform: translateY(-2px);
	}

	.ext-grid-item:active {
		transform: translateY(0);
	}

	.ext-grid-media {
		width: var(--icon-4xl);
		height: var(--icon-4xl);
		border-radius: var(--vanta-radius, 8px);
		overflow: hidden;
		flex-shrink: 0;
	}

	.ext-grid-image {
		display: block;
	}

	.ext-grid-image img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.ext-grid-icon {
		display: grid;
		place-items: center;
		background: var(--vanta-accent, #7b35f0);
		font-size: 24px;
	}

	.ext-grid-placeholder {
		display: grid;
		place-items: center;
		background: rgba(123, 53, 240, 0.2);
	}

	.ext-grid-placeholder-text {
		font-size: 20px;
		font-weight: 700;
		color: var(--vanta-accent, #7b35f0);
	}

	.ext-grid-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}

	.ext-grid-title {
		font-weight: 600;
		font-size: 13px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 100%;
	}

	.ext-grid-subtitle {
		font-size: 11px;
		color: var(--vanta-text-dim, #555);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 100%;
	}
</style>
