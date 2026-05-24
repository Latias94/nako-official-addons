# Official Metadata Addon Provider Search Variant Resilience — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast
```

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast
cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast
cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Workspace Gate

```bash
cargo nextest run --workspace --no-fail-fast
```

### Hygiene Gates

```bash
cargo fmt --all -- --check
git diff --check
```

## Evidence Anchors

- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-search-merge`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/engine/title.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## 2026-05-24 OMPSVR-020

Review result:

- Workstream compliance: PASS with no blocking findings. TMDB changes stay inside provider search
  orchestration; HTTP retry policy, payload shape, and final runtime ranking were not expanded.
- Code quality: PASS with no blocking findings. TMDB records retry-exhausted title-variant search
  failures, keeps useful earlier results, and returns the last search error only when no candidates
  can be salvaged.
- Residual risk: payload-visible partial-search warnings remain follow-on scope.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_preserves_search_results_when_later_title_variant_search_fails tmdb_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast`: PASS, 2 tests passed. Proves TMDB salvage and all-search-failed boundaries.
- `cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast`: PASS, 25 tests passed and 49 skipped. Proves TMDB search-variant resilience plus existing ranking/title coverage.

## 2026-05-24 OMPSVR-030

Review result:

- Workstream compliance: PASS with no blocking findings. Bangumi changes mirror the TMDB search
  variant policy without changing provider public contracts.
- Code quality: PASS with no blocking findings. Bangumi records retry-exhausted title-variant search
  failures, keeps useful earlier results, and returns the last search error only when no candidates
  can be salvaged.
- Residual risk: payload-visible partial-search warnings remain follow-on scope.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper bangumi_provider_preserves_search_results_when_later_title_variant_search_fails bangumi_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast`: PASS, 2 tests passed. Proves Bangumi salvage and all-search-failed boundaries.
- `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast`: PASS through the combined target gate below; Bangumi-specific search-variant tests are included in the 33-test combined provider/ranking/title gate.

## 2026-05-24 OMPSVR-040 Closeout

Review result:

- Workstream compliance: PASS with no blocking findings. OMPSVR-010 through OMPSVR-030 are complete,
  and remaining partial-error-reporting work is explicitly deferred.
- Code quality: PASS with no blocking findings. The search-variant failure policy is provider-local,
  while retry/backoff remains in `ProviderHttpRuntime`.
- Residual risk: live provider payload drift and payload-visible partial-search warnings are outside
  this synthetic gate.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`: PASS, 33 tests passed and 43 skipped. Proves both providers preserve earlier search results after later title-variant search failures while keeping merge, relevance-budget, degraded candidate, ranking, and title helper behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 76 tests passed. Proves the package-level metadata scraper surface after search-variant resilience closeout.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 76 tests passed. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.
