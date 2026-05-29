# Official Browser Worker Provider Render Drift Presets - Handoff

Status: Complete
Last updated: 2026-05-27

## Current Phase

OBWRDP-010 through OBWRDP-040 are complete. Provider-owned Browser Worker render
drift presets are implemented, documented, validated, and ready to commit.

## Current Shape

- `providers::render_drift` owns Browser Worker-compatible case structs and
  enabled-provider case collection.
- `douban::render_drift_case` owns Douban search URL and selector.
- `javbus::render_drift_case` owns JavBus detail URL, selector, and optional
  age-gate actions.
- `javlibrary::render_drift_case` owns JavLibrary localized search URL and
  selector.
- `main.rs` supports `render-drift-cases` and `--render-drift-cases`.

## Next Action

Commit this lane. Follow-ups should add more rendered provider presets or an
explicit IMDb provider/recipe before adding IMDb presets.
