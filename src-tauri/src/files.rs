use crate::config::FilesConfig;
use crate::matcher::ResultSource;
use crate::matcher::SearchResult;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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
pub type FileIndex = Arc<Mutex<Vec<FileEntry>>>;

/// Build the index (blocking – call from a background thread).
pub fn build_index(config: &FilesConfig) -> Vec<FileEntry> {
    let home_dir = dirs::home_dir().unwrap_or(PathBuf::from("/"));
    let mut entries: Vec<FileEntry> = Vec::with_capacity(50_000);

    let walker = WalkDir::new(&home_dir)
        .max_depth(config.max_depth)
        .follow_links(false)
        .into_iter();

    for entry in walker.filter_entry(|e| config.include_hidden || !is_hidden(e)) {
        if let Ok(entry) = entry {
            // Skip the root itself
            if entry.depth() == 0 {
                continue;
            }

            let name_display = entry.file_name().to_string_lossy().to_string();
            let path_obj = entry.path();
            let path_str = path_obj.to_string_lossy().to_string();

            let icon = if path_obj.is_dir() {
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

    entries
}

/// Quick search against the in-memory index. Returns up to `limit` results.
pub fn search_index(index: &[FileEntry], query: &str, limit: usize) -> Vec<SearchResult> {
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

    for entry in index {
        // Empty term = list everything (up to limit)
        if term.is_empty() || entry.name.contains(&term) {
            results.push(SearchResult {
                title: entry.name_display.clone(),
                subtitle: Some(entry.path.clone()),
                icon: Some(entry.icon.clone()),
                exec: entry.path.clone(),
                score: 50,
                match_indices: vec![],
                source: ResultSource::Application,
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
    let new_entries = build_index(config);
    if let Ok(mut guard) = index.lock() {
        *guard = new_entries;
    }
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

    #[test]
    fn test_build_and_search_index() {
        let config = FilesConfig {
            include_hidden: false,
            max_depth: 2,
        };
        let index = build_index(&config);
        println!("Indexed {} entries", index.len());

        let results = search_index(&index, "/Do", 10);
        println!("Results for '/Do':");
        for r in &results {
            println!("  {}", r.title);
        }
        assert!(!index.is_empty());
    }
}
