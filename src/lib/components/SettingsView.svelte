<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
    import { LogicalSize } from "@tauri-apps/api/dpi";
    import type { VantaConfig } from "$lib/types";

    let {
        config = $bindable(),
        onClose,
    }: { config: VantaConfig; onClose: () => void } = $props();

    let saveTimeout: any = null;

    function debouncedSave() {
        // 1. Apply visual changes instantly via CSS variables
        applyLivePreview();

        // 2. Debounce the actual save to disk
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(async () => {
            try {
                await invoke("save_config", { newConfig: config });
                console.log("Config saved to disk");
            } catch (e) {
                console.error("Failed to save config:", e);
            }
        }, 500);
    }

    function updateWindowSize() {
        if (config.window) {
            const win = getCurrentWebviewWindow();
            win.setSize(
                new LogicalSize(config.window.width, config.window.height),
            );
        }
    }

    function applyLivePreview() {
        const root = document.documentElement;
        const c = config.appearance.colors;

        root.style.setProperty("--vanta-bg", c.background);
        root.style.setProperty("--vanta-surface", c.surface);
        root.style.setProperty("--vanta-border", c.border);
        root.style.setProperty("--vanta-text", c.text_primary);
        root.style.setProperty("--vanta-text-dim", c.text_secondary);
        root.style.setProperty("--vanta-accent", c.accent);

        root.style.setProperty(
            "--vanta-radius",
            `${config.appearance.border_radius}px`,
        );
        root.style.setProperty(
            "--vanta-blur",
            `${config.appearance.blur_radius}px`,
        );
        root.style.setProperty(
            "--vanta-opacity",
            `${config.appearance.opacity}`,
        );
    }

    // Initial apply
    onMount(() => {
        applyLivePreview();
    });

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Escape") {
            e.stopPropagation();
            onClose();
        }
    }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings-container">
    <header>
        <h2>Settings</h2>
        <div class="actions">
            <button class="close-btn" onclick={onClose}>Done</button>
        </div>
    </header>

    <div class="sections">
        <!-- Theme Section -->
        <section>
            <h3>Theme</h3>

            <div class="control-group">
                <label
                    >Background Color
                    <input
                        type="color"
                        bind:value={config.appearance.colors.background}
                        oninput={debouncedSave}
                    />
                </label>
                <label
                    >Surface Color
                    <input
                        type="color"
                        bind:value={config.appearance.colors.surface}
                        oninput={debouncedSave}
                    />
                </label>
                <label
                    >Accent Color
                    <input
                        type="color"
                        bind:value={config.appearance.colors.accent}
                        oninput={debouncedSave}
                    />
                </label>
                <label
                    >Border Color
                    <input
                        type="text"
                        bind:value={config.appearance.colors.border}
                        oninput={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label
                    >Opacity ({config.appearance.opacity})
                    <input
                        type="range"
                        min="0.1"
                        max="1"
                        step="0.05"
                        bind:value={config.appearance.opacity}
                        oninput={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label
                    >Border Radius ({config.appearance.border_radius}px)
                    <input
                        type="range"
                        min="0"
                        max="50"
                        step="1"
                        bind:value={config.appearance.border_radius}
                        oninput={debouncedSave}
                    />
                </label>
            </div>

            <div class="control-group">
                <label
                    >Blur Radius ({config.appearance.blur_radius}px)
                    <input
                        type="range"
                        min="0"
                        max="100"
                        step="1"
                        bind:value={config.appearance.blur_radius}
                        oninput={debouncedSave}
                    />
                </label>
            </div>
        </section>

        <!-- Window Section -->
        <section>
            <h3>Window</h3>
            <div class="control-group">
                <label
                    >Width
                    <input
                        type="number"
                        min="400"
                        max="1920"
                        bind:value={config.window.width}
                        oninput={() => {
                            debouncedSave();
                            updateWindowSize();
                        }}
                    />
                </label>
            </div>
            <div class="control-group">
                <label
                    >Height
                    <input
                        type="number"
                        min="300"
                        max="1080"
                        bind:value={config.window.height}
                        oninput={() => {
                            debouncedSave();
                            updateWindowSize();
                        }}
                    />
                </label>
            </div>
        </section>

        <!-- General Section -->
        <section>
            <h3>General</h3>
            <div class="control-group">
                <label
                    >Max Results
                    <input
                        type="number"
                        min="1"
                        max="20"
                        bind:value={config.general.max_results}
                        oninput={debouncedSave}
                    />
                </label>
            </div>
            <div class="control-group">
                <label
                    >Hotkey (Restart Required)
                    <input
                        type="text"
                        bind:value={config.general.hotkey}
                        oninput={debouncedSave}
                    />
                </label>
            </div>
        </section>

        <!-- File Search Section -->
        <section>
            <h3>File Search</h3>
            <div class="control-group">
                <label>
                    Include Hidden Files
                    <input
                        type="checkbox"
                        bind:checked={config.files.include_hidden}
                        onchange={debouncedSave}
                    />
                </label>
            </div>
            <div class="control-group">
                <label>
                    Max Depth ({config.files.max_depth})
                    <input
                        type="range"
                        min="1"
                        max="10"
                        step="1"
                        bind:value={config.files.max_depth}
                        oninput={debouncedSave}
                    />
                </label>
            </div>
        </section>
    </div>
</div>

<style>
    .settings-container {
        padding: 24px;
        height: 100%;
        display: flex;
        flex-direction: column;
        color: var(--vanta-text);
    }

    header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 24px;
    }

    h2 {
        font-size: 20px;
        font-weight: 600;
        margin: 0;
    }

    h3 {
        font-size: 14px;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        color: var(--vanta-text-dim);
        margin: 24px 0 12px 0;
        border-bottom: 1px solid var(--vanta-border);
        padding-bottom: 8px;
    }

    .sections {
        flex: 1;
        overflow-y: auto;
        padding-right: 8px;
    }

    /* Hide scrollbar in settings too */
    .sections::-webkit-scrollbar {
        display: none;
    }
    .sections {
        scrollbar-width: none;
    }

    .control-group {
        display: flex;
        flex-direction: column;
        gap: 12px;
        margin-bottom: 16px;
    }

    label {
        display: flex;
        justify-content: space-between;
        align-items: center;
        font-size: 14px;
    }

    input[type="text"],
    input[type="number"] {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid var(--vanta-border);
        color: var(--vanta-text);
        padding: 4px 8px;
        border-radius: 6px;
        width: 120px;
        text-align: right;
    }

    input[type="color"] {
        background: none;
        border: none;
        width: 40px;
        height: 24px;
        cursor: pointer;
    }

    input[type="range"] {
        width: 150px;
        accent-color: var(--vanta-accent);
    }

    .close-btn {
        background: var(--vanta-surface);
        border: 1px solid var(--vanta-border);
        color: var(--vanta-text);
        padding: 6px 12px;
        border-radius: 6px;
        cursor: pointer;
        font-size: 13px;
        transition: all 0.2s;
    }

    .close-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        border-color: var(--vanta-accent);
    }
</style>
