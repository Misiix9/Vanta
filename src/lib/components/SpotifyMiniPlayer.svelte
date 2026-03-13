<script lang="ts">
  import { emit, listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onDestroy, onMount } from "svelte";

  interface SyncedLine {
    time: number;
    text: string;
  }

  interface NowPlayingState {
    track: string | null;
    artist: string | null;
    album: string | null;
    albumArt: string | null;
    isPlaying: boolean;
    progressMs: number;
    durationMs: number;
    updatedAt?: number;
    volumePercent?: number;
    lyrics?: string | null;
    syncedLines?: SyncedLine[] | null;
  }

  type SpotifyCommand = "prev" | "next" | "play-pause" | "set-volume";

  const NOW_PLAYING_RELAY_EVENT = "spotify-now-playing-relay";
  const COMMAND_RELAY_EVENT = "spotify-command-relay";
  const REQUEST_STATE_EVENT = "spotify-request-state";
  const LYRICS_LEAD_MS = 0;

  let nowPlaying = $state<NowPlayingState | null>(null);
  let unlistenRelay: (() => void) | null = null;
  let lastUpdatedAt = $state(0);
  let isMaximized = $state(false);

  function fmtTime(ms: number): string {
    const s = Math.floor(ms / 1000);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return m + ":" + (sec < 10 ? "0" : "") + sec;
  }

  function sendCommand(cmd: SpotifyCommand, value?: number) {
    const detail = cmd === "set-volume" ? { cmd, value } : cmd;
    window.dispatchEvent(new CustomEvent("vanta-spotify-command", { detail }));
    void emit(COMMAND_RELAY_EVENT, detail).catch(() => {
      // best-effort relay to main launcher window
    });
  }

  function handleNowPlaying(e: CustomEvent<NowPlayingState | null>) {
    const incoming = e.detail;
    if (!incoming) {
      nowPlaying = null;
      lastUpdatedAt = 0;
      return;
    }
    const ts = Number(incoming.updatedAt || Date.now());
    if (ts < lastUpdatedAt) return;
    lastUpdatedAt = ts;
    nowPlaying = incoming;
  }

  async function startDrag() {
    try {
      const win = getCurrentWindow();
      await win.startDragging();
    } catch {
      // ignored
    }
  }

  async function closeWindow() {
    try {
      const win = getCurrentWindow();
      await win.close();
    } catch {
      // ignored
    }
  }

  async function toggleFullscreen() {
    try {
      const win = getCurrentWindow();
      const maximized = await win.isMaximized();
      if (maximized) {
        await win.unmaximize();
        isMaximized = false;
      } else {
        await win.maximize();
        isMaximized = true;
      }
    } catch {
      // ignored
    }
  }

  const hasTrack = $derived(Boolean(nowPlaying?.track || nowPlaying?.isPlaying));
  const hasSyncedLines = $derived(
    Boolean(nowPlaying?.syncedLines && nowPlaying.syncedLines.length > 0),
  );
  const activeLyricIndex = $derived(
    !nowPlaying?.syncedLines || nowPlaying.syncedLines.length === 0
      ? -1
      : nowPlaying.syncedLines.reduce(
          (acc: number, line: SyncedLine, i: number) =>
            line.time <= Math.max(0, (nowPlaying?.progressMs || 0) + LYRICS_LEAD_MS) ? i : acc,
          0,
        ),
  );
  const stableLyricIndex = $derived(
    hasSyncedLines
      ? Math.max(0, Math.min(activeLyricIndex, (nowPlaying?.syncedLines?.length || 1) - 1))
      : -1,
  );
  const previousLyricText = $derived(
    hasSyncedLines && stableLyricIndex > 0
      ? nowPlaying?.syncedLines?.[stableLyricIndex - 1]?.text || ""
      : "",
  );
  const activeLyricText = $derived(
    hasSyncedLines ? nowPlaying?.syncedLines?.[stableLyricIndex]?.text || "" : "",
  );
  const nextLyricText = $derived(
    hasSyncedLines
      ? nowPlaying?.syncedLines?.[Math.min(stableLyricIndex + 1, (nowPlaying?.syncedLines?.length || 1) - 1)]?.text || ""
      : "",
  );
  const lyricsText = $derived((nowPlaying?.lyrics || "").trim());
  const lyricLines = $derived(
    lyricsText
      .split("\n")
      .map((line: string) => line.replace(/^\[\d{2}:\d{2}\.\d{2,3}\]\s*/, "").trim())
      .filter((line: string) => line.length > 0)
      .slice(0, 10),
  );

  const hasLyrics = $derived(hasSyncedLines || lyricLines.length > 0);
  
  const detectedFontFamily = $derived((() => {
    if (!lyricsText) return "";
    if (/[\uac00-\ud7af\u1100-\u11ff]/.test(lyricsText)) return "'Noto Sans CJK KR', 'Malgun Gothic', sans-serif";
    if (/[\u3040-\u30ff\u3400-\u4dbf\u4e00-\u9fff]/.test(lyricsText)) return "'Noto Sans CJK JP', 'Noto Sans CJK SC', 'Noto Sans CJK TC', 'Hiragino Sans', sans-serif";
    if (/[\u0600-\u06ff]/.test(lyricsText)) return "'Noto Kufi Arabic', Arial, sans-serif";
    if (/[\u0400-\u04ff]/.test(lyricsText)) return "'Noto Sans', Arial, sans-serif";
    return "";
  })());
  
  const volumeValue = $derived(Math.max(0, Math.min(100, Math.round(nowPlaying?.volumePercent ?? 100))));

  onMount(async () => {
    window.addEventListener("vanta-now-playing", handleNowPlaying as EventListener);

    try {
      const win = getCurrentWindow();
      isMaximized = await win.isMaximized();
      win.onResized(async () => {
        try {
          isMaximized = await win.isMaximized();
        } catch {}
      });
    } catch {
      isMaximized = false;
    }

    unlistenRelay = await listen<NowPlayingState | null>(NOW_PLAYING_RELAY_EVENT, (event) => {
      handleNowPlaying({ detail: event.payload } as CustomEvent<NowPlayingState | null>);
    });

    // Request current state from NowPlayingBar immediately
    void emit(REQUEST_STATE_EVENT, {}).catch(() => {});
    setTimeout(() => void emit(REQUEST_STATE_EVENT, {}).catch(() => {}), 800);
  });

  onDestroy(() => {
    window.removeEventListener("vanta-now-playing", handleNowPlaying as EventListener);
    unlistenRelay?.();
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="mini-player-root"
  class:is-maximized={isMaximized}
  ondblclick={() => {
    if (!isMaximized) void toggleFullscreen();
  }}
>
  {#if nowPlaying?.albumArt}
    <div class="mini-player-backdrop" style={`background-image:url('${nowPlaying.albumArt}')`}></div>
  {/if}
  <div class="mini-player-overlay"></div>

  {#if isMaximized}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="mini-player-drag" onmousedown={startDrag}>
      <span class="mini-player-title">Mini Player</span>
      <div class="mini-player-window-actions">
        <button class="mini-player-action-btn" onclick={toggleFullscreen} aria-label="Toggle fullscreen">
          {#if isMaximized}
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 3H5a2 2 0 0 0-2 2v3"/><path d="M16 3h3a2 2 0 0 1 2 2v3"/><path d="M8 21H5a2 2 0 0 1-2-2v-3"/><path d="M16 21h3a2 2 0 0 0 2-2v-3"/></svg>
          {:else}
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/><line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/></svg>
          {/if}
        </button>
        <button class="mini-player-close" onclick={closeWindow} aria-label="Close">
          <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>
        </button>
      </div>
    </div>
  {/if}

  {#if hasTrack}
    <div class="mini-player-content" class:has-lyrics={hasLyrics}>
      <div class="mini-player-main-pane">
        {#if isMaximized}
          <div class="mini-player-volume-rail-wrap">
            <div class="mini-player-volume-rail-hitbox" title="Volume">
              <div class="mini-player-volume-rail-shell">
                <input
                  class="mini-player-volume-rail"
                  type="range"
                  min="0"
                  max="100"
                  value={volumeValue}
                  oninput={(event) => sendCommand("set-volume", Number((event.currentTarget as HTMLInputElement).value))}
                  aria-label="Spotify volume"
                />
              </div>
            </div>
          </div>
        {/if}

        <div class="mini-player-main-block">
          <div class="mini-player-top">
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

          <div class="mini-player-progress">
            <div class="mini-player-progress-bar">
              <div class="mini-player-progress-fill" style="width: {nowPlaying && nowPlaying.durationMs > 0 ? nowPlaying.progressMs / nowPlaying.durationMs * 100 : 0}%"></div>
            </div>
            <div class="mini-player-times">
              <span>{fmtTime(nowPlaying?.progressMs || 0)}</span>
              <span>{fmtTime(nowPlaying?.durationMs || 0)}</span>
            </div>
          </div>
        </div>
      </div>

      {#if isMaximized && hasLyrics}
        <div class="mini-player-lyrics-pane" style={detectedFontFamily ? `font-family: ${detectedFontFamily}` : ""}>
          {#if hasSyncedLines}
            <div class="mini-player-lyrics-lines mini-player-lyrics-static">
              {#if previousLyricText && previousLyricText !== activeLyricText}
                <div class="mini-player-lyric-line mini-player-lyric-muted">{previousLyricText}</div>
              {/if}
              <div class="mini-player-lyric-line mini-player-lyric-active">{activeLyricText}</div>
              {#if nextLyricText && nextLyricText !== activeLyricText}
                <div class="mini-player-lyric-line mini-player-lyric-muted">{nextLyricText}</div>
              {/if}
            </div>
          {:else if lyricLines.length > 0}
            <div class="mini-player-lyrics-lines mini-player-lyrics-static">
              <div class="mini-player-lyric-line mini-player-lyric-active">{lyricLines[0]}</div>
            </div>
          {/if}
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
