# Changelog

## 4.6.0
- Added Phase 26 Dedicated Window UX Suite v2 with shared shell, header, action, and scroll primitives for dedicated windows.
- Added consistent breadcrumb and action grammar across Feature Hub, Community Hub, Theme Studio, Extensions Hub, and Extension Host.
- Improved responsive dedicated-window behavior with stacked header actions and tighter mobile-safe panel spacing.
- Improved maintainability by removing remaining inline block layout styles in dedicated window views.

## 4.5.0
- Added Phase 25 Store and extensions surface redesign with shared visual primitives across Store, Extensions Hub, and Extension Host.
- Improved trust and install decision scanability by separating risk/trust badges from metadata stats in store cards.
- Improved extension action clarity with stronger install/uninstall status feedback and consistent action rhythm.
- Improved settings extension section alignment with extension ecosystem primitives and direct store entry actions.

## 4.4.0
- Added Phase 24 settings information architecture refresh with a sticky context header and in-settings section search jump.
- Added canonical settings section alias handling so deep-links resolve reliably across legacy and normalized section names.
- Improved settings visual consistency by replacing inline layout styles with reusable utility classes.
- Added missing shared styling primitives for hint lists and hub-window shell regions.

## 4.3.0
- Added Phase 23 launcher layout recomposition with balanced search, results, and status regions.
- Added adaptive density-aware spacing behavior across launcher surfaces using shared spacing tokens.
- Improved result affordance clarity with stronger selected-row contrast, cleaner hover states, and grouped/flat spacing harmony.
- Improved grouped section navigation feedback with selected-state parity for section headers.

## 4.2.0
- Added Phase 22 design primitive foundations with reusable shell, panel, card, stack, and status patterns.
- Added expanded semantic theme token mapping and runtime fallbacks for elevated surfaces, status roles, and border hierarchy.
- Improved launcher and settings layout consistency with density-aware spacing tokens and shared container primitives.
- Improved result and status surfaces with tokenized spacing and cleaner visual hierarchy.

## 4.1.0
- Added a global relevance-first ranking pass for typed queries across apps, files, windows, calculator, extensions, macros, and commands.
- Added app-aware intent entity resolution so multi-step queries like `open zen then open foot` map to the best matching installed apps.
- Added query-aware ranking bonuses and source-intent priors to improve top-result accuracy for natural language searches.
- Improved search consistency with case-insensitive fuzzy matching and stronger launch-history learning.

## 4.0.0
- Added a local-first Adaptive Intent Engine path that detects multi-step intent queries.
- Added intent workflow result generation for chained commands (for example `open settings then open store`).
- Added typed `intent_workflow` command contract support and frontend execution wiring.
- Added explicit confirmation before executing inferred adaptive workflows.

## 3.6.0
- Added a Phase 19 Community settings section with in-app feedback submission and roadmap voting hooks.
- Added trusted snippet export/import flows for workflows, profiles, and theme presets.
- Added opt-in Popular Workflows feed support with one-click macro install.
- Added import safety guardrails that block risky unverified workflow snippets.

## 2.12.0
- Added async macro job lifecycle with persisted history (`running`, `succeeded`, `failed`, `canceled`).
- Added background jobs panel in launcher with retry/cancel controls and live updates.
- Added new workflow job commands: start/list/retry/cancel job, with permission preflight and action-safe status tracking.
- Added blocking system-step execution path for macro jobs to improve completion reporting.

## 2.11.0
- Added compositor-aware window action model in launcher results with explicit backend context (`Hyprland`, `Sway`, `X11`).
- Added workspace-aware quick action to move windows to the current workspace on supported compositors.
- Added minimize action support for Sway and X11, plus X11 window-close fallback.
- Improved graceful degradation by showing limited-action context when compositor capabilities are restricted.

## 2.10.0
- Added first-class accessibility settings with reduced-motion toggle, text-scale control, and spacing presets.
- Added accessibility preset shortcuts in Settings (`Focus`, `Readability`, `Balanced`) for faster setup.
- Added backend config compatibility for legacy configs with safe clamping of accessibility values.
- Improved keyboard accessibility visibility with stronger global `:focus-visible` treatment and reduced-motion-safe transitions.

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
