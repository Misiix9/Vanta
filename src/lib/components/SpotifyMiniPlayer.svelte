<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

  interface NowPlayingState {
    track: string | null;
    artist: string | null;
    album: string | null;
    albumArt: string | null;
    isPlaying: boolean;
    progressMs: number;
    durationMs: number;
  }

  let nowPlaying: NowPlayingState | null = $state(null);
  let isDragging = $state(false);
  let dragStartX = 0;
  let dragStartY = 0;

  function fmtTime(ms: number): string {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m + ':' + (sec < 10 ? '0' : '') + sec;
  }

  function sendCommand(cmd: string) {
    window.dispatchEvent(new CustomEvent('vanta-spotify-command', { detail: cmd }));
  }

  function handleNowPlaying(e: CustomEvent<NowPlayingState | null>) {
    nowPlaying = e.detail;
  }

  async function startDrag(e: MouseEvent) {
    try {
      const win = getCurrentWebviewWindow();
      await win.startDragging();
    } catch { /* fallback */ }
  }

  async function closeWindow() {
    try {
      const win = getCurrentWebviewWindow();
      await win.close();
    } catch { /* ignore */ }
  }

  onMount(() => {
    window.addEventListener('vanta-now-playing', handleNowPlaying as EventListener);
    const existing = (window as any).__vanta_now_playing;
    if (existing && existing.track) nowPlaying = existing;
  });

  onDestroy(() => {
    window.removeEventListener('vanta-now-playing', handleNowPlaying as EventListener);
  });
</script>

<div class="mini-player-root">
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="mini-player-drag" onmousedown={startDrag}>
    <span class="mini-player-title">Spotify</span>
    <button class="mini-player-close" onclick={closeWindow} aria-label="Close">
      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
    </button>
  </div>

  {#if nowPlaying && nowPlaying.track}
    <div class="mini-player-content">
      {#if nowPlaying.albumArt}
        <img class="mini-player-art" src={nowPlaying.albumArt} alt="" />
      {:else}
        <div class="mini-player-art-placeholder">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
        </div>
      {/if}
      <div class="mini-player-info">
        <span class="mini-player-track">{nowPlaying.track}</span>
        <span class="mini-player-artist">{nowPlaying.artist}</span>
        <div class="mini-player-progress">
          <div class="mini-player-progress-bar">
            <div class="mini-player-progress-fill" style="width: {nowPlaying.durationMs > 0 ? (nowPlaying.progressMs / nowPlaying.durationMs * 100) : 0}%"></div>
          </div>
          <div class="mini-player-times">
            <span>{fmtTime(nowPlaying.progressMs)}</span>
            <span>{fmtTime(nowPlaying.durationMs)}</span>
          </div>
        </div>
        <div class="mini-player-controls">
          <button class="mini-player-ctrl" onclick={() => sendCommand('prev')} aria-label="Previous">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
          </button>
          <button class="mini-player-ctrl-main" onclick={() => sendCommand('play-pause')} aria-label="Play/Pause">
            {#if nowPlaying.isPlaying}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
            {:else}
              <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
            {/if}
          </button>
          <button class="mini-player-ctrl" onclick={() => sendCommand('next')} aria-label="Next">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M16 6h2v12h-2zm-10 6l8.5 6V6z" transform="rotate(180 12 12)"/></svg>
          </button>
        </div>
      </div>
    </div>
  {:else}
    <div class="mini-player-empty">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
      <span>Nothing playing</span>
    </div>
  {/if}
</div>
