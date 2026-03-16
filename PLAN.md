# Vanta Master Plan (Reliability -> UX -> Platform Scale)

Transform Vanta into the most usable and trusted Linux launcher by sequencing reliability first, then UX/workflow excellence, then platform-scale capabilities.

Release policy:
- Patch (`vX.Y.Z`): bugfix/reliability only
- Minor (`vX.Y.0`): backward-compatible features
- Major (`vX.0.0`): breaking contracts or config/runtime architecture changes

Phase template:
- **Goal**
- **Prerequisites**
- **What gets done**
- **Definition of Done**

### Phase 01 - v2.2.1 (Patch) - Stability Baseline Lock
- **Goal:** Eliminate current UI/runtime regressions and establish a trusted baseline.
- **Prerequisites:** Current `main` at v2.2.x.
- **What gets done:**
  - Fix launcher section visibility edge cases, icon color regressions in release builds, and focus/navigation glitches.
  - Resolve merge-conflict leftovers, config parsing hazards, and packaging warnings that can break release confidence.
  - Add crash-safe guards around extension/icon rendering fallbacks.
- **Definition of Done:**
  - `npm run check`, `cargo test`, and release smoke build pass on CI.
  - No known critical regressions in launcher/settings/store flows.

### Phase 02 - v2.2.2 (Patch) - Performance And Input Reliability
- **Goal:** Make keyboard-driven interaction feel instant and deterministic.
- **Prerequisites:** Phase 01.
- **What gets done:**
  - Fix selection index desync under rapid result updates.
  - Remove timing-based focus hacks; use condition/frame-based focus transitions.
  - Reduce avoidable list rerenders/reflows in high-frequency typing.
- **Definition of Done:**
  - No navigation jumps under stress typing.
  - Input latency and result render latency stay within target budget.

### Phase 03 - v2.3.0 (Minor) - First-Run And Onboarding UX
- **Goal:** New users succeed in first 60 seconds.
- **Prerequisites:** Phase 02.
- **What gets done:**
  - Add first-run wizard (hotkey setup, theme preview, file search defaults).
  - Add quick tips panel and "try these commands" seeded examples.
  - Add guided permission explanations for scripts/extensions.
- **Definition of Done:**
  - New-user activation funnel measurable and improved.
  - Users can launch apps/files/scripts without docs.

### Phase 04 - v2.4.0 (Minor) - Visual Design System v1
- **Goal:** Make Vanta visually distinctive, polished, and consistently readable.
- **Prerequisites:** Phase 03.
- **What gets done:**
  - Introduce semantic design tokens (surface/text/accent/icon roles) and contrast-safe defaults.
  - Unify typography scale, spacing rhythm, and section hierarchy.
  - Add robust SVG/icon color strategy for dev + release parity.
- **Definition of Done:**
  - No visual inconsistency across launcher/settings/store.
  - Accessibility contrast checks pass for default themes.

### Phase 05 - v2.5.0 (Minor) - Search Quality v2
- **Goal:** Best-in-class relevance for Linux launcher use-cases.
- **Prerequisites:** Phase 04.
- **What gets done:**
  - Hybrid ranking model (recency, frequency, context source weight).
  - Per-source weighting presets (developer, creator, minimal).
  - Explainability panel: why each result ranked where it did.
- **Definition of Done:**
  - Top-3 hit rate improves on benchmark query set.
  - Ranking is tunable and immediately reflected in results.

### Phase 06 - v2.6.0 (Minor) - Action Model v3
- **Goal:** Turn results into workflows, not just launches.
- **Prerequisites:** Phase 05.
- **What gets done:**
  - Rich secondary actions (reveal, copy path, open with, run as).
  - Action chips and discoverable shortcuts in-row.
  - Safe-confirmation layer for destructive actions.
- **Definition of Done:**
  - Action success and discoverability metrics improve.
  - No accidental destructive action execution.

### Phase 07 - v2.7.0 (Minor) - Clipboard And History Excellence
- **Goal:** Make clipboard mode a daily-driver productivity surface.
- **Prerequisites:** Phase 06.
- **What gets done:**
  - Pinned/favorite snippets, semantic grouping, dedupe controls.
  - Fast preview and one-key paste/copy/reveal actions.
  - Privacy controls (sensitive content filtering rules).
- **Definition of Done:**
  - Clipboard retrieval speed remains instant at large history sizes.
  - Privacy filters are enforced and auditable.

### Phase 08 - v2.8.0 (Minor) - Theme Studio And Presets
- **Goal:** Make customization powerful and safe for all users.
- **Prerequisites:** Phase 07.
- **What gets done:**
  - Theme editor with live preview, undo stack, and reset-to-safe default.
  - Curated preset packs (minimal, vivid, high-contrast, compact).
  - Import/export with validation and schema diagnostics.
- **Definition of Done:**
  - Invalid themes never break rendering.
  - Theme switching is instant and reversible.

