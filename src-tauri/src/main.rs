// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

fn main() {
    // Fix for webkit2gtk on Wayland (especially Hyprland) —
    // DMABUF renderer causes GDK Protocol Error 71.
    // This must be set before any GTK/GDK initialization.
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    let cli = vanta_lib::Cli::parse();

    if let Some(command) = cli.command {
        if let Err(err) = vanta_lib::run_cli(command) {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
        return;
    }

    vanta_lib::run()
}
