// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Fix for webkit2gtk on Wayland (especially Hyprland) —
    // DMABUF renderer causes GDK Protocol Error 71.
    // This must be set before any GTK/GDK initialization.
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    vanta_lib::run()
}
