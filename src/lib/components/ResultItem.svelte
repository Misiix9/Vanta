<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { SearchResult } from "$lib/types";
    import FileIcon from "./FileIcon.svelte";

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
    let safeInlineSvgUrl = $derived(svgToDataUri(result.icon));

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

    function isTrustedInlineSvg(icon: string | null | undefined): boolean {
        if (!icon) return false;
        const trimmed = icon.trim().toLowerCase();
        return (
            trimmed.startsWith("<svg") &&
            !trimmed.includes("<script") &&
            !trimmed.includes("onload=") &&
            !trimmed.includes("onerror=") &&
            !trimmed.includes("javascript:")
        );
    }

    function svgToDataUri(icon: string | null | undefined): string {
        if (!isTrustedInlineSvg(icon)) return "";
        return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(icon!.trim())}`;
    }

    function primaryActionLabel(): string {
        if (typeof result.source === "object" && "Script" in result.source) {
            return "Run";
        }

        switch (result.source) {
            case "Calculator":
                return "Copy Result";
            case "Window":
                return "Focus";
            case "Clipboard":
                return "Copy";
            case "File":
                return "Open";
            default:
                return "Launch";
        }
    }
</script>

<button
    class="result-item"
    class:selected={isSelected}
    onmouseenter={() => onSelect(index)}
    onclick={() => onActivate(result)}
    role="option"
    aria-selected={isSelected}
    style="transition: opacity 140ms ease, transform 140ms ease"
>
    <div class="item-icon">
        {#if result.source === "Calculator"}
            <span class="icon-emoji">🧮</span>
        {:else if typeof result.source === "object" && "Script" in result.source}
            <span class="icon-emoji">⚡</span>
        {:else if safeInlineSvgUrl}
            <img src={safeInlineSvgUrl} alt={result.title} class="icon-img" />
        {:else if result.icon && (result.icon === "dir" || result.icon.startsWith("file"))}
            <FileIcon type={result.icon} />
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

    {#if isSelected}
        <div
            class="action-hints"
            style="display:flex;gap:8px;align-items:center;margin-left:auto;flex-wrap:wrap;justify-content:flex-end;"
        >
            <span
                class="action-chip"
                style="display:inline-flex;gap:6px;align-items:center;padding:4px 8px;border:1px solid rgba(255,255,255,0.08);border-radius:8px;font-size:12px;"
            >
                <kbd style="padding:2px 6px;border:1px solid rgba(255,255,255,0.2);border-radius:6px;background:rgba(255,255,255,0.08);">
                    Enter
                </kbd>
                <span>{primaryActionLabel()}</span>
            </span>

            {#if result.actions}
                {#each result.actions as action}
                    <span
                        class="action-chip"
                        style="display:inline-flex;gap:6px;align-items:center;padding:4px 8px;border:1px solid rgba(255,255,255,0.08);border-radius:8px;font-size:12px;"
                    >
                        {#if action.shortcut}
                            <kbd style="padding:2px 6px;border:1px solid rgba(255,255,255,0.2);border-radius:6px;background:rgba(255,255,255,0.08);">
                                {action.shortcut}
                            </kbd>
                        {/if}
                        <span>{action.label}</span>
                    </span>
                {/each}
            {/if}
        </div>
    {/if}
</button>