### Phase 09 - v2.9.0 (Minor) - Store Discoverability And Trust
- **Goal:** Make extensions easy to discover and safe to install.
- **Prerequisites:** Phase 08.
- **What gets done:**
  - Store categories, ratings, install count, changelogs.
  - Strong manifest validation before install.
  - Permission risk labels and trust badges.
- **Definition of Done:**
  - Install failures return actionable errors.
  - Users understand permission impact before install.

### Phase 10 - v2.10.0 (Minor) - Accessibility Completion
- **Goal:** Full keyboard-first and readability-first operation.
- **Prerequisites:** Phase 09.
- **What gets done:**
  - Complete tab order/focus states in every surface.
  - Reduced-motion, text scaling, spacing presets.
  - Screen-reader labels and ARIA audits across key views.
- **Definition of Done:**
  - Primary workflows are fully keyboard operable.
  - Accessibility audits pass agreed baseline.

### Phase 11 - v2.11.0 (Minor) - Window And Workspace Intelligence
- **Goal:** Make window switching a superpower for power users.
- **Prerequisites:** Phase 10.
- **What gets done:**
  - Better grouping (app/class/workspace), recency weighting.
  - Workspace-aware quick actions (focus/move/minimize where supported).
  - Graceful degradation on unsupported compositors.
- **Definition of Done:**
  - Window actions remain reliable across compositor diversity.
  - Switching is measurably faster than current baseline.

### Phase 12 - v2.12.0 (Minor) - Jobs And Async Workflows
- **Goal:** Support long tasks without blocking launcher flow.
- **Prerequisites:** Phase 11.
- **What gets done:**
  - Job lifecycle UI (running/succeeded/failed/retry/cancel).
  - Persistent notifications and task history.
  - Background-safe script/macro execution model.
- **Definition of Done:**
  - Long jobs never block query/navigation.
  - Job state survives restarts where expected.

### Phase 13 - v3.0.0 (Major) - Unified Command Contract
- **Goal:** Establish a typed, future-proof core contract across all result/action sources.
- **Prerequisites:** Phase 12.
- **What gets done:**
  - Replace legacy result/action payload variants with typed protocol.
  - Versioned schema for scripts/extensions.
  - Migration tooling for config + extension compatibility.
- **Definition of Done:**
  - Contract migration is automated and documented.
  - Backward compatibility policy is explicit for ecosystem maintainers.

### Phase 14 - v3.1.0 (Minor) - Profiles v2 (Context-Aware Environments)
- **Goal:** Let users switch instantly between work/personal/project contexts.
- **Prerequisites:** Phase 13.
- **What gets done:**
  - Atomic profile switching with scoped sources, theme, hotkeys, ranking.
  - Profile import/export with compatibility checks.
  - Optional auto-switch by workspace/window rules.
- **Definition of Done:**
  - No settings leakage across profiles.
  - Profile operations are deterministic and reversible.

### Phase 15 - v3.2.0 (Minor) - Diagnostics, Recovery, And Support Bundle v2
- **Goal:** Make bug reports reproducible and fixes faster.
- **Prerequisites:** Phase 14.
- **What gets done:**
  - Health dashboard for key subsystems.
  - One-click sanitized diagnostics bundle.
  - Startup self-heal suggestions based on previous failure state.
- **Definition of Done:**
  - Support bundles contain useful, redacted data.
  - Recovery hints reduce repeated startup failures.

### Phase 16 - v3.3.0 (Minor) - Enterprise Security And Policy Controls
- **Goal:** Make Vanta deployable in stricter environments.
- **Prerequisites:** Phase 15.
- **What gets done:**
  - Policy controls for extension/script sources and permissions.
  - Signed index enforcement and optional allowlists.
  - Audit trail for install/permission decisions.
- **Definition of Done:**
  - Policies are enforceable, testable, and documented.
  - Restricted mode prevents unauthorized paths.

### Phase 17 - v3.4.0 (Minor) - Extension SDK v2
- **Goal:** Make third-party extension development easy, stable, and high quality.
- **Prerequisites:** Phase 16.
- **What gets done:**
  - Better SDK docs, templates, test harness, and linting.
  - UI component primitives for extension views.
  - Compatibility matrix and deprecation lifecycle tooling.
- **Definition of Done:**
  - New extension authors can ship from template quickly.
  - SDK upgrade paths are predictable and low-friction.

### Phase 18 - v3.5.0 (Minor) - Packaging And Distribution Excellence
- **Goal:** Be installable and updatable smoothly across Linux ecosystems.
- **Prerequisites:** Phase 17.
- **What gets done:**
  - Harden deb/rpm/AppImage pipelines and checksum/signing docs.
  - Improve AUR update automation/checks.
  - Add post-install verification and troubleshooting guidance.
- **Definition of Done:**
  - Release artifacts pass CI + install smoke tests per distro target.
  - Packaging regressions are caught before publish.

