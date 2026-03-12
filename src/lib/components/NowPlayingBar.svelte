<script lang="ts">
  import { emit, listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  interface NowPlayingState {
    track: string | null;
    artist: string | null;
    album: string | null;
    albumArt: string | null;
    isPlaying: boolean;
    progressMs: number;
    durationMs: number;
    volumePercent?: number;
    lyrics?: string | null;
    syncedLines?: { time: number; text: string }[] | null;
  }

  type SpotifyCommand = "prev" | "next" | "play-pause" | "set-volume";

  const NOW_PLAYING_RELAY_EVENT = "spotify-now-playing-relay";
  const COMMAND_RELAY_EVENT = "spotify-command-relay";
  const REQUEST_STATE_EVENT = "spotify-request-state";
  const NOW_PLAYING_STORAGE_KEY = "vanta.spotify.nowPlaying";

  let { onOpenExtension }: { onOpenExtension?: () => void } = $props();

  let nowPlaying = $state<NowPlayingState | null>(null);
  let unlistenCommandRelay: (() => void) | null = null;
  let unlistenRequestState: (() => void) | null = null;
  let tickTimer: number | null = null;

  function safeStoreNowPlaying(snapshot: NowPlayingState | null) {
    try {
      if (snapshot) {
        localStorage.setItem(NOW_PLAYING_STORAGE_KEY, JSON.stringify(snapshot));
      } else {
        localStorage.removeItem(NOW_PLAYING_STORAGE_KEY);
      }
    } catch {
      // ignore storage failures in restricted environments
    }
  }

  function loadStoredSnapshot(): NowPlayingState | null {
    try {
      const raw = localStorage.getItem(NOW_PLAYING_STORAGE_KEY);
      if (!raw) return null;
      return JSON.parse(raw) as NowPlayingState;
    } catch {
      return null;
    }
  }

  function handleNowPlaying(e: CustomEvent<NowPlayingState | null>) {
    nowPlaying = e.detail;
    safeStoreNowPlaying(e.detail);
    void emit(NOW_PLAYING_RELAY_EVENT, e.detail).catch(() => {
      // best-effort cross-window relay
    });
  }

  function sendCommand(cmd: SpotifyCommand, value?: number) {
    const detail = cmd === "set-volume" ? { cmd, value } : cmd;
    window.dispatchEvent(new CustomEvent("vanta-spotify-command", { detail }));
  }

  function fmtTime(ms: number): string {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m + ":" + (sec < 10 ? "0" : "") + sec;
  }

  const hasTrack = $derived(Boolean(nowPlaying?.track || nowPlaying?.isPlaying));

  onMount(async () => {
    window.addEventListener("vanta-now-playing", handleNowPlaying as EventListener);

    const existing = (window as any).__vanta_now_playing;
    if (existing && (existing.track || existing.isPlaying)) {
      nowPlaying = existing;
      safeStoreNowPlaying(existing);
    } else {
      const stored = loadStoredSnapshot();
      if (stored) nowPlaying = stored;
    }

    unlistenCommandRelay = await listen(COMMAND_RELAY_EVENT, (event) => {
      window.dispatchEvent(
        new CustomEvent("vanta-spotify-command", {
          detail: event.payload,
        }),
      );
    });

    unlistenRequestState = await listen(REQUEST_STATE_EVENT, () => {
      if (nowPlaying) {
        void emit(NOW_PLAYING_RELAY_EVENT, nowPlaying).catch(() => {});
      }
    });

    tickTimer = window.setInterval(() => {
      if (!nowPlaying || !nowPlaying.isPlaying || nowPlaying.durationMs <= 0) return;
      nowPlaying = {
        ...nowPlaying,
        progressMs: Math.min(nowPlaying.progressMs + 1000, nowPlaying.durationMs),
      };
      safeStoreNowPlaying(nowPlaying);
      void emit(NOW_PLAYING_RELAY_EVENT, nowPlaying).catch(() => {});
    }, 1000);
  });

  onDestroy(() => {
    window.removeEventListener("vanta-now-playing", handleNowPlaying as EventListener);
    unlistenCommandRelay?.();
    unlistenRequestState?.();
    if (tickTimer !== null) {
      window.clearInterval(tickTimer);
      tickTimer = null;
    }
  });
</script>

{#if hasTrack}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="now-playing-bar" onclick={(e) => {
    const target = e.target as HTMLElement;
    if (target.closest('.npb-ctrl, .npb-ctrl-play, .npb-volume, .npb-volume-slider')) return;
    onOpenExtension?.();
  }} style="cursor:pointer">
    <!-- Blurred album art background -->
    {#if nowPlaying?.albumArt}
      <div class="npb-bg" style="background-image: url({nowPlaying.albumArt})"></div>
    {/if}
    <div class="npb-bg-overlay"></div>

    <!-- Content -->
    <div class="npb-content">
      {#if nowPlaying?.albumArt}
        <img class="npb-art" src={nowPlaying.albumArt} alt="" />
      {:else}
        <div class="npb-art-placeholder">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
        </div>
      {/if}

      <div class="npb-info">
        <span class="npb-track">{nowPlaying?.track || "Spotify"}</span>
        <span class="npb-artist">{nowPlaying?.artist || "Playback active"}</span>
      </div>

      <div class="npb-time">{fmtTime(nowPlaying?.progressMs || 0)} / {fmtTime(nowPlaying?.durationMs || 0)}</div>

      <div class="npb-controls">
        <button class="npb-ctrl" onclick={() => sendCommand("prev")} aria-label="Previous">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
        </button>
        <button class="npb-ctrl npb-ctrl-play" onclick={() => sendCommand("play-pause")} aria-label="Play/Pause">
          {#if nowPlaying?.isPlaying}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
          {:else}
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
          {/if}
        </button>
        <button class="npb-ctrl" onclick={() => sendCommand("next")} aria-label="Next">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor"><path d="M16 6h2v12h-2zm-10 6l8.5 6V6z" transform="rotate(180 12 12)"/></svg>
        </button>
      </div>

      <div class="npb-volume" title="Volume">
        <input
          class="npb-volume-slider"
          type="range"
          min="0"
          max="100"
          value={Math.max(0, Math.min(100, Math.round(nowPlaying?.volumePercent ?? 100)))}
          oninput={(event) => sendCommand("set-volume", Number((event.currentTarget as HTMLInputElement).value))}
          aria-label="Spotify volume"
        />
      </div>
    </div>

    <!-- Progress bar -->
    {#if nowPlaying?.durationMs && nowPlaying.durationMs > 0}
      <div class="npb-progress">
        <div class="npb-progress-fill" style="width: {(nowPlaying.progressMs / nowPlaying.durationMs) * 100}%"></div>
      </div>
    {/if}
  </div>
{/if}
