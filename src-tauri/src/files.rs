use crate::config::FilesConfig;
use crate::matcher::{ActionHint, ResultSource, SearchResult};
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

/// A single indexed file entry (lightweight).
#[derive(Clone, Debug)]
pub struct FileEntry {
    pub name: String,         // file_name only, lowercased for fast matching
    pub name_display: String, // original-case display name
    pub path: String,         // full path string
    pub icon: String,         // "dir" or "file:ext"
}

/// The shared, in-memory file index.
#[derive(Clone, Debug, Default)]
pub struct FileIndexState {
    pub entries: Vec<FileEntry>,
    pub indexed_at: Option<u64>, // epoch millis
}

pub type FileIndex = Arc<Mutex<FileIndexState>>;

/// Build the index (blocking – call from a background thread).
pub fn build_index(config: &FilesConfig) -> FileIndexState {
    let default_home = dirs::home_dir().unwrap_or(PathBuf::from("/"));
    let root_dir = std::env::var("VANTA_FILE_INDEX_ROOT")
        .map(PathBuf::from)
        .unwrap_or(default_home);
    let mut entries: Vec<FileEntry> = Vec::with_capacity(50_000);

    let include_matcher = build_globset(&config.include_globs);
    let exclude_matcher = build_globset(&config.exclude_globs);

    let allowed_exts: HashSet<String> = config
        .allowed_extensions
        .iter()
        .map(|e| e.to_lowercase())
        .collect();

    let type_filter = config.type_filter.to_lowercase();

    let walker = WalkDir::new(&root_dir)
        .max_depth(config.max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if !config.include_hidden && is_hidden(e) {
                return false;
            }
            if let Some(exclude) = exclude_matcher.as_ref() {
                let target = e
                    .path()
                    .strip_prefix(&root_dir)
                    .unwrap_or_else(|_| e.path());
                if exclude.is_match(target) {
                    return false;
                }
            }
            true
        });

    for entry in walker {
        if let Ok(entry) = entry {
            if entry.depth() == 0 {
                continue;
            }

            if !config.include_hidden && is_hidden(&entry) {
                continue;
            }

            let path_obj = entry.path();

            let match_target = path_obj.strip_prefix(&root_dir).unwrap_or(path_obj);

            if let Some(exclude) = exclude_matcher.as_ref() {
                if exclude.is_match(match_target) {
                    continue;
                }
            }

            if let Some(include) = include_matcher.as_ref() {
                if !include.is_match(match_target) {
                    continue;
                }
            }

            let is_dir = entry.file_type().is_dir();
            match type_filter.as_str() {
                "file" if is_dir => continue,
                "dir" if !is_dir => continue,
                _ => {}
            }

            if !allowed_exts.is_empty() && entry.file_type().is_file() {
                let ext = path_obj
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s.to_lowercase());
                if ext.map(|e| !allowed_exts.contains(&e)).unwrap_or(true) {
                    continue;
                }
            }

            let name_display = entry.file_name().to_string_lossy().to_string();
            let path_str = path_obj.to_string_lossy().to_string();

            let icon = if is_dir {
                "dir".to_string()
            } else {
                path_obj
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| format!("file:{}", ext.to_lowercase()))
                    .unwrap_or_else(|| "file".to_string())
            };

            entries.push(FileEntry {
                name: name_display.to_lowercase(),
                name_display,
                path: path_str,
                icon,
            });
        }
    }

    let indexed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_millis() as u64);

    FileIndexState { entries, indexed_at }
}

