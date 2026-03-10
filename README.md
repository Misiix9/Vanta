# Vanta Extensions Registry

This repository folder hosts the official Vanta extension catalog.

## Metadata Contract
Each extension must define these manifest fields:
- `name`
- `title`
- `version`
- `schema_version`
- `author`
- `publisher`
- `safe`
- `permissions`
- `commands`

## Community Ratings
Ratings are published in `ratings.json` and surfaced in Vanta Store.

User rating flow:
1. In Vanta Store, click a score button (1-5).
2. Vanta opens a prefilled GitHub issue form.
3. Submitted ratings are visible publicly in the `vanta-extensions` issue tracker.
4. Maintainers can run `node tools/sync-ratings-from-issues.mjs` to aggregate averages into `ratings.json`.

## SDK v2 Quickstart
- Starter template: `templates/view-template/`
- Manifest validator: `node tools/validate-manifests.mjs`
- Ratings sync helper: `node tools/sync-ratings-from-issues.mjs --owner Misiix9 --repo vanta-extensions`

## Compatibility Matrix
See `compatibility-matrix.json` for supported SDK/runtime versions and deprecation windows.