### Phase 19 - v3.6.0 (Minor) - Community Growth Features
- **Goal:** Increase adoption through sharing and discoverability.
- **Prerequisites:** Phase 18.
- **What gets done:**
  - Shareable macros/themes/profile snippets with trust metadata.
  - "Popular workflows" discovery feed (opt-in).
  - In-app feedback channel and roadmap voting hooks.
- **Definition of Done:**
  - Community artifacts are easy to import safely.
  - Feedback loop measurably informs roadmap priorities.

### Phase 20 - v4.0.0 (Major) - Adaptive Intent Engine
- **Goal:** Deliver next-generation launcher intelligence while preserving speed and trust.
- **Prerequisites:** Phase 19.
- **What gets done:**
  - Intent-driven query interpretation (commands, entities, actions) with strict local-first/privacy boundaries.
  - Multi-step suggestions ("Do X then Y") with explicit confirmations.
  - New extensibility contract for adaptive ranking/intents.
- **Definition of Done:**
  - Intent features improve successful first-action rate.
  - Privacy and safety constraints are explicit and enforceable.
  - Migration path from v3.x contract is documented and reliable.

### Phase 21 - v4.1.0 (Minor) - Smartest Search And Result Intelligence
- **Goal:** Deliver the smartest possible search and ranking flow across every source.
- **Prerequisites:** Phase 20.
- **What gets done:**
  - Unify ranking into a global relevance model across apps, commands, extensions, files, windows, workflows, and clipboard items.
  - Resolve natural language entity references to most relevant results (for example, `zen` -> `Zen Browser`).
  - Continuously improve intent disambiguation and step prediction with strict local-first privacy boundaries.
  - Remove rigid category-first presentation in favor of relevance-first ordering.
- **Definition of Done:**
  - Top result accuracy improves significantly on benchmark query suites.
  - Multi-step intent execution consistently targets best-match entities.
  - Results are presented in pure relevance order independent of source category.

### Phase 22 - v4.2.0 (Minor) - Universal UI Redesign Foundation
- **Goal:** Establish a beautiful, unified, and universal visual direction across every Vanta surface.
- **Prerequisites:** Phase 21.
- **What gets done:**
  - Define Design Language v2 (token architecture, spatial rhythm, hierarchy, semantic color roles).
  - Introduce universal layout primitives for shell, panel, card, form, list, and modal surfaces.
  - Publish compatibility contract so existing `default.css` and user themes still render correctly.
- **Definition of Done:**
  - All main surfaces can opt into Design Language v2 without regressions.
  - Theme compatibility matrix exists for legacy + new classes/tokens.

### Phase 23 - v4.3.0 (Minor) - Launcher Layout Recomposition
- **Goal:** Redesign launcher composition for clarity, speed, and visual cohesion.
- **Prerequisites:** Phase 22.
- **What gets done:**
  - Rework search/input/results/status spatial structure into balanced, responsive regions.
  - Standardize item affordances, hover/selected states, and action hint visibility rules.
  - Add adaptive density modes (comfortable, default, compact) using shared spacing tokens.
- **Definition of Done:**
  - Launcher layout feels cohesive at all supported sizes.
  - No spacing/typography inconsistency across launcher sections.

### Phase 24 - v4.4.0 (Minor) - Settings IA And Visual Refresh
- **Goal:** Make settings universally understandable, fast, and visually consistent.
- **Prerequisites:** Phase 23.
- **What gets done:**
  - Redesign settings information architecture by user intent (discoverability, behavior, appearance, integrations).
  - Replace ad-hoc control styling with standardized form and section components.
  - Introduce sticky context header, search-within-settings, and inline explainers.
- **Definition of Done:**
  - Users can find top settings in <= 3 interactions.
  - Settings visuals fully match Design Language v2.

### Phase 25 - v4.5.0 (Minor) - Store And Extension Surfaces Redesign
- **Goal:** Make extension discovery and trust evaluation visually first-class.
- **Prerequisites:** Phase 24.
- **What gets done:**
  - Redesign store cards, filter rails, detail states, and install/uninstall flows with unified components.
  - Improve trust presentation (permission chips, risk labels, verification states) using consistent visual semantics.
  - Align extensions hub and extension-host surfaces with shared layout primitives.
- **Definition of Done:**
  - Store and extensions hub feel like one coherent product.
  - Trust-related information is scannable before install actions.

### Phase 26 - v4.6.0 (Minor) - Dedicated Window UX Suite v2
- **Goal:** Polish all dedicated windows into premium, consistent experiences.
- **Prerequisites:** Phase 25.
- **What gets done:**
  - Refresh Feature Hub, Community Hub, Theme Studio, and Extensions Hub with universal shell and card language.
  - Standardize close/back behavior, breadcrumbs, and contextual action bars.
  - Add responsive layout rules for desktop and narrower form factors.
- **Definition of Done:**
  - Dedicated windows share one visual grammar and navigation model.
  - Window-to-window transitions feel predictable and polished.

