use nucleo_matcher::pattern::{Atom, AtomKind, CaseMatching, Normalization};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use serde::{Deserialize, Serialize};

use crate::scanner::AppEntry;

/// Search result returned to the frontend.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
    pub exec: String,
    pub score: u32,
    pub match_indices: Vec<u32>,
    pub source: ResultSource,
    #[serde(default)]
    pub actions: Option<Vec<ActionHint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionHint {
    pub label: String,
    pub exec: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shortcut: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResultSource {
    Application,
    Calculator,
    Window,
    File,
    Extension { ext_id: String },
}

fn apply_weight(score: u32, weight: u32) -> u32 {
    let clamped = weight.clamp(10, 300);
    let scaled = (score as u128 * clamped as u128) / 100;
    scaled.min(u32::MAX as u128) as u32
}

fn usage_relevance_bonus(usage: u32) -> u32 {
    if usage == 0 {
        return 0;
    }
    // Log-shaped growth keeps heavy users influential without dominating query relevance.
    let bucket = ((usage as f64 + 1.0).ln() * 75.0).round() as u32;
    bucket.min(350)
}

/// Perform fuzzy search across cached app entries using nucleo-matcher.
/// Returns top `max_results` entries sorted by score (descending).
/// Perform fuzzy search across cached app entries using nucleo-matcher.
/// Returns top `max_results` entries sorted by score (descending).
pub fn fuzzy_search(
    query: &str,
    apps: &[AppEntry],
    max_results: usize,
    usage_map: &std::collections::HashMap<String, u32>,
    app_weight: u32,
) -> Vec<SearchResult> {
    let start = std::time::Instant::now();

    if query.is_empty() {
        let mut scored: Vec<(u32, &AppEntry)> = apps
            .iter()
            .map(|app| {
                let usage = usage_map.get(&app.exec).copied().unwrap_or(0);
                (usage, app)
            })
            .collect();

        // Sort by usage descending, then by name alphabetically
        scored.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()))
        });

        return scored
            .into_iter()
            .map(|(score, app)| SearchResult {
                title: app.name.clone(),
                subtitle: app.generic_name.clone().or_else(|| app.comment.clone()),
                icon: app.icon.clone(),
                exec: app.exec.clone(),
                score: apply_weight(score, app_weight),
                match_indices: Vec::new(),
                source: ResultSource::Application,
                actions: None,
                id: None,
                group: None,
                section: Some("Apps".to_string()),
            })
            .collect();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Atom::new(
        query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );

    let mut scored: Vec<(u32, Vec<u32>, &AppEntry)> = Vec::new();
    let mut haystack_buf = Vec::new();
    let mut indices = Vec::new();

    for app in apps {
        let usage = usage_map.get(&app.exec).copied().unwrap_or(0);
        let usage_bonus = usage_relevance_bonus(usage);

        // Match against name (primary)
        haystack_buf.clear();
        indices.clear();
        let haystack = Utf32Str::new(&app.name, &mut haystack_buf);

        if let Some(score) = pattern.indices(haystack, &mut matcher, &mut indices) {
            let final_score = score as u32 + usage_bonus;
            scored.push((final_score, indices.clone(), app));
            continue;
        }

        // Match against generic name (secondary)
        if let Some(ref gname) = app.generic_name {
            haystack_buf.clear();
            indices.clear();
            let haystack = Utf32Str::new(gname, &mut haystack_buf);
            if let Some(score) = pattern.indices(haystack, &mut matcher, &mut indices) {
                // Slightly lower score for secondary matches
                let base_score = score.saturating_sub(10);
                let final_score = base_score as u32 + usage_bonus;
                scored.push((final_score, indices.clone(), app));
                continue;
            }
        }

        // Match against comment (tertiary)
        if let Some(ref comment) = app.comment {
            haystack_buf.clear();
            indices.clear();
            let haystack = Utf32Str::new(comment, &mut haystack_buf);
            if let Some(score) = pattern.indices(haystack, &mut matcher, &mut indices) {
                let base_score = score.saturating_sub(20);
                let final_score = base_score as u32 + usage_bonus;
                scored.push((final_score, indices.clone(), app));
            }
        }
    }

    // Sort by score descending
    scored.sort_by(|a, b| b.0.cmp(&a.0));

    let results: Vec<SearchResult> = scored
        .into_iter()
        .take(max_results)
        .map(|(score, indices, app)| SearchResult {
            title: app.name.clone(),
            subtitle: app.generic_name.clone().or_else(|| app.comment.clone()),
            icon: app.icon.clone(),
            exec: app.exec.clone(),
            score: apply_weight(score, app_weight),
            match_indices: indices,
            source: ResultSource::Application,
            actions: None,
            id: None,
            group: None,
            section: Some("Apps".to_string()),
        })
        .collect();

    let elapsed = start.elapsed();
    log::debug!(
        "Fuzzy search for '{}': {} results in {:?}",
        query,
        results.len(),
        elapsed
    );

    results
}

/// Fuzzy score an arbitrary text snippet, returning the score and matched indices.
pub fn fuzzy_score_text(query: &str, text: &str) -> Option<(u32, Vec<u32>)> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Atom::new(
        trimmed,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );

    let mut haystack_buf = Vec::new();
    let mut indices = Vec::new();
    let haystack = Utf32Str::new(text, &mut haystack_buf);

    pattern
        .indices(haystack, &mut matcher, &mut indices)
        .map(|score| (score as u32, indices))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::AppEntry;
    use std::collections::HashMap;

    fn app(name: &str, exec: &str) -> AppEntry {
        AppEntry {
            name: name.to_string(),
            generic_name: None,
            comment: None,
            exec: exec.to_string(),
            icon: None,
            categories: Vec::new(),
            terminal: false,
            startup_wm_class: None,
            desktop_file_path: String::new(),
        }
    }

    #[test]
    fn fuzzy_search_is_case_insensitive() {
        let apps = vec![app("Discord", "discord"), app("Foot", "foot")];
        let history = HashMap::new();
        let results = fuzzy_search("DISCORD", &apps, 10, &history, 100);
        assert!(!results.is_empty());
        assert_eq!(results[0].title, "Discord");
    }

    #[test]
    fn fuzzy_search_prefers_usage_when_relevance_is_close() {
        let apps = vec![app("Discord Canary", "discord-canary"), app("Discord", "discord")];
        let mut history = HashMap::new();
        history.insert("discord".to_string(), 120);
        let results = fuzzy_search("disc", &apps, 10, &history, 100);
        assert!(!results.is_empty());
        assert_eq!(results[0].exec, "discord");
    }
}
