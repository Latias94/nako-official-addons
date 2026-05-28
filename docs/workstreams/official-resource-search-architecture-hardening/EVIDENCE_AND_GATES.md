# Evidence And Gates

## Required Gates

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-resource-search --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-architecture-hardening`

## Evidence Log

### 2026-05-28 - ORSAH-010

- Workstream opened.
- Current architecture reviewed after fixture and PanSou-compatible provider
  lanes.
- Refactor brief recorded in `DESIGN.md`.

### 2026-05-28 - ORSAH-020

- Split `domain.rs` into `domain::query`, `domain::link`, and
  `domain::result`.
- Added internal `ResourceSearchIntent` inference for free text, media title,
  external id, and exact link searches without changing alpha request/response
  payloads.
- Moved `ResourceLink` construction to `links` while keeping classification and
  normalization there.
- `cargo fmt -p nako-resource-search -- --check`: passed.
- `cargo nextest run -p nako-resource-search --no-fail-fast`: passed, 31 tests.
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-architecture-hardening`:
  passed.

### 2026-05-28 - ORSAH-030

- Added provider descriptors, capability names, and source policy
  classification.
- Added provider registry assembly outside `ResourceSearchRuntime::new`.
- Added redaction-safe provider diagnostics to health payloads.
- Preserved the default no-network runtime: fixture active, PanSou-compatible
  inactive until enabled with a base URL.
- `cargo fmt -p nako-resource-search`: passed.
- `cargo fmt -p nako-resource-search -- --check`: passed.
- `cargo nextest run -p nako-resource-search --no-fail-fast`: passed, 35
  tests.
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-architecture-hardening`:
  passed.
