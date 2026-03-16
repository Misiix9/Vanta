<script lang="ts">
    let { type }: { type: string } = $props();

    // Mapping logic
    let iconType = $derived.by(() => {
        if (type === "dir") return "folder";
        if (type.startsWith("file:")) {
            const ext = type.split(":")[1]?.toLowerCase() ?? "";
            if (["pdf"].includes(ext)) return "pdf";
            if (
                [
                    "java",
                    "c",
                    "h",
                    "cpp",
                    "cxx",
                    "cc",
                    "hpp",
                    "py",
                    "go",
                    "kt",
                    "kts",
                    "rb",
                ].includes(ext)
            )
                return "code";
            if (["doc", "docx", "xls", "xlsx", "ppt", "pptx"].includes(ext))
                return "office";
            if (
                [
                    "txt",
                    "md",
                    "rs",
                    "js",
                    "ts",
                    "json",
                    "conf",
                    "toml",
                    "yaml",
                    "yml",
                    "css",
                    "html",
                ].includes(ext)
            )
                return "text";
            if (
                ["png", "jpg", "jpeg", "gif", "webp", "svg", "bmp"].includes(
                    ext,
                )
            )
                return "image";
            if (["mp4", "mkv", "avi", "mov", "webm"].includes(ext))
                return "video";
            if (["mp3", "wav", "flac", "ogg", "aac"].includes(ext))
                return "audio";
            if (["zip", "tar", "gz", "7z", "rar"].includes(ext))
                return "archive";
            return "file";
        }
        return "file";
    });
</script>

<div class="file-icon">
    {#if iconType === "folder"}
        <!-- Folder Icon -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path
                d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"
            ></path>
        </svg>
    {:else if iconType === "pdf"}
        <!-- PDF Icon (File with text + tiny badge or just distinctive look) -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
            ></path>
            <polyline points="14 2 14 8 20 8"></polyline>
            <!-- Tiny "PDF" letters or symbol implies PDF. Let's use lines + a highlight -->
            <path d="M9 13v-1h6v1" style="opacity: 0.5" />
            <path d="M12 18v-6" />
            <path d="M9 15h6" />
        </svg>
    {:else if iconType === "text"}
        <!-- Text Icon (Lines) -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
            ></path>
            <polyline points="14 2 14 8 20 8"></polyline>
            <line x1="16" y1="13" x2="8" y2="13"></line>
            <line x1="16" y1="17" x2="8" y2="17"></line>
            <line x1="10" y1="9" x2="8" y2="9"></line>
        </svg>
    {:else if iconType === "code"}
        <!-- Code File Icon -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
            ></path>
            <polyline points="14 2 14 8 20 8"></polyline>
            <polyline points="10 13 8 15 10 17"></polyline>
            <polyline points="14 13 16 15 14 17"></polyline>
        </svg>
    {:else if iconType === "office"}
        <!-- Office Document Icon -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"
            ></path>
            <polyline points="14 2 14 8 20 8"></polyline>
            <rect x="8" y="12" width="8" height="6" rx="1"></rect>
            <line x1="8" y1="15" x2="16" y2="15"></line>
            <line x1="12" y1="12" x2="12" y2="18"></line>
        </svg>
    {:else if iconType === "image"}
        <!-- Image Icon (Mountain) -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
            <circle cx="8.5" cy="8.5" r="1.5" />
            <polyline points="21 15 16 10 5 21" />
        </svg>
    {:else if iconType === "video"}
        <!-- Video Icon (Filmstrip-ish) -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"></rect>
            <line x1="7" y1="2" x2="7" y2="22"></line>
            <line x1="17" y1="2" x2="17" y2="22"></line>
            <line x1="2" y1="12" x2="22" y2="12"></line>
            <line x1="2" y1="7" x2="7" y2="7"></line>
            <line x1="2" y1="17" x2="7" y2="17"></line>
            <line x1="17" y1="17" x2="22" y2="17"></line>
            <line x1="17" y1="7" x2="22" y2="7"></line>
        </svg>
    {:else if iconType === "archive"}
        <!-- Archive (Zipper) -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M21 8v13H3V8"></path>
            <path d="M1 3h22v5H1z"></path>
            <path d="M10 12h4"></path>
        </svg>
    {:else}
        <!-- Default File -->
        <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9z"
            ></path>
            <polyline points="13 2 13 9 20 9"></polyline>
        </svg>
    {/if}
</div>