### Phase 27 - v4.7.0 (Minor) - Motion System And Interaction Choreography
- **Goal:** Make interactions feel alive without sacrificing speed or accessibility.
- **Prerequisites:** Phase 26.
- **What gets done:**
  - Define motion primitives (enter, exit, emphasis, hierarchy transition, list stagger) with reduced-motion fallbacks.
  - Add consistent micro-interactions for hover, focus, selection, and action confirmation.
  - Ensure no animation blocks typing, navigation, or command execution.
- **Definition of Done:**
  - Motion is consistent across all major views.
  - Reduced-motion mode remains fully respected.

### Phase 28 - v4.8.0 (Minor) - Typography And Readability Program
- **Goal:** Achieve beautiful and highly readable text hierarchy in all contexts.
- **Prerequisites:** Phase 27.
- **What gets done:**
  - Introduce a refined type scale for display/title/body/meta with language-safe fallbacks.
  - Harmonize line-height, measure, truncation, and emphasis rules for dense launcher usage.
  - Improve contrast and readability at custom text scales.
- **Definition of Done:**
  - Text hierarchy is consistent in launcher, settings, hubs, and store.
  - Readability audits pass across supported scale presets.

### Phase 29 - v4.9.0 (Minor) - Iconography And Visual Semantics v2
- **Goal:** Make icons and visual cues expressive, predictable, and brand-consistent.
- **Prerequisites:** Phase 28.
- **What gets done:**
  - Standardize icon sizing, stroke/weight strategy, and semantic coloring.
  - Improve fallback strategies for missing app icons and extension assets.
  - Add universal status semantics for success/warn/error/info/neutral across components.
- **Definition of Done:**
  - Icon behavior is consistent in dev/release builds.
  - State colors communicate meaning without ambiguity.

### Phase 30 - v5.0.0 (Major) - Theme Engine v3 And Token Inheritance
- **Goal:** Let themes deeply customize appearance while retaining guaranteed UX quality.
- **Prerequisites:** Phase 29.
- **What gets done:**
  - Introduce layered token inheritance (core -> semantic -> component -> context).
  - Add compatibility aliases so old themes remain functional under redesigned layouts.
  - Add theme diagnostics for missing tokens and invalid ranges.
- **Definition of Done:**
  - Legacy and new themes render safely under the new UI.
  - Theme diagnostics produce actionable guidance.

### Phase 31 - v5.0.1 (Patch) - Critical Security And Crash Prevention
- **Goal:** Eliminate every known crash vector and injection vulnerability before adding features.
- **Prerequisites:** Phase 30.
- **What gets done:**
  - Fix `unwrap()` on extension manifest JSON parsing in `extensions.rs` — malformed manifests currently crash the app.
  - Sanitize TERMINAL env variable parsing in `launcher.rs` — no length limit creates a DoS vector.
  - Add proper shell escaping in workflow token substitution in `workflows.rs` — placeholder values are interpolated without escaping, allowing command injection.
  - Add a frontend error boundary component wrapping the root — currently any JS error kills the window with no recovery.
  - Bundle Font Awesome locally instead of CDN `<link>` in `app.html` — single point of failure, no fallback if CDN is down.
- **Definition of Done:**
  - No `unwrap()` on user-supplied data in production code.
  - Crafted TERMINAL/placeholder values cannot crash or inject.
  - Frontend catches and recovers from render errors.
  - Icons render offline.

### Phase 32 - v5.1.0 (Minor) - Unified Error Architecture
- **Goal:** Replace all `Result<T, String>` returns with a typed error system so the frontend can programmatically handle failures.
- **Prerequisites:** Phase 31.
- **What gets done:**
  - Rewrite `errors.rs` using the already-installed `thiserror` crate — currently defines `VantaError` enum but every function returns `String` instead.
  - Add structured error codes (e.g., `ExtManifestInvalid = 1001`) so the frontend can distinguish error types.
  - Surface the 20+ silent `warn!()` calls in `launcher.rs` (window focus/close/minimize failures) as user-visible error events.
  - Add error context preservation (chained errors) instead of `format!()` string allocations.
  - Replace all remaining `unwrap()`/`expect()` in non-test code with proper `?` propagation.
- **Definition of Done:**
  - Zero production `unwrap()`.
  - Frontend receives typed error codes.
  - All user-visible operations surface failure feedback.
  - `thiserror` derive macros used consistently.

### Phase 33 - v5.2.0 (Minor) - Frontend Architecture Decomposition
- **Goal:** Break the two god components into focused, testable modules to unblock all future frontend velocity.
- **Prerequisites:** Phase 32.
- **What gets done:**
  - Split `+page.svelte` (~1500 lines, 50+ state variables) into focused modules: SearchController, ViewManager, KeyboardNavigator, ThemeInjector, MacroRunner, PermissionGate.
  - Split `SettingsView.svelte` (~1000 lines) into per-section components: GeneralSettings, ThemeSettings, AccessibilitySettings, FileSearchSettings, DiagnosticsSettings.
  - Extract command/macro scoring logic from inline functions into a dedicated `ranking.ts` module.
  - Add loading skeleton / shimmer states for all async operations (theme load, extension list, diagnostics fetch).
  - Add proper error recovery UI (config load failure shows retry instead of blank screen).
