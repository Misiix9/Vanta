use crate::matcher::ResultSource;
use crate::matcher::SearchResult;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn search_home(query: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // If query is empty or just "/", don't scan yet (too many files)
    // Wait for at least 2 chars after the prefix to be useful
    if query.trim().len() < 2 {
        return results;
    }

    // Determine root directory to search
    // If query starts with "~/", strip it and search home
    // If query starts with "/", search root (limited)

    let home_dir = dirs::home_dir().unwrap_or(PathBuf::from("/"));
    let search_root = home_dir.clone();
    let search_term;

    if query.starts_with("~/") {
        search_term = query[2..].to_string();
    } else if query.starts_with("/") {
        // Security/Perf: For now, strict mapping. "/" -> Home.
        // Real root search is dangerous deeply.
        // Let's treat "/" as "Home Search" for v1.5.0 unless user types absolute path
        search_term = query[1..].to_string();
    } else {
        // Fallback
        search_term = query.to_string();
    }

    // Limit walk depth and hidden files
    let walker = WalkDir::new(&search_root)
        .max_depth(3) // Don't go too deep for performance
        .into_iter();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        if let Ok(entry) = entry {
            let file_name = entry.file_name().to_string_lossy();

            // Simple subset match
            if file_name
                .to_lowercase()
                .contains(&search_term.to_lowercase())
            {
                let path_obj = entry.path();
                let path_str = path_obj.to_string_lossy().to_string();

                let icon = if path_obj.is_dir() {
                    "dir".to_string()
                } else {
                    // Get extension
                    path_obj
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|ext| format!("file:{}", ext.to_lowercase()))
                        .unwrap_or_else(|| "file".to_string())
                };

                results.push(SearchResult {
                    title: file_name.to_string(),
                    subtitle: Some(path_str.clone()),
                    icon: Some(icon),
                    exec: path_str,
                    score: 50,
                    match_indices: vec![],
                    source: ResultSource::Application,
                });
            }
        }
    }

    // Cap results
    results.truncate(20);
    results
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
