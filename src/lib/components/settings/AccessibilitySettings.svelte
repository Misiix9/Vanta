<script lang="ts">
    import type { VantaConfig } from "$lib/types";

    let {
        config = $bindable(),
        onSave,
    }: {
        config: VantaConfig;
        onSave: () => void;
    } = $props();

    function applyAccessibilityPreset(preset: "focus" | "readability" | "balanced") {
        if (preset === "focus") {
            config.accessibility.reduced_motion = true;
            config.accessibility.text_scale = 1.05;
            config.accessibility.spacing_preset = "compact";
        } else if (preset === "readability") {
            config.accessibility.reduced_motion = true;
            config.accessibility.text_scale = 1.2;
            config.accessibility.spacing_preset = "relaxed";
        } else {
            config.accessibility.reduced_motion = false;
            config.accessibility.text_scale = 1.0;
            config.accessibility.spacing_preset = "comfortable";
        }
        onSave();
    }

    function onTextScaleChange(value: number) {
        config.accessibility.text_scale = Math.max(0.85, Math.min(1.4, value));
        onSave();
    }
</script>

<div class="preset-row">
    <button class="preset-btn" onclick={() => applyAccessibilityPreset("focus")}>Focus</button>
    <button class="preset-btn" onclick={() => applyAccessibilityPreset("readability")}>Readability</button>
    <button class="preset-btn" onclick={() => applyAccessibilityPreset("balanced")}>Balanced</button>
</div>

<div class="control-group">
    <label>Reduced Motion
        <input type="checkbox" bind:checked={config.accessibility.reduced_motion} onchange={onSave} />
    </label>
</div>

<div class="control-group">
    <label>Text Scale ({config.accessibility.text_scale.toFixed(2)}x)
        <input type="range" min="0.85" max="1.40" step="0.05" value={config.accessibility.text_scale}
            oninput={(e) => onTextScaleChange(Number((e.target as HTMLInputElement).value))} />
    </label>
</div>

<div class="control-group">
    <label>Spacing Preset
        <select class="vanta-select" bind:value={config.accessibility.spacing_preset} onchange={onSave}>
            <option value="compact">Compact</option>
            <option value="comfortable">Comfortable</option>
            <option value="relaxed">Relaxed</option>
        </select>
    </label>
</div>
