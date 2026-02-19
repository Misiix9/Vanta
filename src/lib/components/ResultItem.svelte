<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { SearchResult } from "$lib/types";

    let {
        result,
        index,
        isSelected = false,
        onSelect,
        onActivate,
    }: {
        result: SearchResult;
        index: number;
        isSelected: boolean;
        onSelect: (index: number) => void;
        onActivate: (result: SearchResult) => void;
    } = $props();

    let loadFailed = $state(false);
    let iconUrl = $derived(result.icon ? convertFileSrc(result.icon) : "");

    /**
     * Highlight matching characters in the title based on match_indices.
     */
    function highlightTitle(title: string, indices: number[]): string {
        if (!indices || indices.length === 0) return escapeHtml(title);

        const indexSet = new Set(indices);
        let html = "";
        for (let i = 0; i < title.length; i++) {
            const char = escapeHtml(title[i]);
            if (indexSet.has(i)) {
                html += `<span class="match-highlight">${char}</span>`;
            } else {
                html += char;
            }
        }
        return html;
    }

    function escapeHtml(str: string): string {
        return str
            .replace(/&/g, "&amp;")
            .replace(/</g, "&lt;")
            .replace(/>/g, "&gt;");
    }

    function handleImageError(e: Event) {
        console.warn(
            "Failed to load icon:",
            result.icon,
            "Converted:",
            convertFileSrc(result.icon || ""),
        );
        loadFailed = true;
    }
</script>

<button
    class="vanta-item result-item"
    class:selected={isSelected}
    onmouseenter={() => onSelect(index)}
    onclick={() => onActivate(result)}
    role="option"
    aria-selected={isSelected}
    style="animation-delay: {index * 30}ms"
>
    <div class="item-icon">
        {#if result.source === "Calculator"}
            <span class="icon-emoji">🧮</span>
        {:else if typeof result.source === "object" && "Script" in result.source}
            <span class="icon-emoji">⚡</span>
        {:else if result.icon && !loadFailed}
            <img
                src={iconUrl}
                alt={result.title}
                class="icon-img"
                onerror={handleImageError}
            />
        {:else if result.source === "Window"}
            <span class="icon-emoji">🪟</span>
        {:else if result.source === "Clipboard"}
            <span class="icon-number">{index + 1}</span>
        {:else}
            <img
                src="/package.svg"
                alt="Application"
                class="icon-img icon-fallback"
            />
        {/if}
    </div>

    <div class="item-content">
        <div class="item-title">
            {@html highlightTitle(result.title, result.match_indices)}
        </div>
        {#if result.subtitle}
            <div class="item-subtitle">{result.subtitle}</div>
        {/if}
    </div>

    <div class="item-action-hint">
        {#if isSelected}
            <kbd>↵</kbd>
        {/if}
    </div>
</button>

<style>
    .result-item {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 12px 16px;
        width: 100%;
        text-align: left;

        background: transparent;
        border: 1px solid transparent;
        border-radius: 12px; /* Inner radius */

        transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
        cursor: pointer;
        position: relative;
        overflow: visible; /* Allow shadow/scale to spill */
    }

    /* Selected State: Card "Float" */
    .result-item.selected,
    .result-item:hover {
        background: var(--vanta-item-bg);
        transform: scale(1.02) translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.05);
        z-index: 10;
    }

    .item-icon {
        width: 40px;
        height: 40px;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;
        font-weight: 600;
        font-size: 16px;
    }

    .icon-emoji {
        font-size: 24px;
        line-height: 1;
    }

    .icon-number {
        font-size: 20px;
        font-weight: 700;
        color: var(--vanta-text-dim);
        font-variant-numeric: tabular-nums;
        opacity: 0.8;
    }

    .icon-fallback {
        opacity: 0.5;
    }

    .icon-img {
        width: 100%;
        height: 100%;
        object-fit: contain;
    }

    .item-content {
        flex: 1;
        min-width: 0;
        display: flex;
        flex-direction: column;
        justify-content: center;
    }

    .item-title {
        font-size: 15px;
        font-weight: 400; /* Thin/Regular as requested */
        color: var(--vanta-text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .item-title :global(.match-highlight) {
        color: var(--vanta-accent);
        font-weight: 600;
    }

    .item-subtitle {
        font-size: 12px;
        color: var(--vanta-text-dim);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        margin-top: 2px;
        font-weight: 300;
    }

    .item-action-hint {
        display: flex;
        align-items: center;
        flex-shrink: 0;
        opacity: 0;
        transform: translateX(10px);
        transition: all 0.2s ease;
    }

    .result-item.selected .item-action-hint {
        opacity: 1;
        transform: translateX(0);
    }

    .item-action-hint kbd {
        background: rgba(255, 255, 255, 0.1);
        border: none;
        border-radius: 4px;
        padding: 2px 6px;
        font-size: 11px;
        color: var(--vanta-text);
        font-family: inherit;
    }
</style>
