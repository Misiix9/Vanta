(function(vanta) {
  function SpotifyPlayer($$anchor, $$props) {
    var api = $$props.api;
    var token = null;
    var refreshToken = null;
    var clientId = null;
    var codeVerifier = null;
    var REDIRECT_URI = 'http://127.0.0.1:8888/callback';
    var refreshTimer = null;
    var progressTimer = null;
    var currentTrack = null;
    var isPlaying = false;
    var progressMs = 0;
    var durationMs = 0;
    var shuffleState = false;
    var repeatState = 'off';
    var volumePercent = 100;
    var nowPlayingContainer = null;
    var progressFillEl = null;
    var progressCurEl = null;
    var playPauseBtnRef = null;
    var shuffleBtnRef = null;
    var repeatBtnRef = null;
    var searchTimeout = null;
    var albumArtUrl = null;
    var lyricsText = null;
    var lyricsFetchKey = null;
    var lyricsCache = {};

    var style = document.createElement('style');
    style.textContent = [
      '.spot-root{padding:0;font-family:-apple-system,system-ui,sans-serif;color:var(--vanta-text,#e8e8e8);min-height:200px;overflow-y:auto;max-height:520px}',
      '.spot-setup{display:flex;flex-direction:column;align-items:center;gap:14px;padding:20px 16px;text-align:center}',
      '.spot-setup-icon{opacity:.35}',
      '.spot-setup-title{font-size:18px;font-weight:600;margin-top:2px}',
      '.spot-step-label{font-size:11px;color:var(--vanta-text-dim,#888);font-weight:500;text-transform:uppercase;letter-spacing:.5px}',
      '.spot-instructions{text-align:left;font-size:12.5px;color:var(--vanta-text,#e8e8e8);line-height:1.7;background:var(--vanta-surface,#111);border:1px solid var(--vanta-border,rgba(255,255,255,0.08));border-radius:10px;padding:16px 20px;width:100%;max-width:420px;box-sizing:border-box}',
      '.spot-instructions-heading{font-weight:600;margin-bottom:8px;font-size:13px}',
      '.spot-instructions ol{margin:0;padding-left:20px}',
      '.spot-instructions li{margin-bottom:6px}',
      '.spot-instructions ul{margin:4px 0 0;padding-left:18px;list-style:disc}',
      '.spot-instructions ul li{margin-bottom:2px;color:var(--vanta-text-dim,#888);font-size:12px}',
      '.spot-instructions a{color:var(--vanta-accent,#7b35f0);text-decoration:none;cursor:pointer}',
      '.spot-instructions a:hover{text-decoration:underline}',
      '.spot-instructions code{background:rgba(255,255,255,0.08);padding:1px 6px;border-radius:3px;font-size:11px;font-family:ui-monospace,monospace}',
      '.spot-instructions strong{color:var(--vanta-text,#e8e8e8);font-weight:600}',
      '.spot-setup-desc{font-size:13px;color:var(--vanta-text-dim,#888);line-height:1.6;max-width:420px}',
      '.spot-setup-desc strong{color:var(--vanta-text,#e8e8e8)}',
      '.spot-input{width:100%;max-width:420px;padding:10px 14px;background:rgba(255,255,255,0.06);color:var(--vanta-text,#e8e8e8);border:1px solid var(--vanta-border,rgba(255,255,255,0.08));border-radius:8px;font-size:13px;outline:none;box-sizing:border-box;transition:border-color .15s}',
      '.spot-input:focus{border-color:var(--vanta-accent,#7b35f0)}',
      '.spot-input::placeholder{color:var(--vanta-text-dim,#888)}',
      '.spot-btn{background:var(--vanta-accent,#7b35f0);color:#fff;border:none;padding:9px 24px;border-radius:8px;cursor:pointer;font-size:13px;font-weight:500;transition:all .15s}',
      '.spot-btn:hover{opacity:.85}',
      '.spot-btn:active{transform:scale(.97)}',
      '.spot-btn:disabled{opacity:.5;cursor:not-allowed;transform:none}',
      '.spot-btn-outline{background:transparent;border:1px solid var(--vanta-border,rgba(255,255,255,0.08));color:var(--vanta-text-dim,#888)}',
      '.spot-btn-outline:hover{border-color:var(--vanta-accent,#7b35f0);color:var(--vanta-text,#e8e8e8);background:rgba(123,53,240,0.08)}',
      '.spot-btn-sm{font-size:11px;padding:5px 14px}',
      '.spot-btn-danger{background:transparent;border:1px solid rgba(239,68,68,0.3);color:#ef4444}',
      '.spot-btn-danger:hover{background:rgba(239,68,68,0.1);border-color:#ef4444}',

      '.spot-player{display:flex;flex-direction:column;gap:0}',
      '.spot-header{display:flex;align-items:center;justify-content:space-between;padding:12px 16px 0;gap:8px}',
      '.spot-header-left{display:flex;align-items:center;gap:8px}',
      '.spot-header-title{font-size:13px;font-weight:600;color:var(--vanta-text,#e8e8e8)}',
      '.spot-header-dot{width:6px;height:6px;border-radius:50%;background:#22c55e;flex-shrink:0}',
      '.spot-header-actions{display:flex;gap:4px}',
      '.spot-icon-btn{width:28px;height:28px;border-radius:6px;border:none;background:transparent;color:var(--vanta-text-dim,#888);cursor:pointer;display:flex;align-items:center;justify-content:center;transition:all .15s}',
      '.spot-icon-btn:hover{background:rgba(255,255,255,0.06);color:var(--vanta-text,#e8e8e8)}',
      '.spot-icon-btn.accent:hover{background:rgba(123,53,240,0.15);color:var(--vanta-accent,#7b35f0)}',

      '.spot-tabs{display:flex;gap:2px;background:rgba(255,255,255,0.03);border-radius:8px;padding:3px;margin:12px 16px 0}',
      '.spot-tab{flex:1;padding:7px 0;text-align:center;font-size:12px;font-weight:500;border:none;border-radius:6px;cursor:pointer;background:transparent;color:var(--vanta-text-dim,#888);transition:all .15s}',
      '.spot-tab.active{background:var(--vanta-accent,#7b35f0);color:#fff}',

      '.spot-art-section{display:flex;align-items:flex-start;gap:16px;padding:16px}',
      '.spot-art-img{width:120px;height:120px;border-radius:10px;object-fit:cover;flex-shrink:0;box-shadow:0 4px 24px rgba(0,0,0,.5)}',
      '.spot-art-placeholder{width:120px;height:120px;border-radius:10px;flex-shrink:0;background:rgba(255,255,255,0.04);display:flex;align-items:center;justify-content:center;color:var(--vanta-text-dim,#555)}',
      '.spot-track-info{flex:1;min-width:0;display:flex;flex-direction:column;gap:4px;padding-top:4px}',
      '.spot-track-name{font-size:16px;font-weight:700;line-height:1.3;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;color:var(--vanta-text,#f5f5f5)}',
      '.spot-track-artist{font-size:13px;color:var(--vanta-accent,#a78bfa);font-weight:500;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}',
      '.spot-track-album{font-size:12px;color:var(--vanta-text-dim,#888);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;margin-top:2px}',

      '.spot-progress-section{padding:0 16px}',
      '.spot-progress-bar{width:100%;height:4px;background:rgba(255,255,255,0.06);border-radius:2px;overflow:visible;cursor:pointer;transition:height .1s;position:relative}',
      '.spot-progress-bar:hover{height:6px}',
      '.spot-progress-fill{height:100%;background:var(--vanta-accent,#7b35f0);border-radius:2px;transition:width .3s linear;position:relative}',
      '.spot-progress-thumb{position:absolute;right:-4px;top:50%;transform:translateY(-50%);width:10px;height:10px;border-radius:50%;background:#fff;box-shadow:0 1px 4px rgba(0,0,0,.4);opacity:0;transition:opacity .15s}',
      '.spot-progress-bar:hover .spot-progress-thumb{opacity:1}',
      '.spot-progress-times{display:flex;justify-content:space-between;font-size:11px;color:var(--vanta-text-dim,#888);margin-top:6px}',

      '.spot-controls{display:flex;align-items:center;justify-content:center;gap:16px;padding:8px 16px 4px}',
      '.spot-ctrl{background:none;border:none;color:var(--vanta-text-dim,#888);cursor:pointer;padding:6px;border-radius:50%;transition:all .15s;display:flex;align-items:center;justify-content:center}',
      '.spot-ctrl:hover{color:var(--vanta-text,#e8e8e8);background:rgba(255,255,255,0.06)}',
      '.spot-ctrl.active{color:var(--vanta-accent,#7b35f0)}',
      '.spot-ctrl-main{width:44px;height:44px;background:var(--vanta-accent,#7b35f0);color:#fff;border:none;border-radius:50%;cursor:pointer;display:flex;align-items:center;justify-content:center;transition:all .15s;box-shadow:0 2px 12px rgba(123,53,240,0.35)}',
      '.spot-ctrl-main:hover{transform:scale(1.06);box-shadow:0 4px 20px rgba(123,53,240,0.5)}',
      '.spot-ctrl-main:active{transform:scale(.94)}',

      '.spot-volume{display:flex;align-items:center;gap:8px;padding:4px 16px 12px}',
      '.spot-vol-icon{color:var(--vanta-text-dim,#888);flex-shrink:0;display:flex;align-items:center}',
      '.spot-vol-bar{flex:1;height:4px;background:rgba(255,255,255,0.06);border-radius:2px;cursor:pointer;position:relative}',
      '.spot-vol-fill{height:100%;background:var(--vanta-accent,#7b35f0);border-radius:2px;transition:width .1s}',
      '.spot-lyrics{margin:0 16px 12px;padding:10px 12px;background:rgba(255,255,255,0.03);border:1px solid var(--vanta-border,rgba(255,255,255,0.08));border-radius:8px;max-height:150px;overflow:auto}',
      '.spot-lyrics-title{font-size:11px;text-transform:uppercase;letter-spacing:.6px;color:var(--vanta-text-dim,#999);margin-bottom:6px}',
      '.spot-lyrics-line{font-size:12px;line-height:1.45;color:var(--vanta-text,#e8e8e8);margin:0 0 3px}',
      '.spot-lyrics-empty{font-size:12px;color:var(--vanta-text-dim,#888)}',

      '.spot-empty{text-align:center;padding:32px 16px;color:var(--vanta-text-dim,#888);font-size:13px}',
      '.spot-empty-icon{margin-bottom:12px;opacity:.3}',
      '.spot-search-input{width:100%;padding:10px 12px;background:rgba(255,255,255,0.06);color:var(--vanta-text,#e8e8e8);border:1px solid var(--vanta-border,rgba(255,255,255,0.08));border-radius:8px;font-size:13px;outline:none;box-sizing:border-box;margin:0 0 12px}',
      '.spot-search-input:focus{border-color:var(--vanta-accent,#7b35f0)}',
      '.spot-search-input::placeholder{color:var(--vanta-text-dim,#888)}',
      '.spot-search-wrap{padding:0 16px}',

      '.spot-result{display:flex;align-items:center;gap:10px;padding:8px 16px;cursor:pointer;transition:background .1s}',
      '.spot-result:hover{background:rgba(123,53,240,0.08)}',
      '.spot-result-art{width:40px;height:40px;border-radius:4px;object-fit:cover;flex-shrink:0}',
      '.spot-result-art-placeholder{width:40px;height:40px;border-radius:4px;flex-shrink:0;background:rgba(255,255,255,0.04);display:flex;align-items:center;justify-content:center}',
      '.spot-result-info{flex:1;min-width:0}',
      '.spot-result-name{font-size:13px;font-weight:500;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;color:var(--vanta-text,#e8e8e8)}',
      '.spot-result-artist{font-size:11px;color:var(--vanta-text-dim,#888);white-space:nowrap;overflow:hidden;text-overflow:ellipsis}',
      '.spot-result-dur{font-size:11px;color:var(--vanta-text-dim,#888);flex-shrink:0}',
      '.spot-result-play{flex-shrink:0;width:28px;height:28px;border-radius:50%;background:var(--vanta-accent,#7b35f0);color:#fff;border:none;cursor:pointer;display:flex;align-items:center;justify-content:center;opacity:0;transition:opacity .15s}',
      '.spot-result:hover .spot-result-play{opacity:1}',

      '.spot-footer{display:flex;justify-content:center;gap:8px;padding:12px 16px;border-top:1px solid var(--vanta-border,rgba(255,255,255,0.06))}',
      '.spot-loading{display:flex;justify-content:center;padding:24px;color:var(--vanta-text-dim,#888);font-size:13px}',
      '.spot-err{color:#ef4444;font-size:12px;text-align:center;padding:8px 16px}',
      '.spot-hint{font-size:11px;color:var(--vanta-text-dim,#888);background:rgba(255,255,255,0.03);border:1px solid var(--vanta-border,rgba(255,255,255,0.06));border-radius:8px;padding:10px 14px;max-width:420px;line-height:1.6;text-align:left}',
      '.spot-hint strong{color:var(--vanta-text,#e8e8e8)}',
      '.spot-divider{height:1px;background:var(--vanta-border,rgba(255,255,255,0.06));margin:0}'
    ].join('\n');

    var root = document.createElement('div');
    root.className = 'spot-root';
    $$anchor.before(style);
    $$anchor.before(root);

    var svgPrev = '<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="none"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>';
    var svgNext = '<svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor" stroke="none"><path d="M16 6h2v12h-2zm-10 6l8.5 6V6z" transform="rotate(180 12 12)"/></svg>';
    var svgPlay = '<svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor" stroke="none"><path d="M8 5v14l11-7z"/></svg>';
    var svgPause = '<svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor" stroke="none"><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></svg>';
    var svgShuffle = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/><line x1="4" y1="4" x2="9" y2="9"/></svg>';
    var svgRepeat = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="17 1 21 5 17 9"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><polyline points="7 23 3 19 7 15"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/></svg>';
    var svgMusic = '<svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>';
    var svgSmallPlay = '<svg width="12" height="12" viewBox="0 0 24 24" fill="currentColor" stroke="none"><path d="M8 5v14l11-7z"/></svg>';
    var svgVolume = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/><path d="M19.07 4.93a10 10 0 0 1 0 14.14"/><path d="M15.54 8.46a5 5 0 0 1 0 7.07"/></svg>';
    var svgMiniPlayer = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/></svg>';
    var svgSettings = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>';

    function fmtTime(ms) {
      var s = Math.floor(ms / 1000); var m = Math.floor(s / 60); s = s % 60;
      return m + ':' + (s < 10 ? '0' : '') + s;
    }

    function esc(s) { var d = document.createElement('span'); d.textContent = s; return d.innerHTML; }

    function generateCodeVerifier() {
      var arr = new Uint8Array(64);
      crypto.getRandomValues(arr);
      return btoa(String.fromCharCode.apply(null, arr))
        .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    }

    async function generateCodeChallenge(verifier) {
      var encoder = new TextEncoder();
      var data = encoder.encode(verifier);
      var digest = await crypto.subtle.digest('SHA-256', data);
      return btoa(String.fromCharCode.apply(null, new Uint8Array(digest)))
        .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
    }

    async function spotApi(method, path) {
      var args = ['-s', '-w', '\n%{http_code}', '-X', method, '-H', 'Authorization: Bearer ' + token];
      if (method === 'PUT' || method === 'POST') args.push('-H', 'Content-Type: application/json');
      args.push('https://api.spotify.com/v1' + path);
      return await api.shell.execute('curl', args);
    }

    async function spotApiBody(method, path, body) {
      var args = ['-s', '-w', '\n%{http_code}', '-X', method,
        '-H', 'Authorization: Bearer ' + token, '-H', 'Content-Type: application/json',
        '-d', JSON.stringify(body)];
      args.push('https://api.spotify.com/v1' + path);
      return await api.shell.execute('curl', args);
    }

    function parseResponse(raw) {
      var lines = raw.trim().split('\n');
      var code = parseInt(lines[lines.length - 1]) || 0;
      var body = lines.slice(0, -1).join('\n').trim();
      return { code: code, body: body };
    }

    function clearTimers() {
      if (refreshTimer) { clearInterval(refreshTimer); refreshTimer = null; }
      if (progressTimer) { clearInterval(progressTimer); progressTimer = null; }
      if (searchTimeout) { clearTimeout(searchTimeout); searchTimeout = null; }
    }

    function emitNowPlaying() {
      var detail = {
        track: currentTrack ? currentTrack.name : null,
        artist: currentTrack ? currentTrack.artist : null,
        album: currentTrack ? currentTrack.album : null,
        albumArt: albumArtUrl,
        isPlaying: isPlaying,
        progressMs: progressMs,
        durationMs: durationMs,
        volumePercent: volumePercent,
        lyrics: lyricsText
      };
      window.__vanta_now_playing = detail;
      window.dispatchEvent(new CustomEvent('vanta-now-playing', { detail: detail }));
    }

    function clearNowPlaying() {
      window.__vanta_now_playing = null;
      window.dispatchEvent(new CustomEvent('vanta-now-playing', { detail: null }));
    }

    async function fetchLyrics(trackName, artistName) {
      var key = (trackName || '') + '::' + (artistName || '');
      if (!trackName || !artistName) {
        lyricsText = null;
        lyricsFetchKey = null;
        emitNowPlaying();
        return;
      }
      if (lyricsCache[key] !== undefined) {
        lyricsText = lyricsCache[key];
        lyricsFetchKey = key;
        emitNowPlaying();
        return;
      }
      lyricsFetchKey = key;
      try {
        var raw = await api.shell.execute('curl', [
          '-s',
          'https://lrclib.net/api/get?track_name=' + encodeURIComponent(trackName) + '&artist_name=' + encodeURIComponent(artistName)
        ]);
        var parsed = null;
        try {
          parsed = JSON.parse((raw || '').trim() || '{}');
        } catch (e) {
          parsed = null;
        }

        var out = null;
        if (parsed) {
          out = parsed.plainLyrics || parsed.syncedLyrics || null;
          if (out && typeof out === 'string' && out.length > 2400) out = out.slice(0, 2400);
        }

        lyricsCache[key] = out;
        if (lyricsFetchKey === key) {
          lyricsText = out;
          emitNowPlaying();
        }
      } catch (e) {
        lyricsCache[key] = null;
        if (lyricsFetchKey === key) {
          lyricsText = null;
          emitNowPlaying();
        }
      }
    }

    async function exchangeCodeForToken(code) {
      var bodyStr = 'grant_type=authorization_code' +
        '&code=' + encodeURIComponent(code) +
        '&redirect_uri=' + encodeURIComponent(REDIRECT_URI) +
        '&client_id=' + encodeURIComponent(clientId) +
        '&code_verifier=' + encodeURIComponent(codeVerifier);

      var args = ['-s', '-X', 'POST',
        '-H', 'Content-Type: application/x-www-form-urlencoded',
        '-d', bodyStr,
        'https://accounts.spotify.com/api/token'];

      var raw = await api.shell.execute('curl', args);
      var data = JSON.parse(raw.trim());
      if (data.error) throw new Error(data.error_description || data.error);
      return data;
    }

    async function refreshAccessToken() {
      if (!refreshToken || !clientId) return false;
      try {
        var bodyStr = 'grant_type=refresh_token' +
          '&refresh_token=' + encodeURIComponent(refreshToken) +
          '&client_id=' + encodeURIComponent(clientId);

        var args = ['-s', '-X', 'POST',
          '-H', 'Content-Type: application/x-www-form-urlencoded',
          '-d', bodyStr,
          'https://accounts.spotify.com/api/token'];

        var raw = await api.shell.execute('curl', args);
        var data = JSON.parse(raw.trim());
        if (data.error) return false;

        token = data.access_token;
        if (data.refresh_token) refreshToken = data.refresh_token;
        await api.storage.set('spotify-token', token);
        if (data.refresh_token) await api.storage.set('spotify-refresh-token', refreshToken);
        return true;
      } catch (e) { return false; }
    }

    function showStep1() {
      clearTimers(); currentTrack = null; clearNowPlaying(); root.innerHTML = '';
      var wrap = document.createElement('div'); wrap.className = 'spot-setup';

      wrap.innerHTML =
        '<div class="spot-setup-icon">' + svgMusic + '</div>' +
        '<div class="spot-setup-title">Connect to Spotify</div>' +
        '<div class="spot-step-label">Step 1 of 2</div>' +
        '<div class="spot-instructions">' +
          '<div class="spot-instructions-heading">Setup (one-time only):</div>' +
          '<ol>' +
            '<li>Go to <a class="spot-dash-link">developer.spotify.com/dashboard</a></li>' +
            '<li>Log in and click <strong>\u201CCreate App\u201D</strong></li>' +
            '<li>Fill in the app details:' +
              '<ul>' +
                '<li>Name: <strong>Vanta</strong> (or anything you like)</li>' +
                '<li>Description: anything</li>' +
                '<li>Redirect URI \u2014 type this <strong>exactly</strong>:<br><code>' + REDIRECT_URI + '</code></li>' +
                '<li>Check <strong>\u201CWeb API\u201D</strong></li>' +
                '<li>Click <strong>Save</strong></li>' +
              '</ul>' +
            '</li>' +
            '<li>On the app page, click <strong>Settings</strong> and copy the <strong>Client ID</strong></li>' +
          '</ol>' +
        '</div>';

      var input = document.createElement('input');
      input.className = 'spot-input'; input.type = 'text';
      input.placeholder = 'Paste your Client ID here\u2026';
      wrap.appendChild(input);

      var btn = document.createElement('button');
      btn.className = 'spot-btn'; btn.textContent = 'Next \u2192';
      btn.onclick = async function() {
        var val = input.value.trim();
        if (!val) return;
        if (val.length < 10 || val.includes(' ')) {
          api.toast({ title: 'Invalid Client ID', message: 'The Client ID should be a long string of letters and numbers with no spaces.', type: 'error' });
          return;
        }
        btn.textContent = 'Saving\u2026'; btn.disabled = true;
        try {
          clientId = val;
          await api.storage.set('spotify-client-id', val);
          showStep2();
        } catch (e) {
          btn.textContent = 'Next \u2192'; btn.disabled = false;
          api.toast({ title: 'Error', message: 'Failed to save', type: 'error' });
        }
      };
      wrap.appendChild(btn);
      root.appendChild(wrap);

      var dashLink = wrap.querySelector('.spot-dash-link');
      if (dashLink) {
        dashLink.onclick = function(e) {
          e.preventDefault();
          api.shell.execute('xdg-open', ['https://developer.spotify.com/dashboard']).catch(function() {});
        };
      }
      input.focus();
    }

    async function showStep2() {
      clearTimers(); root.innerHTML = '';

      codeVerifier = generateCodeVerifier();
      await api.storage.set('spotify-code-verifier', codeVerifier);
      var challenge = await generateCodeChallenge(codeVerifier);

      var scopes = 'user-read-playback-state user-modify-playback-state user-read-currently-playing';
      var authUrl = 'https://accounts.spotify.com/authorize' +
        '?client_id=' + encodeURIComponent(clientId) +
        '&response_type=code' +
        '&redirect_uri=' + encodeURIComponent(REDIRECT_URI) +
        '&scope=' + encodeURIComponent(scopes) +
        '&code_challenge_method=S256' +
        '&code_challenge=' + challenge +
        '&show_dialog=true';

      var wrap = document.createElement('div'); wrap.className = 'spot-setup';

      wrap.innerHTML =
        '<div class="spot-setup-icon">' + svgMusic + '</div>' +
        '<div class="spot-setup-title">Authorize Spotify</div>' +
        '<div class="spot-step-label">Step 2 of 2</div>';

      var openBtn = document.createElement('button');
      openBtn.className = 'spot-btn'; openBtn.textContent = '1. Open Spotify Login';
      openBtn.onclick = function() {
        api.shell.execute('xdg-open', [authUrl]).catch(function() {
          api.toast({ title: 'Error', message: 'Could not open browser', type: 'error' });
        });
        setTimeout(function() { urlInput.focus(); }, 300);
      };
      wrap.appendChild(openBtn);

      var hint = document.createElement('div');
      hint.className = 'spot-hint';
      hint.innerHTML =
        '<strong>What to do:</strong><br>' +
        '1. Click the button above to open Spotify in your browser<br>' +
        '2. Log in and click <strong>\u201CAgree\u201D</strong><br>' +
        '3. Your browser will try to load a page that <strong>won\u2019t open</strong> \u2014 that\u2019s expected!<br>' +
        '4. Look at your browser\u2019s <strong>address bar</strong> \u2014 the URL should start with:<br>' +
        '<code>' + esc(REDIRECT_URI) + '?code=...</code><br>' +
        '5. <strong>Copy that entire URL</strong> and paste it below';
      wrap.appendChild(hint);

      var urlInput = document.createElement('input');
      urlInput.className = 'spot-input'; urlInput.type = 'text';
      urlInput.placeholder = '2. Paste the redirect URL here\u2026';
      wrap.appendChild(urlInput);

      var errEl = document.createElement('div');
      errEl.className = 'spot-err'; errEl.style.display = 'none';
      wrap.appendChild(errEl);

      var connectBtn = document.createElement('button');
      connectBtn.className = 'spot-btn'; connectBtn.textContent = '3. Connect';
      connectBtn.onclick = async function() {
        var url = urlInput.value.trim();
        if (!url) return;
        errEl.style.display = 'none';

        if (url.includes('accounts.spotify.com/authorize')) {
          errEl.innerHTML = '<strong>Wrong URL!</strong> You pasted the Spotify login page URL.<br><br>' +
            'You need to paste the URL <strong>after</strong> you click \u201CAgree\u201D on Spotify. ' +
            'That URL starts with <code>' + esc(REDIRECT_URI) + '</code> and the page won\u2019t load \u2014 that\u2019s normal. ' +
            'Just copy it from your address bar.';
          errEl.style.display = '';
          return;
        }

        if (url.includes('INVALID_CLIENT') || url.includes('invalid_client')) {
          errEl.innerHTML = '<strong>Redirect URI mismatch!</strong><br><br>' +
            'Go to your <a class="spot-dash-link2" style="color:var(--vanta-accent,#7b35f0);cursor:pointer">Spotify Developer Dashboard</a> \u2192 your app \u2192 <strong>Settings</strong> \u2192 <strong>Redirect URIs</strong><br><br>' +
            'Make sure it contains <strong>exactly</strong>: <code>' + esc(REDIRECT_URI) + '</code><br><br>' +
            'Remove any old URIs and add this one, then click Save.';
          errEl.style.display = '';
          var link2 = errEl.querySelector('.spot-dash-link2');
          if (link2) link2.onclick = function(e) {
            e.preventDefault();
            api.shell.execute('xdg-open', ['https://developer.spotify.com/dashboard']).catch(function(){});
          };
          return;
        }

        var code = null;
        try {
          var urlObj = new URL(url);
          code = urlObj.searchParams.get('code');
        } catch (e) {
          code = null;
          var codeMatch = url.match(/[?&]code=([^&]+)/);
          if (codeMatch) code = decodeURIComponent(codeMatch[1]);
        }

        if (!code) {
          errEl.innerHTML = 'No authorization code found in the URL.<br><br>' +
            'The URL you paste should look like:<br><code>' + esc(REDIRECT_URI) + '?code=AQDx7...</code><br><br>' +
            'Make sure you copied the URL from your browser\u2019s address bar <strong>after</strong> clicking Agree on Spotify.';
          errEl.style.display = '';
          return;
        }

        connectBtn.textContent = 'Connecting\u2026'; connectBtn.disabled = true;
        try {
          var tokenData = await exchangeCodeForToken(code);
          token = tokenData.access_token;
          refreshToken = tokenData.refresh_token || null;
          await api.storage.set('spotify-token', token);
          if (refreshToken) await api.storage.set('spotify-refresh-token', refreshToken);
          api.toast({ title: 'Connected to Spotify!', type: 'success' });
          showPlayer();
        } catch (e) {
          connectBtn.textContent = '3. Connect'; connectBtn.disabled = false;
          errEl.textContent = 'Failed: ' + (e.message || String(e));
          errEl.style.display = '';
        }
      };
      wrap.appendChild(connectBtn);

      var changeBtn = document.createElement('button');
      changeBtn.className = 'spot-btn spot-btn-outline spot-btn-sm';
      changeBtn.textContent = '\u2190 Change Client ID';
      changeBtn.onclick = function() { showStep1(); };
      wrap.appendChild(changeBtn);
      root.appendChild(wrap);
    }

    function showExpired() {
      clearTimers(); token = null; currentTrack = null; lyricsText = null; clearNowPlaying();

      if (refreshToken && clientId) {
        root.innerHTML = '<div class="spot-loading">Refreshing token\u2026</div>';
        refreshAccessToken().then(function(ok) {
          if (ok) { showPlayer(); }
          else { api.storage.set('spotify-token', ''); showStep2(); }
        });
        return;
      }

      api.storage.set('spotify-token', '');
      root.innerHTML = '';

      if (!clientId) { showStep1(); return; }

      var wrap = document.createElement('div'); wrap.className = 'spot-setup';
      wrap.innerHTML =
        '<div class="spot-setup-icon">' + svgMusic + '</div>' +
        '<div class="spot-setup-title">Session Expired</div>' +
        '<div class="spot-setup-desc">Your session has expired.<br>Click below to reconnect.</div>';

      var btn = document.createElement('button');
      btn.className = 'spot-btn'; btn.textContent = 'Reconnect';
      btn.onclick = function() { showStep2(); };
      wrap.appendChild(btn);
      root.appendChild(wrap);
    }

    function showPlayer() {
      clearTimers(); root.innerHTML = '';

      var header = document.createElement('div'); header.className = 'spot-header';
      var headerLeft = document.createElement('div'); headerLeft.className = 'spot-header-left';
      var dot = document.createElement('div'); dot.className = 'spot-header-dot';
      var headerTitle = document.createElement('span'); headerTitle.className = 'spot-header-title'; headerTitle.textContent = 'Spotify';
      headerLeft.appendChild(dot); headerLeft.appendChild(headerTitle);
      header.appendChild(headerLeft);

      var headerActions = document.createElement('div'); headerActions.className = 'spot-header-actions';
      var miniBtn = document.createElement('button'); miniBtn.className = 'spot-icon-btn accent';
      miniBtn.innerHTML = svgMiniPlayer; miniBtn.title = 'Open Mini Player';
      miniBtn.onclick = function() {
        if (window.__vanta_sdk && window.__vanta_sdk.window && window.__vanta_sdk.window.openMiniPlayer) {
          window.__vanta_sdk.window.openMiniPlayer();
        } else {
          api.toast({ title: 'Mini player not available', type: 'info' });
        }
      };
      headerActions.appendChild(miniBtn);
      header.appendChild(headerActions);
      root.appendChild(header);

      var tabs = document.createElement('div'); tabs.className = 'spot-tabs';
      var tabNow = document.createElement('button'); tabNow.className = 'spot-tab active'; tabNow.textContent = 'Now Playing';
      var tabSearch = document.createElement('button'); tabSearch.className = 'spot-tab'; tabSearch.textContent = 'Search';
      tabs.appendChild(tabNow); tabs.appendChild(tabSearch);
      root.appendChild(tabs);

      var content = document.createElement('div');
      root.appendChild(content);

      var footer = document.createElement('div'); footer.className = 'spot-footer';
      var reconnectBtn = document.createElement('button');
      reconnectBtn.className = 'spot-btn spot-btn-outline spot-btn-sm';
      reconnectBtn.textContent = 'Reconnect';
      reconnectBtn.onclick = function() { token = null; currentTrack = null; clearNowPlaying(); api.storage.set('spotify-token', ''); clearTimers(); showStep2(); };

      var disconnectBtn = document.createElement('button');
      disconnectBtn.className = 'spot-btn spot-btn-danger spot-btn-sm';
      disconnectBtn.textContent = 'Disconnect';
      disconnectBtn.onclick = async function() {
        token = null; clientId = null; refreshToken = null; currentTrack = null; clearNowPlaying();
        await api.storage.set('spotify-token', '');
        await api.storage.set('spotify-client-id', '');
        await api.storage.set('spotify-refresh-token', '');
        await api.storage.set('spotify-code-verifier', '');
        clearTimers(); showStep1();
      };

      footer.appendChild(reconnectBtn); footer.appendChild(disconnectBtn);
      root.appendChild(footer);

      tabNow.onclick = function() { tabNow.className = 'spot-tab active'; tabSearch.className = 'spot-tab'; renderNowPlaying(content); };
      tabSearch.onclick = function() { tabSearch.className = 'spot-tab active'; tabNow.className = 'spot-tab'; renderSearch(content); };

      renderNowPlaying(content);
      startRefresh();
    }

    function startRefresh() {
      fetchNowPlaying();
      refreshTimer = setInterval(function() { if (!root.isConnected) { clearTimers(); return; } fetchNowPlaying(); }, 5000);
      progressTimer = setInterval(function() {
        if (!root.isConnected) { clearTimers(); return; }
        if (isPlaying && durationMs > 0) { progressMs = Math.min(progressMs + 1000, durationMs); updateProgressUI(); emitNowPlaying(); }
      }, 1000);
    }

    function renderNowPlaying(container) {
      nowPlayingContainer = container; container.innerHTML = '';

      if (!currentTrack) {
        var empty = document.createElement('div'); empty.className = 'spot-empty';
        empty.innerHTML = '<div class="spot-empty-icon">' + svgMusic + '</div><div>Nothing is playing</div><div style="font-size:11px;margin-top:4px;color:var(--vanta-text-dim,#888)">Play something on Spotify to see it here</div>';
        container.appendChild(empty);
        return;
      }

      var artSection = document.createElement('div'); artSection.className = 'spot-art-section';

      if (albumArtUrl) {
        var img = document.createElement('img'); img.className = 'spot-art-img';
        img.src = albumArtUrl; img.alt = 'Album art';
        img.onerror = function() { img.style.display = 'none'; };
        artSection.appendChild(img);
      } else {
        var placeholder = document.createElement('div'); placeholder.className = 'spot-art-placeholder';
        placeholder.innerHTML = svgMusic;
        artSection.appendChild(placeholder);
      }

      var trackInfo = document.createElement('div'); trackInfo.className = 'spot-track-info';
      var trackName = document.createElement('div'); trackName.className = 'spot-track-name'; trackName.textContent = currentTrack.name;
      var trackArtist = document.createElement('div'); trackArtist.className = 'spot-track-artist'; trackArtist.textContent = currentTrack.artist;
      trackInfo.appendChild(trackName); trackInfo.appendChild(trackArtist);
      if (currentTrack.album) {
        var trackAlbum = document.createElement('div'); trackAlbum.className = 'spot-track-album'; trackAlbum.textContent = currentTrack.album;
        trackInfo.appendChild(trackAlbum);
      }
      artSection.appendChild(trackInfo);
      container.appendChild(artSection);

      var progSection = document.createElement('div'); progSection.className = 'spot-progress-section';
      var pct = durationMs > 0 ? (progressMs / durationMs * 100) : 0;
      progSection.innerHTML =
        '<div class="spot-progress-bar"><div class="spot-progress-fill" style="width:' + pct + '%"><div class="spot-progress-thumb"></div></div></div>' +
        '<div class="spot-progress-times"><span>' + fmtTime(progressMs) + '</span><span>' + fmtTime(durationMs) + '</span></div>';
      progressFillEl = progSection.querySelector('.spot-progress-fill');
      progressCurEl = progSection.querySelector('.spot-progress-times span');

      var progBar = progSection.querySelector('.spot-progress-bar');
      progBar.onclick = function(e) {
        if (durationMs <= 0) return;
        var rect = progBar.getBoundingClientRect();
        var ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
        var seekMs = Math.floor(ratio * durationMs);
        progressMs = seekMs; updateProgressUI();
        spotApi('PUT', '/me/player/seek?position_ms=' + seekMs);
      };
      container.appendChild(progSection);

      var controls = document.createElement('div'); controls.className = 'spot-controls';
      shuffleBtnRef = makeCtrl(svgShuffle, shuffleState, function() { spotApi('PUT', '/me/player/shuffle?state=' + (!shuffleState)); shuffleState = !shuffleState; shuffleBtnRef.className = 'spot-ctrl' + (shuffleState ? ' active' : ''); });
      var btnPrev = makeCtrl(svgPrev, false, function() { spotApi('POST', '/me/player/previous'); setTimeout(fetchNowPlaying, 300); });

      playPauseBtnRef = document.createElement('button'); playPauseBtnRef.className = 'spot-ctrl-main';
      playPauseBtnRef.innerHTML = isPlaying ? svgPause : svgPlay;
      playPauseBtnRef.onclick = function() {
        if (isPlaying) { spotApi('PUT', '/me/player/pause'); isPlaying = false; }
        else { spotApi('PUT', '/me/player/play'); isPlaying = true; }
        playPauseBtnRef.innerHTML = isPlaying ? svgPause : svgPlay;
        emitNowPlaying();
      };

      var btnNext = makeCtrl(svgNext, false, function() { spotApi('POST', '/me/player/next'); setTimeout(fetchNowPlaying, 300); });
      repeatBtnRef = makeCtrl(svgRepeat, repeatState !== 'off', function() {
        var next = repeatState === 'off' ? 'context' : repeatState === 'context' ? 'track' : 'off';
        spotApi('PUT', '/me/player/repeat?state=' + next); repeatState = next;
        repeatBtnRef.className = 'spot-ctrl' + (repeatState !== 'off' ? ' active' : '');
      });

      controls.appendChild(shuffleBtnRef); controls.appendChild(btnPrev); controls.appendChild(playPauseBtnRef);
      controls.appendChild(btnNext); controls.appendChild(repeatBtnRef);
      container.appendChild(controls);

      var volWrap = document.createElement('div'); volWrap.className = 'spot-volume';
      var volIcon = document.createElement('div'); volIcon.className = 'spot-vol-icon'; volIcon.innerHTML = svgVolume;
      var volBar = document.createElement('div'); volBar.className = 'spot-vol-bar';
      var volFill = document.createElement('div'); volFill.className = 'spot-vol-fill'; volFill.style.width = volumePercent + '%';
      volBar.appendChild(volFill);
      volBar.onclick = function(e) {
        var rect = volBar.getBoundingClientRect();
        var ratio = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
        volumePercent = Math.round(ratio * 100);
        volFill.style.width = volumePercent + '%';
        spotApi('PUT', '/me/player/volume?volume_percent=' + volumePercent);
      };
      volWrap.appendChild(volIcon); volWrap.appendChild(volBar);
      container.appendChild(volWrap);

      var lyricsWrap = document.createElement('div'); lyricsWrap.className = 'spot-lyrics';
      var lyricsTitle = document.createElement('div'); lyricsTitle.className = 'spot-lyrics-title'; lyricsTitle.textContent = 'Lyrics';
      lyricsWrap.appendChild(lyricsTitle);
      if (lyricsText && lyricsText.trim()) {
        lyricsText.split('\n').map(function(line) { return line.trim(); }).filter(function(line) { return line.length > 0; }).slice(0, 30).forEach(function(line) {
          var row = document.createElement('div'); row.className = 'spot-lyrics-line'; row.textContent = line;
          lyricsWrap.appendChild(row);
        });
      } else {
        var empty = document.createElement('div'); empty.className = 'spot-lyrics-empty';
        empty.textContent = 'Lyrics unavailable for this track.';
        lyricsWrap.appendChild(empty);
      }
      container.appendChild(lyricsWrap);
    }

    function makeCtrl(svg, active, handler) {
      var btn = document.createElement('button'); btn.className = 'spot-ctrl' + (active ? ' active' : '');
      btn.innerHTML = svg; btn.onclick = handler; return btn;
    }

    function updateProgressUI() {
      if (progressFillEl && durationMs > 0) progressFillEl.style.width = (progressMs / durationMs * 100) + '%';
      if (progressCurEl) progressCurEl.textContent = fmtTime(progressMs);
    }

    async function fetchNowPlaying() {
      try {
        var raw = await spotApi('GET', '/me/player');
        var resp = parseResponse(raw);
        if (resp.code === 401) { showExpired(); return; }
        if (resp.code === 204 || !resp.body) {
          if (currentTrack !== null) { currentTrack = null; albumArtUrl = null; lyricsText = null; isPlaying = false; emitNowPlaying(); if (nowPlayingContainer) renderNowPlaying(nowPlayingContainer); }
          return;
        }
        var data = JSON.parse(resp.body);
        isPlaying = !!data.is_playing; shuffleState = !!data.shuffle_state; repeatState = data.repeat_state || 'off'; progressMs = data.progress_ms || 0;
        if (data.device) volumePercent = data.device.volume_percent || 100;
        if (data.item) {
          durationMs = data.item.duration_ms || 0;
          var artists = (data.item.artists || []).map(function(a) { return a.name; }).join(', ');
          var primaryArtist = data.item.artists && data.item.artists[0] ? data.item.artists[0].name : artists;
          var newId = data.item.id || data.item.uri;
          var artImages = data.item.album && data.item.album.images ? data.item.album.images : [];
          albumArtUrl = artImages.length > 0 ? artImages[0].url : null;

          if (!currentTrack || currentTrack.id !== newId) {
            currentTrack = { id: newId, name: data.item.name, artist: artists, album: data.item.album ? data.item.album.name : '', uri: data.item.uri };
            lyricsText = null;
            fetchLyrics(data.item.name, primaryArtist);
            if (nowPlayingContainer) renderNowPlaying(nowPlayingContainer);
          } else {
            updateProgressUI();
            if (playPauseBtnRef) playPauseBtnRef.innerHTML = isPlaying ? svgPause : svgPlay;
            if (shuffleBtnRef) shuffleBtnRef.className = 'spot-ctrl' + (shuffleState ? ' active' : '');
            if (repeatBtnRef) repeatBtnRef.className = 'spot-ctrl' + (repeatState !== 'off' ? ' active' : '');
          }
          emitNowPlaying();
        }
      } catch (e) { /* retry next interval */ }
    }

    function renderSearch(container) {
      nowPlayingContainer = null; container.innerHTML = '';
      var wrap = document.createElement('div'); wrap.className = 'spot-search-wrap';
      var input = document.createElement('input'); input.className = 'spot-search-input'; input.type = 'text';
      input.placeholder = 'Search for a song, artist, or album\u2026';
      wrap.appendChild(input);
      container.appendChild(wrap);
      var results = document.createElement('div'); container.appendChild(results);

      input.oninput = function() {
        if (searchTimeout) clearTimeout(searchTimeout);
        var q = input.value.trim();
        if (!q) { results.innerHTML = ''; return; }
        searchTimeout = setTimeout(function() { doSearch(q, results); }, 400);
      };
      input.focus();
    }

    async function doSearch(query, container) {
      container.innerHTML = '<div class="spot-loading">Searching\u2026</div>';
      try {
        var raw = await spotApi('GET', '/search?q=' + encodeURIComponent(query) + '&type=track&limit=12');
        var resp = parseResponse(raw);
        if (resp.code === 401) { showExpired(); return; }
        if (resp.code !== 200) { container.innerHTML = '<div class="spot-err">Search failed (HTTP ' + resp.code + ')</div>'; return; }
        var data = JSON.parse(resp.body);
        var tracks = (data.tracks && data.tracks.items) || [];
        container.innerHTML = '';
        if (tracks.length === 0) { container.innerHTML = '<div class="spot-empty" style="padding:16px 0">No results found</div>'; return; }

        tracks.forEach(function(track) {
          var artists = (track.artists || []).map(function(a) { return a.name; }).join(', ');
          var artImages = track.album && track.album.images ? track.album.images : [];
          var thumbUrl = artImages.length > 1 ? artImages[1].url : (artImages.length > 0 ? artImages[0].url : null);

          var row = document.createElement('div'); row.className = 'spot-result';

          if (thumbUrl) {
            var thumb = document.createElement('img'); thumb.className = 'spot-result-art';
            thumb.src = thumbUrl; thumb.alt = '';
            thumb.onerror = function() { thumb.style.display = 'none'; };
            row.appendChild(thumb);
          } else {
            var pholder = document.createElement('div'); pholder.className = 'spot-result-art-placeholder';
            pholder.innerHTML = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>';
            row.appendChild(pholder);
          }

          var info = document.createElement('div'); info.className = 'spot-result-info';
          var nameEl = document.createElement('div'); nameEl.className = 'spot-result-name'; nameEl.textContent = track.name;
          var artEl = document.createElement('div'); artEl.className = 'spot-result-artist'; artEl.textContent = artists;
          info.appendChild(nameEl); info.appendChild(artEl); row.appendChild(info);
          var dur = document.createElement('div'); dur.className = 'spot-result-dur'; dur.textContent = fmtTime(track.duration_ms);
          row.appendChild(dur);
          var playBtn = document.createElement('button'); playBtn.className = 'spot-result-play'; playBtn.innerHTML = svgSmallPlay;
          playBtn.onclick = function(e) { e.stopPropagation(); playTrack(track.uri); };
          row.appendChild(playBtn);
          row.onclick = function() { playTrack(track.uri); };
          container.appendChild(row);
        });
      } catch (e) { container.innerHTML = '<div class="spot-err">Search failed: ' + esc(String(e)) + '</div>'; }
    }

    async function playTrack(uri) {
      try {
        var raw = await spotApiBody('PUT', '/me/player/play', { uris: [uri] });
        var resp = parseResponse(raw);
        if (resp.code === 401) { showExpired(); return; }
        isPlaying = true; setTimeout(fetchNowPlaying, 500);
        api.toast({ title: 'Playing', type: 'success' });
      } catch (e) { api.toast({ title: 'Playback failed', message: String(e), type: 'error' }); }
    }

    window.addEventListener('vanta-spotify-command', function(e) {
      if (!token) return;
      var payload = e.detail;
      var cmd = typeof payload === 'object' && payload ? payload.cmd : payload;
      if (cmd === 'play-pause') {
        if (isPlaying) { spotApi('PUT', '/me/player/pause'); isPlaying = false; }
        else { spotApi('PUT', '/me/player/play'); isPlaying = true; }
        if (playPauseBtnRef) playPauseBtnRef.innerHTML = isPlaying ? svgPause : svgPlay;
        emitNowPlaying();
      } else if (cmd === 'next') {
        spotApi('POST', '/me/player/next'); setTimeout(fetchNowPlaying, 300);
      } else if (cmd === 'prev') {
        spotApi('POST', '/me/player/previous'); setTimeout(fetchNowPlaying, 300);
      } else if (cmd === 'set-volume') {
        var val = typeof payload === 'object' && payload ? Number(payload.value) : NaN;
        if (!isNaN(val)) {
          volumePercent = Math.max(0, Math.min(100, Math.round(val)));
          spotApi('PUT', '/me/player/volume?volume_percent=' + volumePercent);
          if (nowPlayingContainer) renderNowPlaying(nowPlayingContainer);
          emitNowPlaying();
        }
      }
    });

    async function init() {
      try {
        clientId = (await api.storage.get('spotify-client-id')) || null;
        token = (await api.storage.get('spotify-token')) || null;
        refreshToken = (await api.storage.get('spotify-refresh-token')) || null;
        codeVerifier = (await api.storage.get('spotify-code-verifier')) || null;
      } catch (e) { clientId = null; token = null; refreshToken = null; }

      if (token) { showPlayer(); }
      else if (clientId) { showStep2(); }
      else { showStep1(); }
    }

    init();
  }

  vanta.registerExtension('spotify', {
    commands: { 'player': { component: SpotifyPlayer } }
  });
})(window.__vanta_host);
