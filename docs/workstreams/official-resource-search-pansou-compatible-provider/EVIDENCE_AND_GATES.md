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

### 2026-05-28 - ORSPC-020 through ORSPC-040

- Added disabled-by-default PanSou-compatible provider config and manifest
  schema.
- Added reqwest-backed adapter with no live network tests required.
- Added mapping tests for PanSou `results` and fallback `merged_by_type`
  payloads.
- Added token-redaction config tests and runtime provider-registration tests.

Validation:

- `cargo nextest run -p nako-resource-search --no-fail-fast`
  - Result: passed.
  - Coverage: 26 tests across fixture search, PanSou-compatible request
    shaping, response mapping, manifest parity, redaction, and route behavior.
- `cargo fmt -p nako-resource-search -- --check`
  - Result: passed.
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-pansou-compatible-provider`
  - Result: passed with the existing Windows line-ending warning for
    `Cargo.lock`.
- `cargo build -p nako-resource-search`
  - Result: passed.
- `pwsh -File addons/resource-search/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9130`
  - Result: passed against a temporary local sidecar process. Default runtime
    provider count remained 1, so PanSou-compatible network calls are not active
    by default.

Not run:

- Live PanSou-compatible service smoke. It requires an operator-managed
  external service and stays optional.
