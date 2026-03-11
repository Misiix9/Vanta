<script lang="ts">
    let {
        lines = 3,
        width = "100%",
    }: {
        lines?: number;
        width?: string;
    } = $props();
</script>

<div class="loading-skeleton" style="width: {width};" aria-busy="true" aria-label="Loading">
    {#each Array(lines) as _, i}
        <div
            class="skeleton-line"
            style="width: {i === lines - 1 ? '60%' : '100%'}; animation-delay: {i * 80}ms;"
        ></div>
    {/each}
</div>

<style>
    .loading-skeleton {
        display: flex;
        flex-direction: column;
        gap: 10px;
        padding: 12px 0;
    }

    .skeleton-line {
        height: 14px;
        border-radius: 6px;
        background: linear-gradient(
            90deg,
            var(--vanta-surface, rgba(255, 255, 255, 0.04)) 25%,
            var(--vanta-border, rgba(255, 255, 255, 0.08)) 50%,
            var(--vanta-surface, rgba(255, 255, 255, 0.04)) 75%
        );
        background-size: 200% 100%;
        animation: shimmer 1.5s ease-in-out infinite;
    }

    @keyframes shimmer {
        0% { background-position: 200% 0; }
        100% { background-position: -200% 0; }
    }
</style>
