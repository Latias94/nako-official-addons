# Official Metadata Addon AV Field Policy Presets - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper config registry field_policy --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper config registry field_policy --no-fail-fast
cargo nextest run -p nako-metadata-scraper runtime manifest routes --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Format Gate

```bash
cargo fmt -p nako-metadata-scraper -- --check
```

### Docs And Diff Gate

```bash
python -m json.tool docs/workstreams/official-metadata-addon-av-field-policy-presets/WORKSTREAM.json
git diff --check
```

## Evidence Anchors

- `docs/workstreams/official-metadata-addon-av-field-policy-presets/DESIGN.md`
- `docs/workstreams/official-metadata-addon-av-field-policy-presets/TODO.md`
- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/providers/registry.rs`
- `crates/nako-metadata-scraper/src/engine/fusion.rs`
- `crates/nako-metadata-scraper/src/routes.rs`
- `crates/nako-metadata-scraper/src/manifest.rs`

## Fresh Evidence - 2026-05-27

- `cargo nextest run -p nako-metadata-scraper config registry field_policy manifest routes --no-fail-fast`
  - Result: 48 passed, 203 skipped.
  - Covers config parsing, registry policy construction, manifest schema, route diagnostics, and selected field policy tests.
- `cargo nextest run -p nako-metadata-scraper runtime manifest routes --no-fail-fast`
  - Result: 44 passed, 207 skipped.
  - Covers runtime fusion, manifest schema, example manifest parity, and route diagnostics.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
  - Result: 248 passed, 3 skipped.
  - Covers the full metadata scraper package, including all AV providers.
- `cargo fmt -p nako-metadata-scraper -- --check`
  - Result: passed after formatting modified Rust files.
- `python -m json.tool docs/workstreams/official-metadata-addon-av-field-policy-presets/WORKSTREAM.json`
  - Result: passed.
- `python -m json.tool addons/metadata-scraper/manifest.example.json`
  - Result: passed.
- `git diff --check`
  - Result: passed.

## Notes

- The targeted gates prove config parsing, policy construction, runtime precedence, and manifest
  exposure.
- The package gate proves this does not regress existing provider parsers and AV fusion behavior.
