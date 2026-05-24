# Official Metadata Addon Provider Relevance Budget — Evidence And Gates

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

- `docs/workstreams/official-metadata-addon-provider-relevance-budget/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-search-merge`
- `docs/workstreams/official-metadata-addon-provider-degraded-candidates`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/engine/title.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## 2026-05-24 OMPRB-020

Review result:

- Workstream compliance: PASS with no blocking findings. TMDB changes stay inside the provider
  `suggest` path and shared ranking helper; HTTP runtime, payload shape, and final runtime ranking
  were not expanded.
- Code quality: PASS with no blocking findings. TMDB now collects all deduped title-variant search
  results, ranks cheap search-result candidates with provider-neutral facts, and enriches the
  strongest three.
- Residual risk: live TMDB search payload drift remains follow-on scope.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_prioritizes_more_relevant_merged_search_results_for_enrichment --no-fail-fast`: PASS, 1 test passed. Proves a stronger normalized TMDB search result can displace earlier weak raw-title results before detail enrichment.
- `cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast`: PASS, 23 tests passed and 48 skipped. Proves TMDB relevance-budget behavior plus existing ranking/title coverage.

## 2026-05-24 OMPRB-030

Review result:

- Workstream compliance: PASS with no blocking findings. Bangumi changes mirror the TMDB budget
  policy without changing provider public contracts or network policy.
- Code quality: PASS with no blocking findings. Bangumi now collects all deduped title-variant
  search results, ranks cheap search-result candidates with provider-neutral facts, and enriches the
  strongest three.
- Residual risk: live Bangumi search payload drift remains follow-on scope.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper bangumi_provider_prioritizes_more_relevant_merged_search_results_for_enrichment --no-fail-fast`: PASS, 1 test passed. Proves a stronger normalized Bangumi search result can displace earlier weak raw-title results before detail enrichment.
- `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast`: PASS, 22 tests passed and 50 skipped. Proves Bangumi relevance-budget behavior plus existing ranking/title coverage.

## 2026-05-24 OMPRB-040 Closeout

Review result:

- Workstream compliance: PASS with no blocking findings. OMPRB-010 through OMPRB-030 are complete,
  and remaining live-provider validation is explicitly deferred.
- Code quality: PASS with no blocking findings. A shared ranking helper selects provider inputs from
  provider-neutral candidate facts while raw search schemas remain provider-local.
- Residual risk: the cheap pre-enrichment score is bounded by facts available in search payloads;
  detail-only facts can still improve final runtime ranking after enrichment.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`: PASS, 29 tests passed and 43 skipped. Proves both providers prioritize relevant merged search results while preserving search merge, degraded candidate, ranking, and title helper behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 72 tests passed. Proves the package-level metadata scraper surface after relevance-budget closeout.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 72 tests passed. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

## 2026-05-24 Language Primary Subtag Ranking Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper ranking_evidence_matches_language_primary_subtags ranking_evidence_matches_language_primary_subtags_with_script ranking_evidence_penalizes_title_year_and_external_id_mismatch --no-fail-fast`: expected RED before implementation, then PASS 3. Proves ranking treats `en-US`/`en` and `zh-Hans-CN`/`zh` as language matches while preserving different-primary-language mismatch behavior.
- `cargo nextest run -p nako-metadata-scraper ranking tmdb bangumi title --no-fail-fast`: PASS 61. Proves language primary-subtag matching composes with TMDB/Bangumi relevance-budget selection, search-title variants, external-ID lookup, degraded candidates, and payload resilience behavior.

## 2026-05-24 External ID Case Normalization Ranking Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper ranking_evidence_matches_external_id_values_case_insensitively ranking_evidence_keeps_external_id_value_mismatch ranking_evidence_penalizes_title_year_and_external_id_mismatch --no-fail-fast`: expected RED before implementation, then PASS 3. Proves ranking treats case-only external ID value differences such as `TT0133093`/`tt0133093` as exact while preserving distinct-ID mismatch behavior.
- `cargo nextest run -p nako-metadata-scraper ranking tmdb bangumi title --no-fail-fast`: PASS 63. Proves external ID case normalization composes with TMDB/Bangumi relevance-budget selection, TMDB IMDb find lookup, direct lookup, search-title variants, degraded candidates, and payload resilience behavior.