- **Definition of Done:**
  - No component exceeds 300 lines.
  - Each module has a single responsibility.
  - Loading states visible for every async fetch.
  - Error recovery shows actionable UI.

### Phase 34 - v5.3.0 (Minor) - Accessibility Compliance
- **Goal:** Achieve WCAG AA compliance across all primary surfaces — keyboard-first, screen-reader-friendly, contrast-safe.
- **Prerequisites:** Phase 33.
- **What gets done:**
  - Add `:focus-visible` ring on every interactive element (result items, buttons, inputs, accordion headers) — currently keyboard navigation is invisible.
  - Add `role="dialog"`, `aria-modal="true"`, and focus trapping on all modals (permission prompt, action confirmation, settings, store).
  - Add `aria-live="polite"` regions for result count changes, search status, and view transitions.
  - Remove the global `*::-webkit-scrollbar { display: none !important }` and replace with an accessible custom scrollbar that preserves scroll-position feedback.
  - Add a keyboard shortcut help modal (Ctrl+?) listing all navigation shortcuts.
  - Fix WCAG AA contrast violations: `--text-tertiary` and `.icon-fallback` both fail minimum 4.5:1 ratio.
  - Add skip-navigation links from search input to results region.
- **Definition of Done:**
  - All interactive elements have visible focus indicators.
  - Modals trap focus.
  - Dynamic content is announced to screen readers.
  - Contrast passes 4.5:1 for all text.
  - Keyboard shortcut discoverability exists.

### Phase 35 - v5.4.0 (Minor) - Parallel Search And Ranking Engine
- **Goal:** Make search feel instant by querying sources in parallel, streaming results, and cancelling stale queries.
- **Prerequisites:** Phase 34.
- **What gets done:**
  - Parallelize source querying in `lib.rs` — currently apps, files, windows, extensions, and clipboard run sequentially; use `tokio::join!` or spawn tasks.
  - Add result streaming — emit partial results to frontend as each source completes, do not wait for all.
  - Add search cancellation — when a new query arrives, abort in-flight queries for the previous one.
  - Replace raw history counters in `history.rs` with time-weighted frecency (exponential decay so old entries fade).
  - Extract 25+ hardcoded magic scoring numbers from `lib.rs` into a `ranking_config.rs` module with documented constants.
  - Add negative scoring to suppress low-quality/stale matches.
- **Definition of Done:**
  - Search sources run concurrently.
  - Results appear progressively.
  - New keystrokes cancel stale searches.
  - Ranking constants are documented and centralized.
  - Frecency reflects actual recency.

### Phase 36 - v5.5.0 (Minor) - Search UX And Query Intelligence
- **Goal:** Give users powerful query tools and always keep them informed about what the launcher is doing.
- **Prerequisites:** Phase 35.
- **What gets done:**
  - Add search filters: `type:app`, `type:file`, `type:window`, `in:clipboard`, `ext:xyz` to narrow results to a specific source.
  - Add recent searches / query history — recall previous queries with up-arrow or dropdown.
  - Add typo tolerance / "did you mean" suggestions using edit-distance matching.
  - Add search input placeholder with smart contextual guidance (e.g., "Search apps, files, scripts…").
  - Add a searching/loading animation during query execution in the status bar.
  - Add result count announcement for screen readers via `aria-live` update.
  - Make action chips clickable — currently they display shortcut text but do not respond to mouse clicks.
- **Definition of Done:**
  - Users can filter by source.
  - Previous queries are recallable.
  - Typos get correction suggestions.
  - Status bar shows searching state.
  - Action chips work with both keyboard and mouse.

### Phase 37 - v5.6.0 (Minor) - Result Preview And Context Actions
- **Goal:** Let users inspect results before acting and provide full action flexibility.
- **Prerequisites:** Phase 36.
- **What gets done:**
  - Add Quick Look preview panel — press Space on a selected result to see file contents, app details, script output, or clipboard preview in a side panel.
  - Add right-click context menu with the full action set (open, copy path, reveal in file manager, open with, run as, copy to clipboard).
  - Add file thumbnails for image/PDF results in the result row.
  - Improve empty states — show contextual help based on current mode and query instead of generic "Type to search".
  - Add fuzzy match score visibility in a debug/explain mode for power users.
  - Add drag-and-drop from result items to external windows (file paths, text).
- **Definition of Done:**
  - Space opens a preview panel.
  - Right-click opens action menu.
  - Image results show thumbnails.
  - Empty states are helpful, not blank.
  - Results can be dragged to external targets.

