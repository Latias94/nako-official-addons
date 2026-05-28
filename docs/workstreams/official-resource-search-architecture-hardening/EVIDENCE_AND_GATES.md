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

Not run yet:

- Code validation for this lane. No code changes yet.
