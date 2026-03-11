<script lang="ts">
    let {
        title = "Something went wrong",
        message = "An error occurred while loading.",
        onRetry,
    }: {
        title?: string;
        message?: string;
        onRetry: () => void;
    } = $props();
</script>

<div class="error-recovery" role="alert">
    <div class="error-icon">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor"
            stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
        </svg>
    </div>
    <h3 class="error-title">{title}</h3>
    <p class="error-message">{message}</p>
    <button class="error-retry-btn" onclick={onRetry}>Try Again</button>
</div>

<style>
    .error-recovery {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 12px;
        padding: 32px 24px;
        text-align: center;
        min-height: 200px;
    }

    .error-icon {
        color: var(--vanta-accent, #fff);
        opacity: 0.6;
    }

    .error-title {
        font-size: 1rem;
        font-weight: 600;
        color: var(--vanta-text-primary, #f5f5f5);
        margin: 0;
    }

    .error-message {
        font-size: 0.85rem;
        color: var(--vanta-text-secondary, #888);
        margin: 0;
        max-width: 320px;
    }

    .error-retry-btn {
        margin-top: 8px;
        padding: 8px 20px;
        border-radius: 8px;
        border: 1px solid var(--vanta-border, rgba(255, 255, 255, 0.08));
        background: var(--vanta-surface, rgba(255, 255, 255, 0.04));
        color: var(--vanta-accent, #fff);
        font-size: 0.85rem;
        cursor: pointer;
        transition: background 0.15s ease, border-color 0.15s ease;
    }

    .error-retry-btn:hover {
        background: var(--vanta-accent, #fff);
        color: var(--vanta-background, #000);
    }
</style>
