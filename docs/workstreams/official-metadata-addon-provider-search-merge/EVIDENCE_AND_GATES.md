# Official Metadata Addon Provider Search Merge — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast
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

- `docs/workstreams/official-metadata-addon-provider-search-merge/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-search-merge/TODO.md`
- `docs/workstreams/official-metadata-addon-provider-breadth`
- `crates/nako-metadata-scraper/src/engine/title.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## 2026-05-24 OMPSM-020

Review result:

- Workstream compliance: PASS with no blocking findings. Search merging stays inside TMDB and
  Bangumi providers; routes, browser-worker, Douban, and host task runtime were not expanded.
- Code quality: PASS with no blocking findings. The implementation keeps the existing provider
  `suggest` seam, dedupes provider IDs before detail enrichment, and keeps the existing three
  candidate enrichment budget explicit.
- Residual risk: provider-specific transliteration and smarter ranking between merged search
  variants remain follow-on work.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast`: PASS, 19 tests
  passed and 48 skipped. Proves TMDB and Bangumi merge non-empty search-title variant results,
  preserve fallback behavior, skip case-only duplicate search keys, and keep provider tests
  synthetic.

## 2026-05-24 OMPSM-030 Closeout

Review result:

- Workstream compliance: PASS with no blocking findings. OMPSM-010 and OMPSM-020 are complete, and
  remaining provider-search work is explicitly deferred.
- Code quality: PASS with no blocking findings. The code keeps the existing provider public seam,
  avoids a premature cross-provider abstraction over different raw result types, and leaves network
  policy in the shared HTTP runtime.
- Residual risk: merged candidates still rely on provider order before ranking. Smarter
  cross-variant ranking should be a separate ranking-focused lane.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast`: PASS, 19 tests
  passed and 48 skipped. Proves the search merge behavior and title variant helper.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 67 tests passed. Proves the
  package-level metadata scraper surface after search merge.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

## 2026-05-24 Final Release Audit

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast`: PASS, 22 tests
  passed and 48 skipped. Proves the title-variant merge behavior after degraded candidate tests were
  added.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 70 tests passed. Proves the
  package-level metadata scraper surface.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 70 tests passed. Proves the full workspace
  test suite remains green for the current release scope.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

## 2026-05-24 Trailing Qualifier Variant Hardening Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper search_title_variants_include_trailing_qualifier_stripped_forms --no-fail-fast`: expected RED before implementation, then PASS. Proves search-title variants now try trailing bracket/parenthesis qualifier-stripped forms, including consecutive qualifiers such as `The Matrix` from `The Matrix (1999) [1080p]`, before fully normalized noisy search keys.
- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`: PASS 44. Proves the new title variant ordering composes with TMDB/Bangumi search merge, ranking, direct lookup, payload resilience, and degraded fallback behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 87. Proves the package-level metadata scraper surface after title variant hardening.
- `cargo nextest run --workspace --no-fail-fast`: PASS 87. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

## 2026-05-24 Query Year Normalization Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_string_year metadata_query_parses_year_aliases metadata_query_parses_year_from_date_fields --no-fail-fast`: PASS 3. Proves metadata query parsing now accepts trimmed string years, `release_year`/`original_year` aliases, and date-derived years while rejecting non-year date text.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 106. Proves the package-level metadata scraper surface after shared query-year normalization, including TMDB `primary_release_year`, Bangumi `filter.air_date`, and ranking consumers of `MetadataQuery::year`.
- `cargo nextest run --workspace --no-fail-fast`: PASS 106. Proves the full workspace test suite remains green after query-year normalization and evidence updates.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the diff has no whitespace errors.

## 2026-05-24 Query Language Normalization Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper metadata_query_trims_language metadata_query_uses_default_language_when_payload_language_is_blank --no-fail-fast`: PASS 2. Proves metadata query parsing trims payload languages and falls back to the configured default when the payload language is blank.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 108. Proves the package-level metadata scraper surface after shared query-language normalization, including Bangumi title-language selection, ranking language evidence, and writeback tag consumers.
- `cargo nextest run --workspace --no-fail-fast`: PASS 108. Proves the full workspace test suite remains green after query-language normalization.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the diff has no whitespace errors.

## 2026-05-24 Query Title Field Normalization Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper metadata_query_uses_first_non_empty_title_field metadata_query_falls_back_to_original_title --no-fail-fast`: expected RED before implementation, then PASS 2. Proves metadata query parsing no longer lets blank `title` values hide usable `name` or `original_title` fields.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 110. Proves the package-level metadata scraper surface after shared query-title field normalization.
- `cargo nextest run --workspace --no-fail-fast`: PASS 110. Proves the full workspace test suite remains green after query-title field normalization.
- `cargo fmt --all -- --check`: PASS after formatting the helper call.
- `git diff --check`: PASS. Proves the diff has no whitespace errors.

## 2026-05-24 Query Year Boundary Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper metadata_query_ignores_non_positive_years tmdb_search_omits_primary_release_year_when_query_year_is_invalid bangumi_air_date_filter_ignores_non_positive_years --no-fail-fast`: PASS 3. Proves payload year parsing skips non-positive years, TMDB omits `primary_release_year` for invalid direct query years, and Bangumi omits invalid `air_date` filters.

## 2026-05-24 Query Year Range Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper metadata_query_ignores_out_of_range_years tmdb_search_omits_primary_release_year_when_query_year_is_invalid bangumi_air_date_filter_ignores_non_positive_years tmdb_release_year_ignores_zero_year_values bangumi_release_year_ignores_zero_year_values --no-fail-fast`: PASS 5. Proves query and provider date parsing reject out-of-range or overlong year prefixes instead of truncating them into plausible but wrong years.
