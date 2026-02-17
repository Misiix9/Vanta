/// Window management module.
/// This module is the single point of control for show/hide/blur/focus.
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewWindow};

#[derive(Clone, Debug, Serialize)]
pub struct BlurStatus {
    pub mode: String,
}

/// Initialize the window on startup: attempt native blur, emit fallback status.
pub fn init_window(_window: &WebviewWindow, app_handle: &AppHandle) -> Result<(), String> {
    log::info!("Initializing Vanta window");

    // On Linux/Wayland, native vibrancy is not universally supported.
    // We attempt to detect compositor support, but for v1.0 we rely on
    // CSS backdrop-filter as the primary glass effect.
    //
    // The `window-vibrancy` crate has limited Wayland support (it primarily
    // works on macOS/Windows). For Linux, we use CSS fallback by default
    // and can add native blur in future versions when compositor support improves.
    let blur_mode = "fallback".to_string();

    log::info!("Blur mode: {} (CSS backdrop-filter)", blur_mode);

    let status = BlurStatus { mode: blur_mode };
    app_handle
        .emit("blur-status", &status)
        .map_err(|e| format!("Failed to emit blur status: {}", e))?;

    Ok(())
}

/// Show the Vanta window and focus it.
pub fn show_window(window: &WebviewWindow) -> Result<(), String> {
    let start = Instant::now();

    window
        .show()
        .map_err(|e| format!("Failed to show window: {}", e))?;

    window
        .set_focus()
        .map_err(|e| format!("Failed to focus window: {}", e))?;

    let elapsed = start.elapsed();
    log::info!("Window show took: {:?}", elapsed);

    if elapsed > std::time::Duration::from_millis(100) {
        log::warn!("⚠️ Window show exceeded 100ms target! ({:?})", elapsed);
    }

    Ok(())
}

/// Hide the Vanta window.
pub fn hide_window(window: &WebviewWindow) -> Result<(), String> {
    window
        .hide()
        .map_err(|e| format!("Failed to hide window: {}", e))
}

/// Toggle window visibility (for hotkey/CLI).
pub fn toggle_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or("Failed to get main window")?;

    let visible = window
        .is_visible()
        .map_err(|e| format!("Failed to check visibility: {}", e))?;

    if visible {
        hide_window(&window)
    } else {
        // Critical for Wayland/Hyprland: Force always on top when showing
        // to ensure it floats above tiled windows.
        window
            .set_always_on_top(true)
            .map_err(|e| format!("Failed to set always on top: {}", e))?;
        show_window(&window)
    }
}
