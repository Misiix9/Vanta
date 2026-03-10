<script lang="ts">
  import { emit, listen } from "@tauri-apps/api/event";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
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
  }

  type SpotifyCommand = "prev" | "next" | "play-pause" | "set-volume";

  const NOW_PLAYING_RELAY_EVENT = "spotify-now-playing-relay";
  const COMMAND_RELAY_EVENT = "spotify-command-relay";
  const NOW_PLAYING_STORAGE_KEY = "vanta.spotify.nowPlaying";

  let nowPlaying = $state<NowPlayingState | null>(null);
  let showControls = $state(false);
  let unlistenRelay: (() => void) | null = null;
  let tickTimer: number | null = null;

  function fmtTime(ms: number): string {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m + ":" + (sec < 10 ? "0" : "") + sec;
  }

  function safeLoadSnapshot(): NowPlayingState | null {
    try {
      const raw = localStorage.getItem(NOW_PLAYING_STORAGE_KEY);
      if (!raw) return null;
      return JSON.parse(raw) as NowPlayingState;
    } catch {
      return null;
    }
  }

  function sendCommand(cmd: SpotifyCommand, value?: number) {
    const detail = cmd === "set-volume" ? { cmd, value } : cmd;
    window.dispatchEvent(new CustomEvent("vanta-spotify-command", { detail }));
    void emit(COMMAND_RELAY_EVENT, detail).catch(() => {
      // best-effort relay to main launcher window
    });
  }

  function handleNowPlaying(e: CustomEvent<NowPlayingState | null>) {
    nowPlaying = e.detail;
  }

  async function startDrag() {
    try {
      const win = getCurrentWebviewWindow();
      await win.startDragging();
    } catch {
      // ignored
    }
  }

  async function closeWindow() {
    try {
      const win = getCurrentWebviewWindow();
      await win.close();
    } catch {
      // ignored
    }
  }

  const hasTrack = $derived(Boolean(nowPlaying?.track || nowPlaying?.isPlaying));
  const lyricsText = $derived((nowPlaying?.lyrics || "").trim());
  const lyricLines = $derived(
    lyricsText
      .split("\n")
      .map((line: string) => line.trim())
      .filter((line: string) => line.length > 0)
      .slice(0, 10),
  );

  onMount(async () => {
    window.addEventListener("vanta-now-playing", handleNowPlaying as EventListener);

    const existing = (window as any).__vanta_now_playing;
    if (existing && (existing.track || existing.isPlaying)) {
      nowPlaying = existing;
    } else {
      const stored = safeLoadSnapshot();
      if (stored) nowPlaying = stored;
    }

    unlistenRelay = await listen<NowPlayingState | null>(NOW_PLAYING_RELAY_EVENT, (event) => {
      nowPlaying = event.payload;
      (window as any).__vanta_now_playing = event.payload;
    });

    tickTimer = window.setInterval(() => {
      if (!nowPlaying || !nowPlaying.isPlaying || nowPlaying.durationMs <= 0) return;
      nowPlaying = {
        ...nowPlaying,
        progressMs: Math.min(nowPlaying.progressMs + 1000, nowPlaying.durationMs),
      };
    }, 1000);
  });

  onDestroy(() => {
    window.removeEventListener("vanta-now-playing", handleNowPlaying as EventListener);
    unlistenRelay?.();
    if (tickTimer !== null) {
      window.clearInterval(tickTimer);
      tickTimer = null;
    }
  });
</script>

<div class="mini-player-root">
  {#if nowPlaying?.albumArt}
    <div class="mini-player-backdrop" style={`background-image:url('${nowPlaying.albumArt}')`}></div>
  {/if}

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="mini-player-drag" onmousedown={startDrag}>
    <span class="mini-player-title">Spotify Mini Player</span>
    <button class="mini-player-close" onclick={closeWindow} aria-label="Close">
      <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
    </button>
  </div>

  {#if hasTrack}
    <div
      class="mini-player-content"
      role="presentation"
      onmouseenter={() => (showControls = true)}
      onmouseleave={() => (showControls = false)}
    >
      {#if !showControls}
        <div class="mini-player-lyrics-pane">
          <div class="mini-player-trackline">{nowPlaying?.track || "Now Playing"} · {nowPlaying?.artist || "Spotify"}</div>
          {#if lyricLines.length > 0}
            <div class="mini-player-lyrics-lines">
              {#each lyricLines as line}
                <div class="mini-player-lyric-line">{line}</div>
              {/each}
            </div>
          {:else}
            <div class="mini-player-lyrics-empty">Lyrics unavailable for this track.</div>
          {/if}
        </div>
      {:else}
        <div class="mini-player-controls-pane">
          {#if nowPlaying?.albumArt}
            <img class="mini-player-art" src={nowPlaying.albumArt} alt="" />
          {:else}
            <div class="mini-player-art-placeholder">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
            </div>
          {/if}
          <div class="mini-player-info">
            <span class="mini-player-track">{nowPlaying?.track || "Spotify"}</span>
            <span class="mini-player-artist">{nowPlaying?.artist || "Playback"}</span>
            <div class="mini-player-progress">
              <div class="mini-player-progress-bar">
                <div class="mini-player-progress-fill" style="width: {nowPlaying && nowPlaying.durationMs > 0 ? nowPlaying.progressMs / nowPlaying.durationMs * 100 : 0}%"></div>
              </div>
              <div class="mini-player-times">
                <span>{fmtTime(nowPlaying?.progressMs || 0)}</span>
                <span>{fmtTime(nowPlaying?.durationMs || 0)}</span>
              </div>
            </div>
            <div class="mini-player-controls">
              <button class="mini-player-ctrl" onclick={() => sendCommand("prev")} aria-label="Previous">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
              </button>
              <button class="mini-player-ctrl-main" onclick={() => sendCommand("play-pause")} aria-label="Play/Pause">
                {#if nowPlaying?.isPlaying}
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>
                {:else}
                  <svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
                {/if}
              </button>
              <button class="mini-player-ctrl" onclick={() => sendCommand("next")} aria-label="Next">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><path d="M16 6h2v12h-2zm-10 6l8.5 6V6z" transform="rotate(180 12 12)"/></svg>
              </button>
            </div>
            <div class="mini-player-volume">
              <input
                class="mini-player-volume-slider"
                type="range"
                min="0"
                max="100"
                value={Math.max(0, Math.min(100, Math.round(nowPlaying?.volumePercent ?? 100)))}
                oninput={(event) => sendCommand("set-volume", Number((event.currentTarget as HTMLInputElement).value))}
                aria-label="Spotify volume"
              />
            </div>
          </div>
        </div>
      {/if}
    </div>
  {:else}
    <div class="mini-player-empty">
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
      <span>Nothing playing</span>
    </div>
  {/if}
</div>
