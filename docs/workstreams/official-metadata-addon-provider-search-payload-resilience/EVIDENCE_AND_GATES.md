# Official Metadata Addon Provider Search Payload Resilience — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast
```

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-metadata-scraper tmdb ranking --no-fail-fast
cargo nextest run -p nako-metadata-scraper bangumi ranking --no-fail-fast
cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast
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

- `docs/workstreams/official-metadata-addon-provider-search-payload-resilience/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-search-variant-resilience`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## Evidence Log

### 2026-05-24 — OMPSP-020 TMDB Search Payload Resilience

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_skips_malformed_search_result_items --no-fail-fast`: expected RED before implementation, then PASS. Proves one malformed TMDB search result item does not discard a valid item from the same response.
- `cargo nextest run -p nako-metadata-scraper tmdb ranking --no-fail-fast`: PASS 24. Proves TMDB search-item tolerance remains compatible with provider ranking, direct lookup, search-variant resilience, relevance-budget selection, and degraded fallback behavior.

### 2026-05-24 — OMPSP-030 Bangumi Search Payload Resilience

- `cargo nextest run -p nako-metadata-scraper bangumi_provider_skips_malformed_search_subject_items --no-fail-fast`: expected RED before implementation, then PASS. Proves one malformed Bangumi search subject item does not discard a valid subject from the same response.
- `cargo nextest run -p nako-metadata-scraper bangumi ranking --no-fail-fast`: PASS 23. Proves Bangumi search-item tolerance remains compatible with provider ranking, direct lookup, search-variant resilience, relevance-budget selection, and degraded fallback behavior.

### 2026-05-24 — OMPSP-040 Closeout

Review result:

- Workstream compliance: PASS with no blocking findings. OMPSP-010 through OMPSP-030 are complete,
  and live provider drift checks remain explicitly out of scope.
- Code quality: PASS with no blocking findings. Search-item tolerance is provider-local, preserves
  strict top-level search response shape, and keeps detail parsing strict.
- Residual risk: if a provider changes the top-level search response shape, the provider still
  fails; live drift monitoring remains follow-on scope.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`: PASS 38. Proves both provider search-item tolerance slices compose with ranking, direct lookup, search-variant resilience, relevance-budget selection, and degraded fallback behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 84. Proves the package-level metadata scraper surface after search payload resilience closeout.
- `cargo nextest run --workspace --no-fail-fast`: PASS 84. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

### 2026-05-24 — OMPSP-050 All-Malformed Search Item Guard

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_reports_error_when_all_search_result_items_are_malformed --no-fail-fast`: expected RED before implementation, then PASS 1. Proves TMDB does not silently treat a non-empty search response as an empty success when every result item is malformed.
- `cargo nextest run -p nako-metadata-scraper bangumi_provider_reports_error_when_all_search_subject_items_are_malformed --no-fail-fast`: expected RED before implementation, then PASS 1. Proves Bangumi does not silently treat a non-empty search response as an empty success when every subject item is malformed.
- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`: PASS 46. Proves the all-malformed guard composes with TMDB/Bangumi search-item salvage, title variants, provider ranking, direct lookup, relevance-budget selection, and degraded fallback behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 89. Proves the package-level metadata scraper surface after the all-malformed item guard.
- `cargo nextest run --workspace --no-fail-fast`: PASS 89. Proves the full workspace test suite remains green after the all-malformed item guard.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable after the guard.
- `git diff --check`: PASS. Proves the diff has no whitespace errors before the evidence addendum.

### 2026-05-24 — Provider Release-Year Boundary Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_release_year_ignores_zero_year_values bangumi_release_year_ignores_zero_year_values --no-fail-fast`: PASS 2. Proves TMDB and Bangumi ignore zero-valued or overlong release-year prefixes when mapping provider payload dates into candidate facts, keeping ranking inputs and visible metadata from carrying `0` or truncated years as release years.

### 2026-05-24 — Provider Zero-ID Search Payload Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_search_response_skips_zero_id_items bangumi_search_response_skips_zero_id_items --no-fail-fast`: PASS 2. Proves TMDB and Bangumi skip zero-valued search result IDs before enrichment, preventing `movie/0` or `subjects/0` lookups from malformed-but-deserializable payload items.

### 2026-05-24 — Provider Text Boundary Normalization Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_candidate_mapping_trims_provider_text_boundaries bangumi_candidate_mapping_trims_provider_text_boundaries --no-fail-fast`: expected RED before implementation, then PASS 2. Proves TMDB and Bangumi candidate mapping trims provider text fields before writing metadata patch, facts, external IDs, alternate titles, genres/tags, and artwork URLs.
- `cargo nextest run -p nako-metadata-scraper http_runtime tmdb bangumi ranking title --no-fail-fast`: PASS 73. Proves provider text-boundary normalization composes with HTTP runtime behavior, TMDB/Bangumi direct lookup, search payload salvage, relevance-budget ranking, title variants, degraded candidates, and partial-search diagnostics.
