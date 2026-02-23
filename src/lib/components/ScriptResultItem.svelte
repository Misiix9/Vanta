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
    style="transition: opacity 140ms ease, transform 140ms ease"
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

    <div class="item-meta" style="display:flex;gap:8px;align-items:center;">
        {#if item.badge}
            <span class="item-badge">{item.badge}</span>
        {/if}
        {#if isSelected && item.action}
            <span
                class="action-chip"
                style="display:inline-flex;gap:6px;align-items:center;padding:4px 8px;border:1px solid rgba(255,255,255,0.08);border-radius:8px;font-size:12px;color:#ffffff;"
            >
                <span>{item.action.type === "copy" ? "Copy" : "Run"}</span>
                <kbd
                    class="action-hint"
                    style="padding:2px 6px;border:1px solid rgba(255,255,255,0.2);border-radius:6px;background:rgba(255,255,255,0.08);color:#ffffff;font-family:inherit;margin:0;"
                >
                    {item.action.type === "copy" ? "Ctrl C" : "↵"}
                </kbd>
            </span>
        {/if}
    </div>
</button>
