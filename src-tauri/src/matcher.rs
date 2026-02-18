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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ResultSource {
    Application,
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
) -> Vec<SearchResult> {
    let start = std::time::Instant::now();

    if query.is_empty() {
        return Vec::new();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Atom::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );

    let mut scored: Vec<(u32, Vec<u32>, &AppEntry)> = Vec::new();

    for app in apps {
        // Calculate history boost first
        let usage = usage_map.get(&app.exec).copied().unwrap_or(0);
        // Cap the bonus at 200 points (e.g. 40 launches) to prevent overuse from
        // completely overshadowing relevance.
        let usage_bonus = std::cmp::min(usage * 5, 200);

        // Match against name (primary)
        let mut haystack_buf = Vec::new();
        let haystack = Utf32Str::new(&app.name, &mut haystack_buf);
        let mut indices = Vec::new();

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
            score: score,
            match_indices: indices,
            source: ResultSource::Application,
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
