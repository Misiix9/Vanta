use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Deserialize)]
pub struct WindowEntry {
    pub title: String,
    pub class: String,
    pub address: String, // "address" in Hyprland, "id" in Sway (as string for uniformity)
    pub workspace: String,
    pub backend: String,
    pub last_active: u64,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct HyprlandClient {
    address: String,
    class: String,
    title: String,
    workspace: HyprlandWorkspace,
    #[serde(default)]
    monitor: Option<u32>,
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
    #[serde(default)]
    rect: Option<SwayRect>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SwayWindowProperties {
    class: Option<String>,
    title: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SwayRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// Detects the running environment and lists open windows.
fn list_windows_uncached(recency: &RecencyStore) -> Vec<WindowEntry> {
    // 1. Try Hyprland (explicitly resolving signature and binary path)
    if let Some(output) = run_hyprctl_json::<HyprlandClient>(&["clients", "-j"]) {
        recency.bump_active_hyprland();
        let mut seen = HashSet::new();
        let windows: Vec<WindowEntry> = output
            .into_iter()
            .filter(|c| !c.title.is_empty()) // Filter out empty titles (e.g. overlays)
            .map(|c| {
                let addr = c.address;
                seen.insert(addr.clone());
                WindowEntry {
                    title: c.title,
                    class: c.class,
                    address: addr.clone(),
                    workspace: c.workspace.name,
                    backend: "hyprland".to_string(),
                    last_active: recency.stamp(&addr),
                }
            })
            .collect();
        recency.prune(&seen);
        eprintln!("[vanta][hyprctl] clients parsed windows={}", windows.len());
        if !windows.is_empty() {
            return windows;
        }
    }

    // 2. Try Sway
    if let Ok(output) = Command::new("swaymsg").arg("-t").arg("get_tree").output() {
        if output.status.success() {
            if let Ok(root) = serde_json::from_slice::<SwayNode>(&output.stdout) {
                recency.bump_active_sway(&root);
                let mut windows = Vec::new();
                collect_sway_windows(&root, None, &mut windows, recency);
                let seen: HashSet<String> = windows.iter().map(|w| w.address.clone()).collect();
                recency.prune(&seen);
                return windows;
            }
        }
    }

    // 3. Fallback: X11 via wmctrl
    if let Ok(output) = Command::new("wmctrl").arg("-lp").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut windows = Vec::new();

            for line in stdout.lines() {
                // Format: 0x01200003  0 pid host title...
                let mut parts = line.split_whitespace();
                let id_hex = parts.next();
                let workspace = parts.next();
                let pid_str = parts.next();
                // skip host
                let _host = parts.next();
                let title = parts.collect::<Vec<&str>>().join(" ");

                if let (Some(id_hex), Some(ws), Some(pid_str)) = (id_hex, workspace, pid_str) {
                    let pid = pid_str.parse::<i32>().unwrap_or_default();
                    let class = proc_name_from_pid(pid).unwrap_or_else(|| "Window".to_string());

                    // wmctrl gives hex window id like 0x04600007; use as address
                    let address = id_hex.to_string();
                    windows.push(WindowEntry {
                        title: title.clone(),
                        class,
                        address: address.clone(),
                        workspace: ws.to_string(),
                        backend: "x11".to_string(),
                        last_active: recency.stamp(&address),
                    });
                }
            }

            let seen: HashSet<String> = windows.iter().map(|w| w.address.clone()).collect();
            recency.prune(&seen);
            if !windows.is_empty() {
                return windows;
            }
        }
    }

    // Default: Return empty if neither is found
    Vec::new()
}

fn proc_name_from_pid(pid: i32) -> Option<String> {
    if pid <= 0 {
        return None;
    }
    let comm_path = format!("/proc/{}/comm", pid);
    if let Ok(name) = std::fs::read_to_string(&comm_path) {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    if let Ok(cmdline) = std::fs::read(cmdline_path) {
        let parts: Vec<&str> = cmdline.split(|b| *b == 0).filter_map(|s| std::str::from_utf8(s).ok()).collect();
        if let Some(first) = parts.first() {
            if let Some(bin) = first.split('/').last() {
                if !bin.is_empty() {
                    return Some(bin.to_string());
                }
            }
        }
    }
    None
}

struct WindowsCache {
    updated_at: Instant,
    entries: Vec<WindowEntry>,
}

static WINDOWS_CACHE: OnceLock<Mutex<WindowsCache>> = OnceLock::new();
static WINDOW_RECENCY: OnceLock<Mutex<HashMap<String, u64>>> = OnceLock::new();

#[cfg(test)]
static MOCK_WINDOW_GROUPS: OnceLock<Mutex<Option<Vec<WindowGroup>>>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct WindowGroup {
    pub app_class: String,
    pub entries: Vec<WindowEntry>,
    pub last_active: u64,
}

#[derive(Clone)]
struct RecencyStore {
    inner: &'static Mutex<HashMap<String, u64>>,
}

impl RecencyStore {
    fn new() -> Self {
        let mutex = WINDOW_RECENCY.get_or_init(|| Mutex::new(HashMap::new()));
        Self { inner: mutex }
    }

    fn now() -> u64 {
        static START: OnceLock<Instant> = OnceLock::new();
        let start = *START.get_or_init(Instant::now);
        start.elapsed().as_millis() as u64
    }

    fn stamp(&self, addr: &str) -> u64 {
        if let Ok(mut map) = self.inner.lock() {
            let now = Self::now();
            let entry = map
                .entry(addr.to_string())
                .or_insert_with(|| now.saturating_sub(1));
            *entry
        } else {
            Self::now()
        }
    }

    fn set_active(&self, addr: &str) {
        if let Ok(mut map) = self.inner.lock() {
            map.insert(addr.to_string(), Self::now());
        }
    }

    fn prune(&self, keep: &HashSet<String>) {
        if let Ok(mut map) = self.inner.lock() {
            map.retain(|k, _| keep.contains(k));
        }
    }

    fn bump_active_hyprland(&self) {
        if let Some(mut wins) = run_hyprctl_json::<HyprlandClient>(&["activewindow", "-j"]) {
            if let Some(active) = wins.pop() {
                self.set_active(&active.address);
            }
        }
    }

    fn bump_active_sway(&self, node: &SwayNode) {
        if node.focused {
            self.set_active(&node.id.to_string());
        }
        for child in &node.nodes {
            self.bump_active_sway(child);
        }
        for child in &node.floating_nodes {
            self.bump_active_sway(child);
        }
    }
}

/// Cached window list for faster search. Cache TTL is short to keep UX fresh.
pub fn list_windows() -> Vec<WindowEntry> {
    const TTL: Duration = Duration::from_millis(400);

    let cache = WINDOWS_CACHE.get_or_init(|| {
        Mutex::new(WindowsCache {
            updated_at: Instant::now() - TTL,
            entries: Vec::new(),
        })
    });

    let recency = RecencyStore::new();

    if let Ok(mut guard) = cache.lock() {
        if guard.updated_at.elapsed() >= TTL {
            guard.entries = list_windows_uncached(&recency);
            guard.updated_at = Instant::now();
        }
        return guard.entries.clone();
    }

    list_windows_uncached(&recency)
}

fn group_windows(entries: Vec<WindowEntry>, max_items: usize) -> Vec<WindowGroup> {
    let mut grouped: HashMap<String, Vec<WindowEntry>> = HashMap::new();
    for w in entries {
        grouped.entry(w.class.clone()).or_default().push(w);
    }

    let mut groups: Vec<WindowGroup> = grouped
        .into_iter()
        .map(|(class, mut items)| {
            items.sort_by(|a, b| b.last_active.cmp(&a.last_active));
            let last_active = items.first().map(|w| w.last_active).unwrap_or(0);
            WindowGroup {
                app_class: class,
                entries: items,
                last_active,
            }
        })
        .collect();

    groups.sort_by(|a, b| b.last_active.cmp(&a.last_active));

    let mut capped = Vec::new();
    let mut count = 0;
    for mut g in groups {
        if count >= max_items {
            break;
        }
        if count + g.entries.len() > max_items {
            g.entries.truncate(max_items - count);
        }
        count += g.entries.len();
        capped.push(g);
    }

    capped
}

pub fn list_windows_grouped(max_items: usize) -> Vec<WindowGroup> {
    #[cfg(test)]
    {
        if let Some(mock) = MOCK_WINDOW_GROUPS.get() {
            if let Ok(guard) = mock.lock() {
                if let Some(data) = guard.as_ref() {
                    return data.clone();
                }
            }
        }
    }

    let recency = RecencyStore::new();
    let entries = list_windows_uncached(&recency);
    group_windows(entries, max_items)
}

#[cfg(test)]
pub fn set_mock_window_groups(groups: Option<Vec<WindowGroup>>) {
    let lock = MOCK_WINDOW_GROUPS.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = lock.lock() {
        *guard = groups;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_groups_by_class_and_sorts_by_recency() {
        let now = 1_000u64;
        let entries = vec![
            WindowEntry {
                title: "A1".into(),
                class: "AppA".into(),
                address: "1".into(),
                workspace: "1".into(),
                backend: "hyprland".into(),
                last_active: now - 10,
            },
            WindowEntry {
                title: "A2".into(),
                class: "AppA".into(),
                address: "2".into(),
                workspace: "1".into(),
                backend: "hyprland".into(),
                last_active: now - 5,
            },
            WindowEntry {
                title: "B1".into(),
                class: "AppB".into(),
                address: "3".into(),
                workspace: "2".into(),
                backend: "sway".into(),
                last_active: now - 2,
            },
        ];

        let grouped = group_windows(entries, 10);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].app_class, "AppB");
        assert_eq!(grouped[0].entries[0].title, "B1");
        assert_eq!(grouped[1].entries[0].title, "A2");
    }

    #[test]
    fn windows_caps_total_items_across_groups() {
        let now = 1_000u64;
        let entries = vec![
            WindowEntry {
                title: "A1".into(),
                class: "AppA".into(),
                address: "1".into(),
                workspace: "1".into(),
                backend: "hyprland".into(),
                last_active: now - 1,
            },
            WindowEntry {
                title: "A2".into(),
                class: "AppA".into(),
                address: "2".into(),
                workspace: "1".into(),
                backend: "hyprland".into(),
                last_active: now - 2,
            },
            WindowEntry {
                title: "B1".into(),
                class: "AppB".into(),
                address: "3".into(),
                workspace: "2".into(),
                backend: "sway".into(),
                last_active: now - 3,
            },
        ];

        let grouped = group_windows(entries, 2);
        let total: usize = grouped.iter().map(|g| g.entries.len()).sum();
        assert_eq!(total, 2);
        assert_eq!(grouped[0].app_class, "AppA");
        assert_eq!(grouped[0].entries.len(), 2);
    }

    #[test]
    fn windows_prunes_stale_recency() {
        let store = RecencyStore::new();
        // Seed two entries
        store.stamp("keep");
        store.stamp("drop");

        let mut keep = HashSet::new();
        keep.insert("keep".to_string());
        store.prune(&keep);

        if let Ok(map) = store.inner.lock() {
            assert!(map.contains_key("keep"));
            assert!(!map.contains_key("drop"));
        }
    }

    #[test]
    fn windows_threads_workspace_name_in_sway() {
        let recency = RecencyStore::new();
        let mut windows = Vec::new();

        let sway = SwayNode {
            id: 1,
            name: Some("1: dev".to_string()),
            app_id: None,
            window_properties: None,
            nodes: vec![SwayNode {
                id: 2,
                name: Some("child".to_string()),
                app_id: Some("AppA".to_string()),
                window_properties: None,
                nodes: vec![],
                floating_nodes: vec![],
                pid: Some(123),
                focused: true,
                rect: None,
            }],
            floating_nodes: vec![],
            pid: None,
            focused: false,
            rect: None,
        };

        collect_sway_windows(&sway, None, &mut windows, &recency);
        assert_eq!(windows.len(), 1);
        assert_eq!(windows[0].workspace, "1: dev");
    }
}

fn collect_sway_windows(
    node: &SwayNode,
    workspace: Option<&str>,
    windows: &mut Vec<WindowEntry>,
    recency: &RecencyStore,
) {
    let mut current_ws = workspace.map(|s| s.to_string());
    if let Some(name) = node.name.as_ref() {
        let is_workspace_like = name
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            || name.contains(':');

        if is_workspace_like {
            current_ws = Some(name.clone());
        }
    }

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
            let ws = current_ws
                .as_deref()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string());
            let address = node.id.to_string();
            windows.push(WindowEntry {
                title: name,
                class: app_id,
                address: address.clone(), // Sway uses distinct integer IDs
                workspace: ws,
                backend: "sway".to_string(),
                last_active: recency.stamp(&address),
            });
        }
    }

    for child in &node.nodes {
        collect_sway_windows(child, current_ws.as_deref(), windows, recency);
    }
    for child in &node.floating_nodes {
        collect_sway_windows(child, current_ws.as_deref(), windows, recency);
    }
}