### Phase 38 - v5.7.0 (Minor) - Notification And Feedback System
- **Goal:** Never leave the user wondering "did it work?" — every async action gets visible feedback.
- **Prerequisites:** Phase 37.
- **What gets done:**
  - Build a toast notification system (non-blocking, bottom-right, auto-dismiss with severity levels).
  - Add progress indicators for long operations: index rebuild, extension install/update, macro execution, theme import.
  - Replace all silent `console.error` calls in frontend with user-visible error toasts.
  - Add visual success/failure feedback for every user action (copy path → "Copied!", launch → "Opening…", install → progress bar).
  - Add a notification center accessible from the status bar to review recent events.
  - Status bar live updates during operations (e.g., "Rebuilding index… 2847/50000 files").
- **Definition of Done:**
  - Every async operation shows progress.
  - Every success/failure shows feedback.
  - No action is silently ignored.
  - Recent notifications are reviewable.

### Phase 39 - v5.8.0 (Minor) - Extension Lifecycle And Ecosystem Health
- **Goal:** Make extensions upgradeable, rollbackable, and granularly permissioned — ready for an ecosystem.
- **Prerequisites:** Phase 38.
- **What gets done:**
  - Add automatic update checking — compare installed version against registry, notify user of available updates.
  - Add version management with rollback — keep previous version on disk, allow one-click revert.
  - Add granular permission scoping — `Filesystem` restricted to specific paths, `Network` restricted to specific domains, `Shell` restricted to specific commands.
  - Add extension hot-reload — apply changes without requiring app restart.
  - Add inter-extension dependency resolution — manifest can declare `requires: [{ ext_id, version_range }]`.
  - Surface all extension errors to users instead of silent `console.error` — use the Phase 38 toast system.
- **Definition of Done:**
  - Users get notified of updates.
  - Rollback preserves previous version.
  - Permissions are scoped with path/domain restrictions.
  - Extensions reload without restart.
  - Dependencies resolve before install.

Implementation notes (March 2026):
- Enforce scoped permissions at runtime in extension bridge commands (`extension_fetch`, `extension_shell_execute`) using manifest allowlists.
- Keep backward compatibility: if scope arrays are omitted/empty for legacy extensions, allow existing behavior but emit warnings and guide migration.
- Add `permission_scopes` to extension manifests (`network_domains`, `shell_commands`, `filesystem_paths`) and provide scoped defaults for first-party extensions.
- Trigger hot-reload from extension filesystem watcher (`extensions-changed`) and remount active extension host view when current extension changes.
- Block install when dependency requirements (`requires`) are not satisfied and return user-actionable errors.

### Phase 40 - v5.9.0 (Minor) - Workflow Engine v2 And Automation
- **Goal:** Make workflows powerful enough for real automation — conditions, error handling, and composition.
- **Prerequisites:** Phase 39.
- **What gets done:**
  - Add conditional execution — `if` step checks exit code, output content, or system state; `else` branch on failure.
  - Add step output chaining — Step 1's stdout/result becomes available as `{step.1.output}` in Step 2.
  - Add per-step error handling — configurable retry count, skip-on-failure flag, and cleanup/finally steps.
  - Add workflow timeouts — per-step and per-workflow max duration with configurable behavior (abort/skip).
  - Add workflow composition — a step can invoke another workflow by ID.
  - Add scheduled workflows — run on system events (login, network connect) or periodic (daily, hourly).
  - Add a secrets/credential store for sensitive variables (encrypted at rest, never logged).
- **Definition of Done:**
  - Workflows support branching.
  - Step outputs chain.
  - Failures do not leave partial state without cleanup.
  - Workflows can nest and schedule.
  - Sensitive data is encrypted.

Implementation notes (March 2026):
- Added step-output chaining support in blocking workflow execution: successful system-step stdout is now exposed as `{step.N.output}` for downstream steps.
- Introduced conditional workflow steps with `if` branches and `else` fallback (`kind: "if"`) and condition expressions for step-output checks and command exit-code checks.
- Extended dry-run preflight to traverse conditional branches for capability visibility so permission prompts happen before job execution.
- Non-blocking `run_macro` now returns a clear error for conditional workflows and routes users toward macro job execution path.
- Added per-step error policy (`on_error`) with retry count, skip-on-failure behavior, and recursive `finally_steps` cleanup execution in the blocking runner.
- Added timeout controls for workflow and system steps (`timeout_ms`, `timeout_behavior` with `abort|skip`) and enforce timeout behavior during blocking macro execution.
- Added workflow composition step support (`kind: "workflow"`) so a macro can invoke another macro by ID with argument mapping, including cycle detection and dry-run expansion.
- Added periodic scheduled workflow execution support via optional macro `schedule` config (enabled, interval_minutes, run_on_startup) with background runner emitting `scheduled-workflow-run` events.
- Added encrypted workflow secrets store (AES-GCM, encrypted at rest) with management commands and `secret.<name>` placeholder support in workflows, including runtime redaction in step result args.
- Extended scheduled workflows with event triggers (`startup`, `network_connected`) in addition to interval polling, with online-edge detection to avoid continuous re-runs while already connected.
- Added startup-safe UI/window state persistence via `ui-state.json`, restoring launcher size/position and last active view across restarts while continuously saving move/resize/view updates.
- Added centralized config audit trail (`config-audit.jsonl`) with timestamp/source (`user`, `migration`, `community`, `indexer`, etc.) and recursive JSON-path diffs for every config mutation.

