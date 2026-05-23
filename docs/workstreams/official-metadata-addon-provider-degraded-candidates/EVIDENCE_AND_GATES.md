# Official Metadata Addon Provider Degraded Candidates — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Hygiene Gates

```bash
cargo fmt --all -- --check
git diff --check
```

## Evidence Anchors

- `docs/workstreams/official-metadata-addon-provider-degraded-candidates/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-degraded-candidates/TODO.md`
- `docs/workstreams/official-metadata-addon-provider-enrichment-resilience`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## 2026-05-24 OMPDC-020

Review result:

- Workstream compliance: PASS with no blocking findings. The behavior stays inside TMDB and
  Bangumi provider modules; routes, browser-worker behavior, host task runtime, and HTTP retry
  policy were not expanded.
- Code quality: PASS with no blocking findings. Search-result types build degraded candidates from
  existing provider-neutral facts, add provider-specific degraded tags, and keep redaction-safe
  `provider_note` text.
- Residual risk: live provider payload drift and operator-facing partial-result warning semantics
  remain follow-on scope.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`: PASS, 23 tests
  passed and 46 skipped. Proves TMDB and Bangumi return degraded candidates after candidate detail
  enrichment failure, still merge search-title variants, and keep ranking behavior covered.

## 2026-05-24 OMPDC-030 Closeout

Review result:

- Workstream compliance: PASS with no blocking findings. OMPDC-010 and OMPDC-020 are complete, and
  remaining degraded/error-reporting work is explicitly deferred.
- Code quality: PASS with no blocking findings. Fully enriched candidates still use detail
  responses; only per-candidate detail failures degrade to search-result facts.
- Residual risk: live network validation is intentionally not part of this synthetic provider gate.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`: PASS, 23 tests
  passed and 46 skipped. Proves the degraded candidate behavior and related provider ranking tests.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 70 tests passed. Proves the
  package-level metadata scraper surface after degraded candidate closeout.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 70 tests passed. Proves the full workspace
  test suite remains green for the current release scope.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.
