<script lang="ts">
    import type { ScriptItem } from "$lib/types";

    let {
        item,
        index,
        isSelected = false,
        onSelect,
        onActivate,
    }: {
        item: ScriptItem;
        index: number;
        isSelected: boolean;
        onSelect: (index: number) => void;
        onActivate: (item: ScriptItem) => void;
    } = $props();

    const urgencyColors: Record<string, string> = {
        low: "rgba(255, 255, 255, 0.3)",
        normal: "var(--vanta-accent)",
        critical: "#ff4757",
    };

    function getScriptIconColor(title: string): string {
        let hash = 0;
        for (let i = 0; i < title.length; i++) {
            hash = title.charCodeAt(i) + ((hash << 5) - hash);
        }
        const hue = (Math.abs(hash) % 60) + 240; // bias toward purple/blue
        return `hsl(${hue}, 55%, 50%)`;
    }
</script>

<button
    class="vanta-item script-item"
    class:selected={isSelected}
    class:urgency-critical={item.urgency === "critical"}
    class:urgency-low={item.urgency === "low"}
    onmouseenter={() => onSelect(index)}
    onclick={() => onActivate(item)}
    role="option"
    aria-selected={isSelected}
    style="animation-delay: {index * 30}ms"
>
    <div
        class="item-icon script-icon"
        style="background: {getScriptIconColor(item.title)}"
    >
        <span class="icon-glyph">⚡</span>
    </div>

    <div class="item-content">
        <div class="item-title">
            {item.title}
            <span class="script-badge">SCRIPT</span>
        </div>
        {#if item.subtitle}
            <div class="item-subtitle">{item.subtitle}</div>
        {/if}
    </div>

    <div class="item-meta">
        {#if item.badge}
            <span class="item-badge">{item.badge}</span>
        {/if}
        {#if isSelected && item.action}
            <kbd class="action-hint">
                {item.action.type === "copy" ? "⌘C" : "↵"}
            </kbd>
        {/if}
    </div>
</button>

<style>
    .script-item {
        display: flex;
        align-items: center;
        gap: 16px;
        padding: 12px 16px;
        width: 100%;
        text-align: left;

        background: transparent;
        border: 1px solid transparent;
        border-radius: 12px;

        transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
        cursor: pointer;
        position: relative;
        overflow: visible;
    }

    .script-item.selected,
    .script-item:hover {
        background: var(--vanta-item-bg);
        transform: scale(1.02) translateY(-2px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.05);
        z-index: 10;
    }

    .script-item.urgency-critical {
        border-left: 2px solid #ff4757;
    }

    .script-item.urgency-low {
        opacity: 0.65;
    }

    .script-icon {
        width: 40px;
        height: 40px;
        border-radius: 10px;
        display: flex;
        align-items: center;
        justify-content: center;
        flex-shrink: 0;

        background: rgba(
            255,
            255,
            255,
            0.1
        ) !important; /* Force monochrome bg */
        box-shadow: none;

        /* Monochrome Icon filter */
        filter: grayscale(100%) brightness(200%);
    }

    .icon-glyph {
        font-size: 18px;
        filter: none;
        color: #fff;
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
        font-weight: 400;
        color: var(--vanta-text);
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .script-badge {
        font-size: 9px;
        font-weight: 700;
        letter-spacing: 0.5px;
        padding: 2px 6px;
        border-radius: 4px;
        background: rgba(255, 255, 255, 0.15);
        color: #fff;
        border: none;
        flex-shrink: 0;
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

    .item-meta {
        display: flex;
        align-items: center;
        gap: 8px;
        flex-shrink: 0;
        opacity: 0.7;
        transition: opacity 0.2s ease;
    }

    .script-item.selected .item-meta {
        opacity: 1;
    }

    .item-badge {
        font-size: 10px;
        font-weight: 600;
        padding: 2px 8px;
        border-radius: 6px;
        background: rgba(255, 255, 255, 0.08);
        color: var(--vanta-text-dim);
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .action-hint {
        background: rgba(255, 255, 255, 0.1);
        border: none;
        border-radius: 4px;
        padding: 2px 6px;
        font-size: 11px;
        color: var(--vanta-text);
        font-family: inherit;
    }
</style>