### Phase 41 - v5.10.0 (Minor) - Config And State Management v2
- **Goal:** Make configuration resilient, auditable, and stateful across restarts.
- **Prerequisites:** Phase 40.
- **What gets done:**
  - Replace the 25+ `.clone()` calls on `VantaConfig` in `config.rs` with `Arc<RwLock<>>` for shared immutable access.
  - Add config change audit trail — every mutation logged with timestamp, source (user/migration/extension), and diff.
  - Add window state persistence — remember position, size, last active view, and restore on next launch.
  - Add factory reset capability — one-click restore to defaults with confirmation.
  - Add config diff visualization — show what changed between current and previous versions.
  - Add external JSON Schema validation alongside Serde defaults — generate `config.schema.json` for editor tooling.
- **Definition of Done:**
  - Config mutations are auditable.
  - Window state survives restarts.
  - Factory reset resets all sections.
  - Config validates against external schema.
  - Memory usage drops from reduced cloning.

### Phase 42 - v5.11.0 (Minor) - Smart Features And Daily Driver Polish
- **Goal:** Add the small features that make Vanta indispensable for daily use.
- **Prerequisites:** Phase 41.
- **What gets done:**
  - Calculator enhancements — unit conversion (km↔mi, kg↔lb), currency conversion (offline rates), hex/bin/oct number bases, timezone lookups.
  - Smart clipboard — auto-detect pasted content type (URL → offer open, JSON → offer format, color hex → show swatch, command → offer execute) and offer contextual actions.
  - File bookmarks — star frequently accessed files/folders for instant recall without typing.
  - Quick note capture — type `note: buy groceries` to save a quick note retrievable later as `note:` search.
  - Command palette mode — type `>` to access internal Vanta commands (like VS Code).
  - Usage analytics dashboard — local-only statistics showing most-used apps, peak usage times, search patterns (no telemetry).
- **Definition of Done:**
  - Calculator handles units/currency/bases.
  - Clipboard actions are context-aware.
  - Bookmarks persist across sessions.
  - Notes are searchable.
  - `>` accesses internal commands.
  - Usage stats are viewable.

### Phase 43 - v5.12.0 (Minor) - Performance And Resource Optimization
- **Goal:** Strip waste, bound caches, and ensure Vanta stays fast at scale.
- **Prerequisites:** Phase 42.
- **What gets done:**
  - CSS deduplication — extract shared component styles into a `base.css`, theme files only override color/spacing tokens (eliminate ~2000 lines of copy-paste between themes).
  - Add icon cache bounds and integrity checks — currently `icon-cache.json` grows unbounded with no verification.
  - Remove unused `rusqlite` crate dependency from `Cargo.toml` if confirmed unused — reduces binary size.
  - Lazy-load non-critical views (store, community hub, theme studio, extensions hub) — do not bundle upfront.
  - Add missing file extension icons — Java, C/C++, Python, Go, Kotlin, Ruby, .docx/.xlsx/.pptx.
  - Pin Svelte dependency to exact version in `package.json` — `^5.0.0` auto-upgrades to potentially breaking 5.x releases.
  - Add visual performance budget to CI — detect layout thrash and excessive style recalculations.
- **Definition of Done:**
  - Theme CSS deduplication eliminates >1500 lines of duplication.
  - Icon cache has size limits and integrity checks.
  - Binary is smaller.
  - Non-critical views load on demand.
  - File icons cover all common development and office formats.

### Phase 44 - v5.13.0 (Minor) - Universal Layout Runtime
- **Goal:** Promote all major surfaces to a shared layout runtime and component contract.
- **Prerequisites:** Phase 43.
- **What gets done:**
  - Replace remaining one-off layout implementations with universal shell/panel/list/form/card primitives.
  - Migrate all internal surfaces to a single spacing/typography/motion contract.
  - Publish migration docs for extension UI authors.
- **Definition of Done:**
  - Every first-party surface uses the universal layout runtime.
  - Major-version migration notes are complete and tested.

### Phase 45 - v5.14.0 (Minor) - Workspace Canvas And Multi-Pane Views
- **Goal:** Enable richer productivity layouts while preserving launcher speed.
- **Prerequisites:** Phase 44.
- **What gets done:**
  - Add optional multi-pane composition for advanced workflows (results + preview + actions).
  - Persist per-profile layout preferences.
  - Keep single-pane fast mode as a first-class default.
- **Definition of Done:**
  - Multi-pane mode improves complex workflows without performance regressions.
  - Single-pane mode remains instant and stable.

### Phase 46 - v5.15.0 (Minor) - Contextual Surfaces And Smart Panels
- **Goal:** Present the right UI affordances for the current intent/context.
- **Prerequisites:** Phase 45.
- **What gets done:**
  - Add context-sensitive side panels for details, actions, and safety context.
  - Introduce intent-specific panel templates (files, scripts, extensions, windows, workflows).
  - Ensure keyboard-first navigation across all contextual panels.