/// Quick search against the in-memory index. Returns up to `limit` results.
pub fn search_index(index: &FileIndexState, query: &str, limit: usize) -> Vec<SearchResult> {
    // Strip the trigger prefix to get the actual search term.
    let term = if query.starts_with("~/") {
        &query[2..]
    } else if query.starts_with('/') {
        &query[1..]
    } else {
        query
    }
    .to_lowercase();

    let mut results = Vec::new();

    for entry in &index.entries {
        // Empty term = list everything (up to limit)
        if term.is_empty() || entry.name.contains(&term) {
            let is_dir = entry.icon == "dir";
            let mut actions: Vec<ActionHint> = Vec::new();

            // File/dir secondary actions
            actions.push(ActionHint {
                label: "Copy Path".to_string(),
                exec: format!("copy-path:{}", entry.path),
                shortcut: Some("Ctrl+Shift+C".to_string()),
            });

            if !is_dir {
                actions.push(ActionHint {
                    label: "Reveal".to_string(),
                    exec: format!("reveal:{}", entry.path),
                    shortcut: Some("Shift+Enter".to_string()),
                });
                actions.push(ActionHint {
                    label: "Open with Editor".to_string(),
                    exec: format!("open-with:{}", entry.path),
                    shortcut: Some("Alt+Enter".to_string()),
                });
            }

            results.push(SearchResult {
                title: entry.name_display.clone(),
                subtitle: Some(entry.path.clone()),
                icon: Some(entry.icon.clone()),
                exec: entry.path.clone(),
                score: 50,
                match_indices: vec![],
                source: ResultSource::File,
                actions: Some(actions),
            });

            if results.len() >= limit {
                break;
            }
        }
    }

    results
}

/// Rebuild the given index in-place with the new config.
pub fn rebuild(index: &FileIndex, config: &FilesConfig) {
    let new_state = build_index(config);
    if let Ok(mut guard) = index.lock() {
        *guard = new_state;
    }
}

fn build_globset(patterns: &[String]) -> Option<GlobSet> {
    if patterns.is_empty() {
        return None;
    }

    let mut builder = GlobSetBuilder::new();
    let mut added = 0;

    for pat in patterns {
        if let Ok(glob) = Glob::new(pat) {
            builder.add(glob);
            added += 1;
        } else {
            log::warn!("Skipping invalid glob pattern: {}", pat);
        }
    }

    if added == 0 {
        return None;
    }

    builder.build().ok()
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::FilesConfig;
    use std::fs;
    use std::io::Write;

    fn temp_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    #[test]
    fn test_build_and_search_index_filters() {
        let dir = temp_dir();
        let base = dir.path();

        let root = base.join("root");
        fs::create_dir_all(&root).unwrap();

        let md = root.join("keep.md");
        let rs = root.join("skip.rs");
        let hidden = root.join(".secret.txt");
        let subdir = root.join("logs");
        fs::create_dir_all(&subdir).unwrap();

        fs::write(&md, "hello").unwrap();
        fs::write(&rs, "fn main() {}").unwrap();
        fs::write(&hidden, "hidden").unwrap();
        let mut log_file = fs::File::create(subdir.join("today.log")).unwrap();
        writeln!(log_file, "log").unwrap();

        let config = FilesConfig {
            include_hidden: false,
            max_depth: 5,
            include_globs: vec!["**/*.md".to_string(), "**/*.log".to_string()],
            exclude_globs: vec!["**/logs/**".to_string()],
            allowed_extensions: vec!["md".to_string(), "log".to_string()],
            type_filter: "file".to_string(),
            ..Default::default()
        };

        // Temporarily set home to the temp dir for the walker
        std::env::set_var("VANTA_FILE_INDEX_ROOT", &root);
        let index = build_index(&config);
        std::env::remove_var("VANTA_FILE_INDEX_ROOT");

        assert!(index.indexed_at.is_some());
        // Should include keep.md, exclude rs (ext) and logs/ via exclude_globs
        let results = search_index(&index, "/keep", 10);
        assert_eq!(results.len(), 1);
        assert!(results[0].subtitle.as_ref().unwrap().ends_with("keep.md"));

        let log_results = search_index(&index, "/today", 10);
        assert!(log_results.is_empty());
        let hidden_results = search_index(&index, "/.secret", 10);
        assert!(hidden_results.is_empty());
    }
}
