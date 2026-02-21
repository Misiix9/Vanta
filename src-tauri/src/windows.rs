use serde::Deserialize;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Deserialize)]
pub struct WindowEntry {
    pub title: String,
    pub class: String,
    pub address: String, // "address" in Hyprland, "id" in Sway (as string for uniformity)
    pub workspace: String,
}

#[derive(Deserialize)]
struct HyprlandClient {
    address: String,
    class: String,
    title: String,
    workspace: HyprlandWorkspace,
}

#[derive(Deserialize)]
struct HyprlandWorkspace {
    name: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SwayNode {
    id: i64,
    name: Option<String>,
    app_id: Option<String>,                          // Wayland
    window_properties: Option<SwayWindowProperties>, // XWayland
    nodes: Vec<SwayNode>,
    floating_nodes: Vec<SwayNode>,
    pid: Option<i32>,
    focused: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SwayWindowProperties {
    class: Option<String>,
    title: Option<String>,
}

/// Detects the running environment and lists open windows.
fn list_windows_uncached() -> Vec<WindowEntry> {
    // 1. Try Hyprland
    if let Ok(output) = Command::new("hyprctl").arg("clients").arg("-j").output() {
        if output.status.success() {
            if let Ok(clients) = serde_json::from_slice::<Vec<HyprlandClient>>(&output.stdout) {
                return clients
                    .into_iter()
                    .filter(|c| !c.title.is_empty()) // Filter out empty titles (e.g. overlays)
                    .map(|c| WindowEntry {
                        title: c.title,
                        class: c.class,
                        address: c.address,
                        workspace: c.workspace.name,
                    })
                    .collect();
            }
        }
    }

    // 2. Try Sway
    if let Ok(output) = Command::new("swaymsg").arg("-t").arg("get_tree").output() {
        if output.status.success() {
            if let Ok(root) = serde_json::from_slice::<SwayNode>(&output.stdout) {
                let mut windows = Vec::new();
                collect_sway_windows(&root, &mut windows);
                return windows;
            }
        }
    }

    // Default: Return empty if neither is found
    Vec::new()
}

struct WindowsCache {
    updated_at: Instant,
    entries: Vec<WindowEntry>,
}

static WINDOWS_CACHE: OnceLock<Mutex<WindowsCache>> = OnceLock::new();

/// Cached window list for faster search. Cache TTL is short to keep UX fresh.
pub fn list_windows() -> Vec<WindowEntry> {
    const TTL: Duration = Duration::from_millis(400);

    let cache = WINDOWS_CACHE.get_or_init(|| {
        Mutex::new(WindowsCache {
            updated_at: Instant::now() - TTL,
            entries: Vec::new(),
        })
    });

    if let Ok(mut guard) = cache.lock() {
        if guard.updated_at.elapsed() >= TTL {
            guard.entries = list_windows_uncached();
            guard.updated_at = Instant::now();
        }
        return guard.entries.clone();
    }

    list_windows_uncached()
}

fn collect_sway_windows(node: &SwayNode, windows: &mut Vec<WindowEntry>) {
    // If it has a PID, it's likely a window
    if node.pid.is_some() {
        let name = node.name.clone().unwrap_or_default();
        let app_id = node
            .app_id
            .clone()
            .or_else(|| {
                node.window_properties
                    .as_ref()
                    .and_then(|p| p.class.clone())
            })
            .unwrap_or_default();

        if !name.is_empty() {
            windows.push(WindowEntry {
                title: name,
                class: app_id,
                address: node.id.to_string(), // Sway uses distinct integer IDs
                workspace: "Unknown".to_string(), // Retrieving workspace in Sway requires tracking parent nodes, simplistic for now
            });
        }
    }

    for child in &node.nodes {
        collect_sway_windows(child, windows);
    }
    for child in &node.floating_nodes {
        collect_sway_windows(child, windows);
    }
}