- **Definition of Done:**
  - Contextual panels reduce action time for advanced tasks.
  - Panels are discoverable and fully keyboard operable.

### Phase 47 - v5.16.0 (Minor) - Data Visualization And System Health UX
- **Goal:** Make system state and metrics beautiful, useful, and actionable.
- **Prerequisites:** Phase 46.
- **What gets done:**
  - Redesign diagnostics and health dashboards with clear visual hierarchy and semantic states.
  - Add trend cards, sparkline visualizations, and risk summaries.
  - Align diagnostics visuals with universal design language.
- **Definition of Done:**
  - Health information is quickly interpretable.
  - Visuals are consistent with all other Vanta surfaces.

### Phase 48 - v5.17.0 (Minor) - Input System v2 And Command Composer
- **Goal:** Make command composition powerful while keeping interaction elegant.
- **Prerequisites:** Phase 47.
- **What gets done:**
  - Introduce command composer UI (arguments, previews, and safety constraints).
  - Improve keyboard hinting and inline completion visuals.
  - Add composable command templates for repeated power-user tasks.
- **Definition of Done:**
  - Advanced commands are easier to compose correctly.
  - Error-prevention guidance appears before risky execution.

### Phase 49 - v5.18.0 (Minor) - Collaborative Artifact UX
- **Goal:** Make community snippets/macros/themes feel premium and safe to consume.
- **Prerequisites:** Phase 48.
- **What gets done:**
  - Redesign share/import/export flows with richer previews and trust metadata.
  - Add source attribution and compatibility badges in artifact cards.
  - Improve rollback and conflict handling visuals for imports.
- **Definition of Done:**
  - Artifact sharing/import is visually clear and confidence-inspiring.
  - Users can safely undo problematic imports.

### Phase 50 - v5.19.0 (Minor) - Global Navigation And Cross-Surface Wayfinding
- **Goal:** Make movement between all Vanta surfaces feel effortless and coherent.
- **Prerequisites:** Phase 49.
- **What gets done:**
  - Add universal wayfinding patterns (breadcrumbs, section memory, return points).
  - Standardize global shortcuts and contextual nav hints.
  - Reduce dead-ends by providing clear next-step actions in every surface.
- **Definition of Done:**
  - Users can move between launcher/settings/hubs/store without confusion.
  - Navigation consistency score improves in usability testing.

### Phase 51 - v5.20.0 (Minor) - Adaptive Appearance Profiles
- **Goal:** Automatically keep the UI beautiful and readable across environments.
- **Prerequisites:** Phase 50.
- **What gets done:**
  - Add adaptive appearance profiles (lighting, display density, performance tier, accessibility prefs).
  - Support per-profile style transforms with strict compatibility guards.
  - Ensure dynamic adaptation is deterministic and reversible.
- **Definition of Done:**
  - Appearance adaptation improves comfort without surprise behavior.
  - Profile-specific styling remains stable across restarts.

### Phase 52 - v5.21.0 (Minor) - Performance-First Visual Runtime
- **Goal:** Guarantee visual polish at scale without sacrificing responsiveness.
- **Prerequisites:** Phase 51.
- **What gets done:**
  - Optimize paint/composition paths for heavy lists, rich cards, and animated transitions.
  - Add visual budget checks to CI (layout thrash, style recalculation hotspots).
  - Introduce graceful degradation rules for low-end systems.
- **Definition of Done:**
  - Visual runtime meets latency/perf targets under stress.
  - Polished visuals remain smooth on supported hardware tiers.

### Phase 53 - v6.0.0 (Major) - Design QA Automation And UI Contract Testing
- **Goal:** Keep the universal UI system reliable as features evolve.
- **Prerequisites:** Phase 52.
- **What gets done:**
  - Add screenshot/regression tests for key surfaces and interaction states.
  - Validate theme token completeness and compatibility in CI.
  - Add lint-like checks for forbidden one-off styling patterns.
- **Definition of Done:**
  - Visual regressions are detected pre-release.
  - Styling contract violations fail CI with clear diagnostics.

### Phase 54 - v6.1.0 (Minor) - Vanta Experience Platform
- **Goal:** Deliver a fully unified, beautiful, and extensible experience platform.
- **Prerequisites:** Phase 53.
- **What gets done:**
  - Finalize universal UI architecture across first-party and extension surfaces.
  - Ship platform-level design governance (component lifecycle, token policy, migration standards).
  - Provide long-term compatibility guarantees for themes, layouts, and extension UI integrations.
- **Definition of Done:**
  - Vanta ships with end-to-end visual and interaction continuity.
  - Platform documentation and migration tooling are complete for ecosystem authors.

### Global Release Gates (apply to every phase)
- Backend quality gate: `cargo test`
- Frontend quality gate: `npm run check`
- Build gate: `tauri build` smoke run
- Docs gate: update README/config migration notes/changelog
- Rollback gate: define fallback for partial failures
