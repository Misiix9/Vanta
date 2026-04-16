# Changelog

## Unreleased

- Ongoing work for next phase.

## 5.21.0

- Completed Phase 52 performance-first visual runtime with adaptive visual degradation for low-end and battery-sensitive environments.
- Added automatic runtime performance tiering in theme application (`full`/`balanced`/`degraded`) using adaptive performance intent plus device capability heuristics.
- Reduced high-cost animation/compositing paths by introducing performance-aware CSS overrides for blur, hover effects, and media backdrops.
- Extended visual budget CI checks to fail on `transition: all` regressions and excess layout-affecting dynamic style directives.

## 5.20.0

- Completed Phase 51 adaptive appearance profiles with environment-aware profile controls in Theme Profile and Theme Studio.
- Added adaptive appearance config schema (`appearance.adaptive`) with strict backend compatibility guards and safe migration defaults.
- Added deterministic runtime transforms for blur, opacity, radius, spacing, text scale, and motion based on profile, lighting, density, performance, and accessibility presets.
- Added profile-aware import/export handling so theme profile JSON round-trips adaptive settings safely.

## 5.19.0

- Completed Phase 50 global navigation and cross-surface wayfinding improvements.
- Added universal wayfinding overlay with breadcrumb trail, back/search actions, and contextual next-step guidance across non-launcher surfaces.
- Added navigation return-point memory so users can move back through recent surfaces without losing context.
- Added global shortcuts (`Alt+Left`, `Alt+Home`) for consistent navigation behavior across launcher surfaces.
- Added explicit next-step action buttons in the wayfinding layer to reduce dead-end navigation states.

## 5.18.0

- Completed Phase 49 collaborative artifact UX with preview-first import flow for workflow/profile/theme snippets.
- Added richer trust metadata and source attribution (publisher/source, compatibility hints, ratings, and vote context) across Community Hub artifact cards.
- Added import conflict handling strategies (`replace` or `keep_local`) so users can avoid overwriting local artifacts.
- Added rollback-safe import history with backend persistence plus in-app "Undo Latest Import" and per-entry rollback actions.
- Updated release workflow note generation to render changelog bullets once (no duplicate Highlights/Patch Notes sections).

## 5.17.0

- Synced Rust package metadata version in `src-tauri/Cargo.toml` with the current app release (`5.16.0`).
- Added CI workflow for pull requests and pushes to `main` with backend/frontend/visual/extension quality gates.
- Hardened release workflow with dynamic version checks, changelog-driven patch notes, and required pre-build verification steps.
- Updated packaging metadata (`PKGBUILD`, `vanta-bin/PKGBUILD`) and README install/version references.
- Started Phase 48 command composer implementation with reusable macro command templates (save/list/delete), `>` command-mode template search, and keyboard-first macro composer shortcuts (`Enter`, `Ctrl+Enter`, `Esc`).

## 5.16.0
- Completed Phase 47 data visualization and system health UX.
- Redesigned diagnostics and health dashboards with trend cards, sparklines, and clear semantic states.
- Added a Health Risk summary to make system health quickly interpretable and actionable.

## 5.15.0
- Completed Phase 46 contextual surfaces and smart panels.
- Added intent-specific contextual panel templates for files, windows, clipboard items, extension actions, and workflows.
- Added safety-context messaging in side panel for destructive system actions and elevated extension permissions.
- Added keyboard-first panel navigation (`Ctrl+Right` focus panel, `Ctrl+Left` return focus to search).

## 5.14.0
- Completed Phase 45 workspace canvas and multi-pane launcher views.
- Added optional multi-pane composition (results + inspector/actions) while preserving single-pane default mode.
- Added persisted `search.layout_mode` preference with per-profile carryover during profile snapshot updates.
- Added keyboard and status-bar layout switching (`Ctrl+\` and status toggle).
- Added workspace layout preference control in Settings > General.

## 5.13.0
- Completed Phase 44 universal layout runtime: canonical v2 layout vocabulary with documented primitives.
- Migrated all hub windows, StoreView, settings, and SDK components to use `v2-shell`/`v2-panel`/`v2-card`/`v2-stack` classes.
- Added `v2-fill`, `v2-scroll`, `v2-header`, `v2-gap-xs`, `v2-gap-lg` modifier classes for composable layouts.
- Moved notification center and permission modal styles from scoped CSS to global `base.css`.
- Removed dead universal layout compatibility classes (`layout-shell-v2`, `panel-universal`, etc.).
- Added proper CSS for the permission modal dialog.
- Added `docs/layout-primitives.md` with full migration guide for extension authors.

## 5.12.0
- Completed Phase 43 performance and resource optimization with shared theme CSS extraction into `base.css`.
- Added visual performance budget checks for inline layout styles and theme file size drift in the release workflow.
- Added bounded icon-cache pruning and integrity checks to keep cached icon metadata healthy at scale.
- Lazy-loaded non-critical hub views (`Store`, `Community Hub`, `Theme Studio`, and `Extensions Hub`) to reduce initial UI cost.
- Expanded file-type icon coverage for common developer and office formats.
- Pinned `svelte` to an exact version for release stability.

## 5.11.0
- Completed Phase 42 daily-driver polish with bookmarks, quick notes, command palette mode, smart clipboard actions, and local analytics.
- Added calculator enhancements for unit conversion, currency conversion, timezone lookup, and hex/bin/oct literal support.

## 5.10.0
- Completed Phase 41 config and state management improvements with config audit trail, window state persistence, factory reset, and config schema validation.
- Added diagnostics visibility for recent config diffs and mutation sources.

## 5.9.0
- Completed Phase 40 workflow engine v2 with conditional branches, step-output chaining, retry/finally handling, timeouts, workflow composition, schedules, and encrypted secrets.

## 5.8.0
- Completed Phase 39 extension lifecycle work with update detection, rollback support, dependency validation, and user-visible lifecycle feedback.

## 4.7.0
- Added Phase 27 motion system primitives (enter, emphasis, hierarchy, and stagger cadence) with reduced-motion-safe fallbacks.
- Added consistent micro-interaction choreography across launcher results, grouped headers, store filters, and extension cards.
- Added a full universal theme redesign (Aurora Slate) with refreshed palette depth, elevated surfaces, and refined visual hierarchy.
- Improved interaction responsiveness by keeping motion transform/opacity-first and non-blocking for typing and keyboard navigation.

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
