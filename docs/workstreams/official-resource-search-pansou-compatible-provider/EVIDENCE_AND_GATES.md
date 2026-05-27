# Evidence And Gates

## Required Gates

- `cargo nextest run -p nako-resource-search --no-fail-fast`
- `cargo fmt -p nako-resource-search -- --check`
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-pansou-compatible-provider`

## Optional Manual Gates

- Run a local PanSou-compatible service.
- Set `NAKO_RESOURCE_SEARCH_PANSOU_PROVIDER_ENABLED=1`.
- Set `NAKO_RESOURCE_SEARCH_PANSOU_BASE_URL=http://127.0.0.1:8888`.
- Run `pwsh -File addons/resource-search/smoke.local.ps1`.

## Evidence Log

### 2026-05-28 - ORSPC-010

- Workstream opened.
- Adapter boundary fixed as optional PanSou-compatible HTTP provider.
