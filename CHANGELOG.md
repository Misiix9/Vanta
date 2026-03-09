# Changelog

## 2.9.0
- Added Vanta Store discoverability metadata support for categories, ratings, install counts, trust badges, and changelog previews.
- Added category filtering and richer store cards with permission risk labels (`Low`/`Medium`/`High`) for clearer install decisions.
- Added stronger install-time manifest validation (name/title/commands checks) with explicit failure reasons.
- Improved install safety by rolling back partially downloaded extensions when required files fail to download/write.

## 2.8.0
- Added Theme Studio controls in Settings with preset packs (`Minimal`, `Vivid`, `High Contrast`, `Compact`).
- Added theme profile import/export workflow with JSON validation and copy support.
- Added one-click reset to safe default appearance values.
- Improved customization safety by clamping imported appearance and window values.

## 2.7.0
- Clipboard view now supports dedupe controls, grouping modes, and privacy masking for sensitive entries.
- Added grouped clipboard sections (`Pinned`/`Recent` with optional type grouping) for faster scanning at scale.
- Added keyboard-friendly clipboard actions (`C` copy, `P` pin/unpin, `Delete` delete, `R` reveal masked content).
- Added persistent user preferences for clipboard dedupe/grouping/masking toggles.

## 2.6.0
- Added a safe confirmation layer for destructive actions (system power actions and window close actions).
- Added a dedicated action confirmation modal with keyboard support (`Enter` confirm, `Esc` cancel).
- Improved action model safety while preserving existing shortcut-driven secondary action flow.

## 2.5.0
- Added Search Quality v2 source-weight presets in Settings (`Developer`, `Creator`, `Minimal`) for faster ranking tuning.
- Added in-app ranking explainability panel that shows why top results rank where they do (source, weight, score, and match metadata).
- Improved ranking visibility and tuning workflow without breaking existing search configuration compatibility.

## 2.4.0
- Introduced Phase 04 semantic design tokens for color roles, typography scales, and spacing rhythm in the default theme.
- Updated runtime theme application to publish both semantic and legacy CSS variables, preserving compatibility with existing themes.
- Improved visual hierarchy across launcher surfaces by aligning section labels, item typography, and status text to the new token system.
- Refined surface and border role usage to improve consistency across launcher and settings views.

## 2.3.0
- Added a first-run onboarding wizard with guided setup for quick-start commands, hotkey/theme personalization, file-search defaults, and permission guidance.
- Added an idle quick tips panel with seeded command chips to help new users discover launcher capabilities faster.
- Added persisted onboarding/tips state so onboarding runs once and tips can be dismissed.
- Added mobile-safe styling for onboarding surfaces and integrated them with the existing theme system.

## 2.2.2
- Replaced timeout-based keyboard scroll synchronization with frame-coalesced selection scrolling for more deterministic navigation under rapid input.
- Added teardown-safe cancellation for pending animation-frame scroll work to avoid stale callbacks during view lifecycle changes.
- Reworked launcher focus transitions (`tauri://show` and query-fill actions) to use frame-based focus scheduling instead of timing hacks.
- Improved stress-typing stability by reducing timing races between result updates and selected-row visibility.

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
