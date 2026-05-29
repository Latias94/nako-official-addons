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

### 2026-05-28 - ORSAF-020 through ORSAF-040

- `cargo nextest run -p nako-resource-search --no-fail-fast`
  - Result: passed.
  - Coverage: 20 tests across config, manifest, routes, domain wire names,
    fixture provider, link classification, URL normalization, source filtering,
    link-type filtering, fusion deduplication, and checked-in example manifest
    parity.
- `cargo fmt -p nako-resource-search -- --check`
  - Result: passed.
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-addon-foundation`
  - Result: passed with the existing Windows line-ending warning for
    `Cargo.lock`.
- `cargo build -p nako-resource-search`
  - Result: passed.
- `pwsh -File addons/resource-search/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9130`
  - Result: passed after starting `target/debug/nako-resource-search.exe`
    locally.

Not run:

- Live provider or downloader smoke tests. They are follow-on gates after the
  host protocol and operator policy are explicit.
