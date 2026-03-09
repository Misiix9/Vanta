# Changelog

## 2.2.1
- Search results now hide sections that have `0` matches; only populated sections are shown.
- Launcher section headers remain always open and non-interactive, with improved selection stability.
- Inline SVG icons now use sanitized markup with `currentColor`-based theming so release builds no longer fall back to black icons.
- Theme updates in `default.css`: thinner section header labels, white extension icons, and light-gray back/refresh/control icons.
- Clipboard mode focus behavior is now frame-synchronized for more reliable keyboard readiness.
- Permission modal UX now supports `Esc` as a safe deny shortcut.
- Store extension actions now show install/uninstall success and failure notices.
- Extension host error UI now includes a retry action.

## 1.16.0
- Workflow macros beta: define multi-step macros with variables, optional step prompts, dry-run previews, and per-macro enable toggles.
- Launcher UI for macros: shows macro search results with script details, status bar prompts, and argument editing before run.
- Default workflow sample macro shipped for easier onboarding and testing.

## 1.14.0
- Window Switcher v2: grouped per app/class with recency ordering and focus/close actions.
- Windows section shows in launcher results with optional cap derived from `windows_max_results`.
- Frontend sections render with subheaders for window groups and preserve Apps → Documents → Settings order when idle.

## 1.13.0
- File search v2: include/exclude globs, allowed extensions, and type filter.
- Manual index rebuild with freshness indicator in Settings.
- New manual QA checklist for file search filters.
