<script lang="ts">
	type ListItem = {
		id: string;
		title: string;
		subtitle?: string;
		icon?: string;
		accessories?: string[];
	};

	let {
		items = [],
		searchBarPlaceholder = 'Search...',
		onSearchTextChange,
		isLoading = false,
		onItemSelect,
	}: {
		items: ListItem[];
		searchBarPlaceholder?: string;
		onSearchTextChange?: (text: string) => void;
		isLoading?: boolean;
		onItemSelect?: (item: ListItem) => void;
	} = $props();

	let searchText = $state('');
	let selectedIndex = $state(0);
	let listRef: HTMLDivElement | null = $state(null);
	let itemRefs: (HTMLButtonElement | null)[] = $state([]);

	let filteredItems = $derived.by(() => {
		if (!searchText.trim()) return items;
		const q = searchText.toLowerCase().trim();
		return items.filter(
			(item) =>
				item.title.toLowerCase().includes(q) ||
				(item.subtitle && item.subtitle.toLowerCase().includes(q))
		);
	});

	$effect(() => {
		selectedIndex = Math.min(Math.max(0, selectedIndex), Math.max(0, filteredItems.length - 1));
	});

	$effect(() => {
		itemRefs = Array(filteredItems.length).fill(null);
	});

	function handleKeydown(e: KeyboardEvent) {
		if (filteredItems.length === 0) return;
		switch (e.key) {
			case 'ArrowDown':
				e.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, filteredItems.length - 1);
				scrollToSelected();
				break;
			case 'ArrowUp':
				e.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				scrollToSelected();
				break;
			case 'Enter':
				e.preventDefault();
				const item = filteredItems[selectedIndex];
				if (item) onItemSelect?.(item);
				break;
			default:
				break;
		}
	}

	function scrollToSelected() {
		const el = itemRefs[selectedIndex];
		if (el && listRef) {
			el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
		}
	}

	function handleSearchInput(e: Event) {
		const target = e.target as HTMLInputElement;
		searchText = target.value;
		onSearchTextChange?.(searchText);
		selectedIndex = 0;
	}
</script>

<div class="ext-list v2-stack v2-panel" onkeydown={handleKeydown} tabindex="-1" role="listbox">
	<div class="ext-list-search">
		<input
			type="text"
			class="ext-list-input"
			placeholder={searchBarPlaceholder}
			value={searchText}
			oninput={handleSearchInput}
			aria-label="Search"
		/>
	</div>

	{#if isLoading}
		<div class="ext-list-loading">Loading...</div>
	{:else}
		<div class="ext-list-items" bind:this={listRef} role="list">
			{#each filteredItems as item, i (item.id)}
				<button
					type="button"
					class="ext-list-item"
					class:selected={i === selectedIndex}
					onclick={() => onItemSelect?.(item)}
					onmouseenter={() => (selectedIndex = i)}
					bind:this={itemRefs[i]}
					role="option"
					aria-selected={i === selectedIndex}
				>
					{#if item.icon}
						<div class="ext-list-icon">{item.icon}</div>
					{/if}
					<div class="ext-list-content">
						<span class="ext-list-title">{item.title}</span>
						{#if item.subtitle}
							<span class="ext-list-subtitle">{item.subtitle}</span>
						{/if}
					</div>
					{#if item.accessories && item.accessories.length > 0}
						<div class="ext-list-accessories">
							{#each item.accessories as acc}
								<span class="ext-list-acc">{acc}</span>
							{/each}
						</div>
					{/if}
				</button>
			{/each}
		</div>
		{#if filteredItems.length === 0}
			<div class="ext-list-empty">No items found</div>
		{/if}
	{/if}
</div>

<style>
	.ext-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
		background: var(--vanta-surface, #0c0c0c);
		border-radius: var(--vanta-radius, 12px);
		padding: 8px;
		outline: none;
	}

	.ext-list:focus {
		outline: none;
	}

	.ext-list-search {
		flex-shrink: 0;
	}

	.ext-list-input {
		width: 100%;
		padding: 10px 12px;
		background: var(--vanta-bg, #000);
		border: 1px solid var(--vanta-border, rgba(123, 53, 240, 0.14));
		border-radius: var(--vanta-radius, 8px);
		color: var(--vanta-text, #ebebeb);
		font-size: 14px;
		outline: none;
		transition: border-color 140ms ease;
	}

	.ext-list-input::placeholder {
		color: var(--vanta-text-dim, #555);
	}

	.ext-list-input:focus {
		border-color: var(--vanta-accent, #7b35f0);
		box-shadow: 0 0 0 1px var(--vanta-accent-glow, rgba(123, 53, 240, 0.3));
	}

	.ext-list-items {
		display: flex;
		flex-direction: column;
		gap: 4px;
		overflow-y: auto;
		max-height: 320px;
	}

	.ext-list-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 12px;
		background: transparent;
		border: 1px solid transparent;
		border-radius: var(--vanta-radius, 8px);
		color: var(--vanta-text, #ebebeb);
		cursor: pointer;
		text-align: left;
		transition: background 140ms ease, border-color 140ms ease;
	}

	.ext-list-item:hover,
	.ext-list-item.selected {
		background: rgba(123, 53, 240, 0.08);
		border-color: var(--vanta-border, rgba(123, 53, 240, 0.14));
	}

	.ext-list-item.selected {
		background: rgba(123, 53, 240, 0.12);
		box-shadow: 0 0 0 1px var(--vanta-accent-glow, rgba(123, 53, 240, 0.2));
	}

	.ext-list-icon {
		width: var(--icon-2xl);
		height: var(--icon-2xl);
		display: grid;
		place-items: center;
		background: var(--vanta-accent, #7b35f0);
		border-radius: 8px;
		font-size: 16px;
		flex-shrink: 0;
	}

	.ext-list-content {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}

	.ext-list-title {
		font-weight: 600;
		font-size: 14px;
	}

	.ext-list-subtitle {
		font-size: 12px;
		color: var(--vanta-text-dim, #555);
	}

	.ext-list-accessories {
		display: flex;
		gap: 6px;
		flex-shrink: 0;
	}

	.ext-list-acc {
		font-size: 11px;
		padding: 2px 8px;
		background: rgba(255, 255, 255, 0.06);
		border-radius: 6px;
		color: var(--vanta-text-dim, #555);
	}

	.ext-list-loading,
	.ext-list-empty {
		padding: 24px;
		text-align: center;
		color: var(--vanta-text-dim, #555);
		font-size: 14px;
	}
</style>
