# Changelog

<<<<<<< Updated upstream
## 1.16.0
- Workflow macros beta: define multi-step macros with variables, optional step prompts, dry-run previews, and per-macro enable toggles.
- Launcher UI for macros: shows macro search results with script details, status bar prompts, and argument editing before run.
- Default workflow sample macro shipped for easier onboarding and testing.
=======
## 1.18.0
- Launcher sections are always visible with static headers; no expand/collapse, all items shown per section.
- Section headers stay selectable without jumping back to Apps when navigating.
- Rust backend tests (`cargo test`) pass.
- Known issue: AppImage bundling via linuxdeploy fails to strip RELR ELF sections; `deb`/`rpm` builds not yet verified in this run.
>>>>>>> Stashed changes

## 1.14.0
- Window Switcher v2: grouped per app/class with recency ordering and focus/close actions.
- Windows section shows in launcher results with optional cap derived from `windows_max_results`.
- Frontend sections render with subheaders for window groups and preserve Apps → Documents → Settings order when idle.

## 1.13.0
- File search v2: include/exclude globs, allowed extensions, and type filter.
- Manual index rebuild with freshness indicator in Settings.
- New manual QA checklist for file search filters.
