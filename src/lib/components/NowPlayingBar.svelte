<script lang="ts">
  import { onDestroy, onMount } from 'svelte';

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

  function handleNowPlaying(e: CustomEvent<NowPlayingState | null>) {
    nowPlaying = e.detail;
  }

  function sendCommand(cmd: string) {
    window.dispatchEvent(new CustomEvent('vanta-spotify-command', { detail: cmd }));
  }

  function fmtTime(ms: number): string {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m + ':' + (sec < 10 ? '0' : '') + sec;
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

{#if nowPlaying && nowPlaying.track}
  <div class="now-playing-bar">
    {#if nowPlaying.albumArt}
      <img class="npb-art" src={nowPlaying.albumArt} alt="" />
    {:else}
      <div class="npb-art-placeholder">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
      </div>
    {/if}
    <div class="npb-info">
      <span class="npb-track">{nowPlaying.track}</span>
      <span class="npb-artist">{nowPlaying.artist}</span>
    </div>
    <div class="npb-time">{fmtTime(nowPlaying.progressMs)} / {fmtTime(nowPlaying.durationMs)}</div>
    <div class="npb-controls">
      <button class="npb-ctrl" onclick={() => sendCommand('prev')} aria-label="Previous">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
      </button>
      <button class="npb-ctrl npb-ctrl-play" onclick={() => sendCommand('play-pause')} aria-label="Play/Pause">
        {#if nowPlaying.isPlaying}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
        {:else}
          <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
        {/if}
      </button>
      <button class="npb-ctrl" onclick={() => sendCommand('next')} aria-label="Next">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M16 6h2v12h-2zm-10 6l8.5 6V6z" transform="rotate(180 12 12)"/></svg>
      </button>
    </div>
  </div>
{/if}
