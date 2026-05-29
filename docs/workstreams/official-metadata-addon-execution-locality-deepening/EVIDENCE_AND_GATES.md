# Official Metadata Addon Execution Locality Deepening - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper bulk provider_execution --no-fail-fast
```

This proves the first slice: Bulk task behavior and provider execution reporting after removing the
internal JSON tunnel.

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-metadata-scraper bulk provider_execution --no-fail-fast
cargo nextest run -p nako-metadata-scraper provider registry config --no-fail-fast
cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Format And Hygiene

```bash
cargo fmt -p nako-metadata-scraper -- --check
git diff --check
python -m json.tool docs/workstreams/official-metadata-addon-execution-locality-deepening/WORKSTREAM.json
```

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record blocking findings, missing
gates, and residual risks in this file or a journal entry.

## Evidence Anchors

- `crates/nako-metadata-scraper/src/engine/bulk.rs`
- `crates/nako-metadata-scraper/src/engine/runtime.rs`
- `crates/nako-metadata-scraper/src/engine/provider_execution.rs`
- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/providers/registry.rs`
- `crates/nako-metadata-scraper/src/providers/render_drift.rs`
- `docs/workstreams/official-metadata-addon-execution-locality-deepening/TODO.md`

## Recorded Evidence

- 2026-05-27 OMAELD-020: `cargo nextest run -p nako-metadata-scraper bulk provider_execution --no-fail-fast` passed, 15 passed, 261 skipped. This covers Bulk task behavior and Provider Execution policy/report behavior after removing Bulk request JSON mutation.
- 2026-05-27 OMAELD-030: `cargo nextest run -p nako-metadata-scraper provider registry config --no-fail-fast` passed, 189 passed, 87 skipped. This covers provider catalog assembly, provider configuration, rendered-page config diagnostics, and provider-specific tests after moving rendered-page support facts into provider catalog entries.
- 2026-05-27 OMAELD-040: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast` passed, 6 passed, 270 skipped. This covers render drift output order, provider sample fallback behavior, proxy/session redaction, and disabled-provider filtering after moving drift case descriptors into provider catalog entries.
- 2026-05-27 OMAELD-050: `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed, 273 passed, 3 skipped. This covers the full metadata-scraper package after all three locality refactors.
- 2026-05-27 OMAELD-050: `cargo fmt -p nako-metadata-scraper -- --check` passed. This covers formatting for the touched Rust package.
- 2026-05-27 OMAELD-050: `python -m json.tool docs/workstreams/official-metadata-addon-execution-locality-deepening/WORKSTREAM.json` passed. This covers the workstream control file syntax.
- 2026-05-27 OMAELD-050: `git diff --check` passed after normalizing `crates/nako-metadata-scraper/src/providers/fixture.rs` back to LF line endings. This covers diff whitespace hygiene.

## Closeout Review

- Workstream compliance: no blocking findings. OMAELD-020, OMAELD-030, and OMAELD-040 match the target state and their evidence is recorded.
- Code quality: no blocking findings. Bulk no longer mutates request JSON for provider execution; central rendered-page config matching was removed; render drift routing is descriptor-driven and sorted by explicit order.
- Missing gates: none for this lane. The package gate, format check, JSON check, and diff hygiene gate passed.
- Residual risk: `DEFAULT_FIELD_PROVIDER_PREFERENCES` and related provider order tables still live centrally in `ProviderRegistry`. That is a future field-policy ownership lane, not a blocker for this execution-locality closeout.

## Notes

Fresh verification is required before marking each task complete. Prefer targeted nextest filters
during iteration and the full metadata-scraper package gate before closeout.
