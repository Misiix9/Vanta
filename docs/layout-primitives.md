# Vanta Layout Primitives (v2)

Phase 44 introduced a canonical set of CSS layout primitives that every first-party surface and SDK component should use. All classes are global (defined in `base.css`) and compose with active theme tokens.

## Quick Reference

| Class | Purpose |
|---|---|
| `.v2-shell` | Root flex column, gap/padding via `--space-3` |
| `.v2-panel` | Bordered surface-1 container |
| `.v2-card` | Lighter bordered item container |
| `.v2-stack` | Flex column, gap via `--space-2` |
| `.v2-hstack` | Flex row (centered), gap via `--space-2` |
| `.v2-form-field` | Label+input column, gap via `--space-1` |
| `.v2-form-label` | Caption-sized secondary label |
| `.v2-form-help` | Caption-sized muted help text |
| `.v2-status-badge` | Pill badge (`.success`/`.warning`/`.danger`/`.info`) |

### Modifiers

| Class | Combines With |
|---|---|
| `.v2-fill` | Any root -- adds `height:100%; overflow:hidden` |
| `.v2-scroll` | Any container -- `flex:1; min-height:0; overflow:auto` |
| `.v2-header` | Header bar -- flex, space-between, border-bottom |
| `.v2-gap-xs` | Override gap to `--space-1` |
| `.v2-gap-lg` | Override gap to `--space-4` |

### Density

Wrap any surface in a density class to scale all `--space-*` values:

| Class | Scale |
|---|---|
| `.density-compact` | 0.88 |
| `.density-comfortable` | 1.0 (default) |
| `.density-relaxed` | 1.16 |

## Design Tokens

All primitives read from these CSS custom properties (set in the active theme's `:root`):

- **Spacing**: `--space-1` through `--space-5`, multiplied by `--vanta-space-scale`
- **Typography**: `--type-caption`, `--type-body`, `--type-heading`, `--type-display`
- **Surfaces**: `--ds-surface-0` (background), `--ds-surface-1` (panel), `--ds-surface-2` (elevated)
- **Text**: `--ds-text-primary`, `--ds-text-secondary`, `--ds-text-muted`
- **Border**: `--ds-border`
- **Accent**: `--ds-accent`, `--ds-success`, `--ds-warning`, `--ds-danger`, `--ds-info`
- **Radius**: `--radius` (container), `--radius-item` (items)
- **Motion**: `--motion-enter-duration`, `--motion-ease-enter`, `--motion-stagger-step`

## Usage Examples

### Full-screen view (hub/store pattern)

```html
<div class="v2-shell v2-fill">
  <div class="v2-header">
    <h2>Title</h2>
    <button class="btn-ghost">Back</button>
  </div>
  <div class="v2-scroll">
    <!-- scrollable content -->
  </div>
</div>
```

### Card list inside a panel

```html
<div class="v2-panel v2-stack">
  <div class="v2-card">Item 1</div>
  <div class="v2-card">Item 2</div>
</div>
```

### Form field

```html
<div class="v2-form-field">
  <label class="v2-form-label">Name</label>
  <input type="text" />
  <span class="v2-form-help">Your display name.</span>
</div>
```

## For Extension Authors

SDK components (`ExtList`, `ExtForm`, `ExtGrid`, `ExtDetail`, `ExtActionPanel`) automatically compose with v2 primitives. If you build custom HTML inside an extension, use `--vanta-*` CSS variables for colors and the `v2-*` classes for layout:

```js
container.innerHTML = `
  <div class="v2-stack">
    <div class="v2-card">Custom content</div>
  </div>
`;
```

These classes are injected globally by the theme runtime, so they are always available inside extension host containers.

## Migration Notes

Legacy class names (`hub-window-root`, `window-shell`, `ext-surface-card`, etc.) are still defined in `base.css` for backward compatibility. New code should prefer `v2-*` classes exclusively. The legacy aliases will be removed in a future major version.
