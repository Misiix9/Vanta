<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { SearchResult } from "$lib/types";
    import FileIcon from "./FileIcon.svelte";

    let {
        result,
        index,
        displayIndex = undefined,
        isSelected = false,
        onSelect,
        onActivate,
    }: {
        result: SearchResult;
        index: number;
        displayIndex?: number;
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

    function isFontAwesome(icon: string | null | undefined): boolean {
        if (!icon) return false;
        return icon.includes("fa-");
    }

    function svgToDataUri(icon: string | null | undefined): string {
        if (!isTrustedInlineSvg(icon)) return "";
        const themed = icon!.trim().replace(/currentColor/g, '#ebebeb');
        return `data:image/svg+xml;charset=utf-8,${encodeURIComponent(themed)}`;
    }

    function primaryActionLabel(): string {
        if (typeof result.source === "object" && "Extension" in result.source) {
            return "Open";
        }
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
>
    <div class="item-icon">
        {#if result.source === "Calculator"}
            <span class="icon-emoji">🧮</span>
        {:else if typeof result.source === "object" && "Script" in result.source}
            <span class="icon-emoji">⚡</span>
        {:else if isFontAwesome(result.icon)}
            <i class="{result.icon} icon-fa"></i>
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
            <span class="icon-number">{(displayIndex ?? index) + 1}</span>
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
        <div class="action-hints">
            <span class="action-chip">
                <span>{primaryActionLabel()}</span>
                <kbd>↵</kbd>
            </span>

            {#if result.actions}
                {#each result.actions as action}
                    <span class="action-chip">
                        <span>{action.label}</span>
                        {#if action.shortcut}
                            <kbd>
                                {action.shortcut
                                    .replace("Shift+", "⇧ ")
                                    .replace("Ctrl+", "Ctrl ")
                                    .replace("Alt+", "Alt ")
                                    .replace("Enter", "↵")}
                            </kbd>
                        {/if}
                    </span>
                {/each}
            {/if}
        </div>
    {/if}
</button>