// ----- Hyprland helpers -----

fn hypr_signature() -> Option<String> {
    if let Ok(sig) = env::var("HYPRLAND_INSTANCE_SIGNATURE") {
        if !sig.is_empty() {
            return Some(sig);
        }
    }
    if let Ok(entries) = fs::read_dir("/tmp/hypr") {
        for entry in entries.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
    }
    None
}

pub fn hyprctl_path() -> String {
    if let Ok(p) = env::var("HYPRCTL_PATH") {
        if !p.is_empty() {
            return p;
        }
    }
    if Path::new("/usr/bin/hyprctl").exists() {
        return "/usr/bin/hyprctl".to_string();
    }
    if Path::new("/usr/local/bin/hyprctl").exists() {
        return "/usr/local/bin/hyprctl".to_string();
    }
    "hyprctl".to_string()
}

fn run_hyprctl_json<T: serde::de::DeserializeOwned>(args: &[&str]) -> Option<Vec<T>> {
    let sig_opt = hypr_signature();
    let sig_log = sig_opt.clone();
    let bin = hyprctl_path();
    let mut cmd = Command::new(&bin);
    cmd.args(args);
    if let Some(ref sig) = sig_opt {
        cmd.env("HYPRLAND_INSTANCE_SIGNATURE", sig);
    }

    eprintln!("[vanta][hyprctl] invoking {:?} sig={:?}", cmd, sig_log);

    match cmd.output() {
        Ok(output) if output.status.success() => {
            // Hyprland may return either an array (clients) or a single object (activewindow).
            if let Ok(vec) = serde_json::from_slice::<Vec<T>>(&output.stdout) {
                eprintln!(
                    "[vanta][hyprctl] {:?} ok array len={} stderr={}",
                    args,
                    vec.len(),
                    String::from_utf8_lossy(&output.stderr)
                );
                return Some(vec);
            }
            if let Ok(single) = serde_json::from_slice::<T>(&output.stdout) {
                eprintln!(
                    "[vanta][hyprctl] {:?} ok single stderr={}",
                    args,
                    String::from_utf8_lossy(&output.stderr)
                );
                return Some(vec![single]);
            }
            eprintln!(
                "[vanta][hyprctl] parse failed for {:?}; raw={} stderr={}",
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            None
        }
        Ok(output) => {
            eprintln!(
                "[vanta][hyprctl] exited {:?} for {:?}; stdout={} stderr={}",
                output.status.code(),
                args,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            None
        }
        Err(e) => {
            eprintln!("[vanta][hyprctl] failed to run {}: {}", bin, e);
            None
        }
    }
}
