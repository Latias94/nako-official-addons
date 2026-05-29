# Official Metadata Addon Provider Live Drift Checks — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Manual Live Gate

```bash
NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT=1 cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored
```

Result on 2026-05-24: passed. TMDB skipped cleanly without a read token in this workspace;
Bangumi direct lookup executed successfully against the live API.

### Formatting And Diff Gate

```bash
cargo fmt --all -- --check
git diff --check
```

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record blocking findings, missing
gates, and residual risks here or link to the review note.

## Evidence Anchors

- `docs/workstreams/official-metadata-addon-provider-live-drift-checks/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-live-drift-checks/TODO.md`
- `crates/nako-metadata-scraper/tests/live_provider_drift.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`

## Notes

The live gate is intentionally opt-in and ignored by default. TMDB requires an operator-supplied
read token for meaningful live execution. Bangumi uses the public live API path and the provider's
existing user-agent configuration.
