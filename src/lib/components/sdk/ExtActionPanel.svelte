<script lang="ts">
	type ActionItem = {
		id: string;
		title: string;
		icon?: string;
		shortcut?: string;
		onAction: () => void;
	};

	let {
		actions = [],
		visible = false,
		onClose,
	}: {
		actions: ActionItem[];
		visible?: boolean;
		onClose?: () => void;
	} = $props();

	let selectedIndex = $state(0);

	$effect(() => {
		if (visible) {
			selectedIndex = 0;
		}
	});

	function handleKeydown(e: KeyboardEvent) {
		if (!visible) return;
		switch (e.key) {
			case 'Escape':
				e.preventDefault();
				onClose?.();
				break;
			case 'ArrowDown':
				e.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, actions.length - 1);
				break;
			case 'ArrowUp':
				e.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, 0);
				break;
			case 'Enter':
				e.preventDefault();
				const action = actions[selectedIndex];
				if (action) {
					action.onAction();
					onClose?.();
				}
				break;
			default:
				break;
		}
	}
</script>

{#if visible}
	<div
		class="ext-action-overlay"
		role="dialog"
		aria-modal="true"
		aria-label="Action panel"
		onkeydown={handleKeydown}
		onclick={(e) => e.target === e.currentTarget && onClose?.()}
		tabindex="-1"
	>
		<div class="ext-action-panel" role="presentation" onclick={(e) => e.stopPropagation()}>
			<div class="ext-action-list">
				{#each actions as action, i (action.id)}
					<button
						type="button"
						class="ext-action-item"
						class:selected={i === selectedIndex}
						onclick={() => {
							action.onAction();
							onClose?.();
						}}
						onmouseenter={() => (selectedIndex = i)}
					>
						{#if action.icon}
							<span class="ext-action-icon">{action.icon}</span>
						{/if}
						<span class="ext-action-title">{action.title}</span>
						{#if action.shortcut}
							<kbd class="ext-action-shortcut">{action.shortcut}</kbd>
						{/if}
					</button>
				{/each}
			</div>
		</div>
	</div>
{/if}

<style>
	.ext-action-overlay {
		position: fixed;
		inset: 0;
		z-index: 1000;
		display: flex;
		align-items: center;
		justify-content: center;
		background: rgba(0, 0, 0, 0.5);
		backdrop-filter: blur(4px);
		outline: none;
	}

	.ext-action-panel {
		min-width: 280px;
		max-width: 90vw;
		max-height: 70vh;
		overflow-y: auto;
		background: var(--vanta-surface, #0c0c0c);
		border: 1px solid var(--vanta-border, rgba(123, 53, 240, 0.14));
		border-radius: var(--vanta-radius, 12px);
		box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5), 0 0 0 1px rgba(123, 53, 240, 0.1);
	}

	.ext-action-list {
		display: flex;
		flex-direction: column;
		padding: 8px;
		gap: 4px;
	}

	.ext-action-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 14px;
		background: transparent;
		border: 1px solid transparent;
		border-radius: var(--vanta-radius, 8px);
		color: var(--vanta-text, #ebebeb);
		cursor: pointer;
		text-align: left;
		width: 100%;
		transition: background 140ms ease, border-color 140ms ease;
	}

	.ext-action-item:hover,
	.ext-action-item.selected {
		background: rgba(123, 53, 240, 0.08);
		border-color: var(--vanta-border, rgba(123, 53, 240, 0.14));
	}

	.ext-action-item.selected {
		background: rgba(123, 53, 240, 0.12);
		box-shadow: 0 0 0 1px var(--vanta-accent-glow, rgba(123, 53, 240, 0.2));
	}

	.ext-action-icon {
		width: var(--icon-xl);
		height: var(--icon-xl);
		display: grid;
		place-items: center;
		background: var(--vanta-accent, #7b35f0);
		border-radius: 6px;
		font-size: 14px;
		flex-shrink: 0;
	}

	.ext-action-title {
		flex: 1;
		font-weight: 600;
		font-size: 14px;
	}

	.ext-action-shortcut {
		padding: 4px 8px;
		font-size: 11px;
		font-family: inherit;
		background: rgba(255, 255, 255, 0.08);
		border: 1px solid var(--vanta-border, rgba(123, 53, 240, 0.14));
		border-radius: 6px;
		color: var(--vanta-text-dim, #555);
	}
</style>
