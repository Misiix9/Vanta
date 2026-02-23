use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Deserialize)]
pub struct WindowEntry {
    pub title: String,
    pub class: String,
    pub address: String, // "address" in Hyprland, "id" in Sway (as string for uniformity)
    pub workspace: String,
    pub last_active: u64,
}

#[derive(Deserialize)]
struct HyprlandClient {
    address: String,
    class: String,
    title: String,
    workspace: HyprlandWorkspace,
    #[serde(default)]
    monitor: Option<String>,
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
    // 1. Try Hyprland
    if let Ok(output) = Command::new("hyprctl").arg("clients").arg("-j").output() {
        if output.status.success() {
            if let Ok(clients) = serde_json::from_slice::<Vec<HyprlandClient>>(&output.stdout) {
                recency.bump_active_hyprland();
                let mut seen = HashSet::new();
                let windows = clients
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
                            last_active: recency.stamp(&addr),
                        }
                    })
                    .collect();
                recency.prune(&seen);
                return windows;
            }
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

    // Default: Return empty if neither is found
    Vec::new()
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
            map.insert(addr.to_string(), now);
            now
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
        if let Ok(output) = Command::new("hyprctl").arg("activewindow").arg("-j").output() {
            if output.status.success() {
                if let Ok(active) = serde_json::from_slice::<HyprlandClient>(&output.stdout) {
                    self.set_active(&active.address);
                }
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
                last_active: now - 10,
            },
            WindowEntry {
                title: "A2".into(),
                class: "AppA".into(),
                address: "2".into(),
                workspace: "1".into(),
                last_active: now - 5,
            },
            WindowEntry {
                title: "B1".into(),
                class: "AppB".into(),
                address: "3".into(),
                workspace: "2".into(),
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
                last_active: now - 1,
            },
            WindowEntry {
                title: "A2".into(),
                class: "AppA".into(),
                address: "2".into(),
                workspace: "1".into(),
                last_active: now - 2,
            },
            WindowEntry {
                title: "B1".into(),
                class: "AppB".into(),
                address: "3".into(),
                workspace: "2".into(),
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
