# TODO

## ORSPC-010 - Open workstream

Status: complete
Owner: Codex
Dependencies: ORSAF foundation complete

Scope:

- Create durable workstream docs.
- Fix the adapter boundary as PanSou-compatible HTTP, not copied PanSou code.

Validation:

- Docs exist under this workstream path.

## ORSPC-020 - Config and manifest provider toggle

Status: pending
Owner: Codex
Dependencies: ORSPC-010

Scope:

- Add disabled-by-default PanSou-compatible config.
- Add environment parsing for base URL, token, source type, plugins, cloud
  types, and timeout.
- Reflect provider toggle and safe settings in manifest configuration schema.

Validation:

- Config tests cover enablement and token redaction.
- Manifest example matches runtime manifest.

## ORSPC-030 - Provider adapter and mapping

Status: pending
Owner: Codex
Dependencies: ORSPC-020

Scope:

- Add reqwest-backed provider adapter.
- Map PanSou `results` into resource search results.
- Map fallback `merged_by_type` into synthetic resource search results.
- Convert requested Nako link types into PanSou `cloud_types`.

Validation:

- Unit tests cover request shaping and response mapping without live network.

## ORSPC-040 - Verify, docs, and commit

Status: pending
Owner: Codex
Dependencies: ORSPC-030

Scope:

- Update README and addon docs.
- Run focused package gates.
- Commit only resource-search provider adapter files.

Validation:

- `cargo nextest run -p nako-resource-search --no-fail-fast`
- `cargo fmt -p nako-resource-search -- --check`
- `git diff --check -- Cargo.toml Cargo.lock README.md crates/nako-resource-search addons/resource-search docs/workstreams/official-resource-search-pansou-compatible-provider`
