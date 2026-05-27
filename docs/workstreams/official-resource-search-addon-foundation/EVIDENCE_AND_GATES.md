# Evidence And Gates

## Required Gates

- `cargo nextest run -p nako-resource-search --no-fail-fast`
- `cargo fmt -p nako-resource-search -- --check`
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-addon-foundation`

## Optional Manual Gates

These are not default gates because live resource providers and download tools
need network, credentials, proxy, and operator policy decisions.

- `cargo run -p nako-resource-search`
- `pwsh -File addons/resource-search/smoke.local.ps1`

## Evidence Log

### 2026-05-28 - ORSAF-010

- Workstream opened.
- PanSou reference findings recorded as architecture takeaways.
- Nako core protocol changes deferred to a later host-side lane.

Not run yet:

- Package tests. `nako-resource-search` has not been added yet.
