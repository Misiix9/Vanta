<script lang="ts">
    import { convertFileSrc } from "@tauri-apps/api/core";
    import type { SearchResult } from "$lib/types";
    import FileIcon from "./FileIcon.svelte";

    let {
        result,
        index,
        displayIndex = undefined,
        isSelected = false,
        showScore = false,
        onSelect,
        onActivate,
        onActionClick,
        onContextMenu,
    }: {
        result: SearchResult;
        index: number;
        displayIndex?: number;
        isSelected: boolean;
        showScore?: boolean;
        onSelect: (index: number) => void;
        onActivate: (result: SearchResult) => void;
        onActionClick?: (result: SearchResult, action: import("$lib/types").ResultAction) => void;
        onContextMenu?: (e: MouseEvent, result: SearchResult) => void;
    } = $props();

    let loadFailed = $state(false);
    let iconUrl = $derived(result.icon ? convertFileSrc(result.icon) : "");
    let safeInlineSvgMarkup = $derived(sanitizeInlineSvg(result.icon));

    // Detect image file results for thumbnail display
    const imageExts = new Set(["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico"]);
    let filePath = $derived.by(() => {
        if (result.source !== "File") return null;
        const exec = result.exec;
        if (exec.startsWith("open:")) return exec.slice(5);
        if (exec.startsWith("xdg-open ")) return exec.slice(9).replace(/^"|"$/g, "");
        if (exec.startsWith("/")) return exec;
        if (result.subtitle?.startsWith("/")) return result.subtitle;
        return null;
    });
    let fileExt = $derived(filePath ? (filePath.split(".").pop() ?? "").toLowerCase() : "");
    let isImageFile = $derived(imageExts.has(fileExt));
    let thumbnailUrl = $derived(isImageFile && filePath ? convertFileSrc(filePath) : null);
    let thumbFailed = $state(false);

    function getDragData(): string {
        if (filePath) return filePath;
        if (result.source === "Clipboard") return result.subtitle || result.title;
        return result.exec;
    }

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

    function sanitizeInlineSvg(icon: string | null | undefined): string {
        if (!icon) return "";

        const trimmed = icon.trim();
        if (!trimmed.toLowerCase().startsWith("<svg")) return "";

        // Remove risky tags/attributes and force color inheritance for release-safe theming.
        let cleaned = trimmed
            .replace(/<script[\s\S]*?<\/script>/gi, "")
            .replace(/<foreignObject[\s\S]*?<\/foreignObject>/gi, "")
            .replace(/\son[a-z]+\s*=\s*("[^"]*"|'[^']*'|[^\s>]+)/gi, "")
            .replace(/javascript:/gi, "")
            .replace(/\s(fill|stroke)\s*=\s*("([^"]*)"|'([^']*)')/gi, (_m, attr, _quoted, dq, sq) => {
                const value = (dq ?? sq ?? "").trim().toLowerCase();
                if (value === "none" || value === "currentcolor") {
                    return ` ${attr}="${value}"`;
                }
                return ` ${attr}="currentColor"`;
            });

        if (!cleaned.toLowerCase().includes("<svg")) return "";

        if (!/\sviewBox=/i.test(cleaned)) {
            cleaned = cleaned.replace(
                /<svg/i,
                '<svg viewBox="0 0 24 24"',
            );
        }

        return cleaned;
    }

    function isFontAwesome(icon: string | null | undefined): boolean {
        if (!icon) return false;
        return icon.includes("fa-");
    }

    let isExtensionResult = $derived(
        typeof result.source === "object" && "Extension" in result.source,
    );

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
    class="result-item v2-card"
    class:selected={isSelected}
    onmouseenter={() => onSelect(index)}
    onclick={() => onActivate(result)}
    oncontextmenu={(e) => { if (onContextMenu) { e.preventDefault(); onContextMenu(e, result); } }}
    draggable="true"
    ondragstart={(e) => { e.dataTransfer?.setData("text/plain", getDragData()); }}
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
        {:else if safeInlineSvgMarkup}
            <span
                class="icon-svg"
                class:extension-svg={isExtensionResult}
                aria-label={result.title}
            >
                {@html safeInlineSvgMarkup}
            </span>
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
            {#if showScore}
                <span class="score-badge" title="Fuzzy match score">{Math.round(result.score)}</span>
            {/if}
        </div>
        {#if result.subtitle}
            <div class="item-subtitle">{result.subtitle}</div>
        {/if}
    </div>

    {#if thumbnailUrl && !thumbFailed}
        <img
            src={thumbnailUrl}
            alt={result.title}
            class="result-thumbnail"
            onerror={() => { thumbFailed = true; }}
        />
    {/if}

    {#if isSelected}
        <div class="action-hints">
            <div
                class="action-chip action-chip-btn"
                role="button"
                tabindex="-1"
                onclick={(e) => { e.stopPropagation(); onActivate(result); }}
                onkeydown={(e) => { if (e.key === "Enter") { e.stopPropagation(); onActivate(result); } }}
            >
                <span>{primaryActionLabel()}</span>
                <kbd>↵</kbd>
            </div>

            {#if result.actions}
                {#each result.actions as action}
                    <div
                        class="action-chip action-chip-btn"
                        role="button"
                        tabindex="-1"
                        onclick={(e) => { e.stopPropagation(); onActionClick?.(result, action); }}
                        onkeydown={(e) => { if (e.key === "Enter") { e.stopPropagation(); onActionClick?.(result, action); } }}
                    >
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
                    </div>
                {/each}
            {/if}
        </div>
    {/if}
</button>
