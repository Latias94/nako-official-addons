# Official Metadata Addon Provider External ID Lookup — Evidence And Gates

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

- `docs/workstreams/official-metadata-addon-provider-external-id-lookup/DESIGN.md`
- `docs/workstreams/official-metadata-addon-result-quality`
- `crates/nako-metadata-scraper/src/engine/mod.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## Evidence Log

### 2026-05-24 — OMPEIL-020 TMDB Direct Lookup

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_external_id_for_direct_movie_lookup --no-fail-fast`: PASS. Proves a query-native `tmdb` external ID fetches `movie/{id}`, `external_ids`, and `alternative_titles` without issuing `search/movie`.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_external_id_for_direct_movie_lookup tmdb_provider_falls_back_to_search_when_query_external_id_is_invalid tmdb_provider_falls_back_to_search_when_direct_movie_lookup_fails --no-fail-fast`: PASS. Proves direct lookup, invalid-ID fallback, and failed-direct-lookup fallback behavior.
- `cargo nextest run -p nako-metadata-scraper tmdb ranking --no-fail-fast`: PASS 23. Proves the TMDB direct lookup slice remains compatible with ranking and existing TMDB provider behavior.

### 2026-05-24 — OMPEIL-030 Bangumi Direct Lookup

- `cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_query_external_id_for_direct_subject_lookup --no-fail-fast`: expected RED before implementation, then PASS after implementation. Proves a query-native `bangumi` external ID fetches `v0/subjects/{id}` without issuing `v0/search/subjects`.
- `cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_query_external_id_for_direct_subject_lookup bangumi_provider_falls_back_to_search_when_query_external_id_is_invalid bangumi_provider_falls_back_to_search_when_direct_subject_lookup_fails --no-fail-fast`: PASS. Proves direct lookup, invalid-ID fallback, and failed-direct-lookup fallback behavior.
- `cargo nextest run -p nako-metadata-scraper bangumi ranking --no-fail-fast`: PASS 22. Proves the Bangumi direct lookup slice remains compatible with ranking and existing Bangumi provider behavior.
- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`: PASS 36. Proves TMDB and Bangumi direct lookup changes compose with provider ranking, relevance-budget, and search-variant resilience behavior.

### 2026-05-24 — OMPEIL-040 Closeout

Review result:

- Workstream compliance: PASS with no blocking findings. OMPEIL-010 through OMPEIL-030 are complete,
  and cross-provider ID mapping remains explicitly out of scope.
- Code quality: PASS with no blocking findings. Direct lookup is provider-local, uses existing
  `suggest` and candidate mapping paths, and preserves title-search fallback behavior.
- Residual risk: direct lookup only recognizes provider-native numeric IDs; cross-provider mapping
  and live provider drift checks remain follow-on scope.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`: PASS 36. Proves both provider direct lookup slices compose with ranking, relevance-budget, and search-variant resilience behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 82. Proves the package-level metadata scraper surface after direct lookup closeout.
- `cargo nextest run --workspace --no-fail-fast`: PASS 82. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

### 2026-05-24 — Direct Lookup Duplicate-ID Hardening Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_later_valid_query_external_id_when_first_is_invalid --no-fail-fast`: expected RED before implementation, then PASS. Proves TMDB direct lookup scans all query-native `tmdb` external IDs and uses a later valid ID when an earlier same-provider value is malformed.
- `cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_later_valid_query_external_id_when_first_is_invalid --no-fail-fast`: expected RED before implementation, then PASS. Proves Bangumi direct lookup scans all query-native `bangumi` external IDs and uses a later valid ID when an earlier same-provider value is malformed.
- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`: PASS 40. Proves duplicate-ID hardening composes with ranking, direct lookup, search payload resilience, search-variant resilience, relevance-budget selection, and degraded fallback behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 86. Proves the package-level metadata scraper surface after duplicate-ID hardening.
- `cargo nextest run --workspace --no-fail-fast`: PASS 86. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

### 2026-05-24 — Query Payload Compatibility Addendum

- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_external_id_object_arrays --no-fail-fast`: expected RED before implementation, then PASS. Proves object-form `external_ids` can carry arrays of same-provider string IDs instead of dropping them.
- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_external_id_object_arrays ranking_evidence_metadata_query_parses_external_ids --no-fail-fast`: PASS 2. Proves object-value array parsing coexists with the existing object string parsing behavior.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_later_valid_query_external_id_when_first_lookup_fails bangumi_provider_uses_later_valid_query_external_id_when_first_lookup_fails --no-fail-fast`: expected RED before implementation, then PASS. Proves TMDB and Bangumi keep trying later same-provider IDs when an earlier valid direct lookup fails.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_external_id_for_direct_movie_lookup tmdb_provider_uses_later_valid_query_external_id_when_first_is_invalid tmdb_provider_falls_back_to_search_when_direct_movie_lookup_fails bangumi_provider_uses_query_external_id_for_direct_subject_lookup bangumi_provider_uses_later_valid_query_external_id_when_first_is_invalid bangumi_provider_falls_back_to_search_when_direct_subject_lookup_fails --no-fail-fast`: PASS 6. Proves the new multi-ID failure path preserves single direct lookup, invalid-ID continuation, and fallback-to-search behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 95. Proves the package-level metadata scraper surface after query payload compatibility hardening.
- `cargo nextest run --workspace --no-fail-fast`: PASS 95. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting remains stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

### 2026-05-24 — Direct Lookup Duplicate Request Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_deduplicates_query_external_ids_before_direct_lookup bangumi_provider_deduplicates_query_external_ids_before_direct_lookup --no-fail-fast`: expected RED before implementation, then PASS 2. Proves repeated provider-native query IDs are only requested once before later distinct IDs are tried.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_later_valid_query_external_id_when_first_lookup_fails bangumi_provider_uses_later_valid_query_external_id_when_first_lookup_fails --no-fail-fast`: PASS 2. Proves direct lookup dedupe preserves later distinct ID continuation after an earlier direct lookup fails.

### 2026-05-24 — Query Payload Array Object Alias Addendum

- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_external_id_array_object_value_aliases --no-fail-fast`: expected RED before implementation, then PASS 1. Proves array-of-object `external_ids` accepts `value`, `id`, and `external_id` string fields.
- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_external_id_array_object_value_aliases ranking_evidence_metadata_query_parses_external_ids metadata_query_parses_external_id_object_arrays --no-fail-fast`: PASS 3. Proves alias parsing preserves existing object-form string and object-form array parsing.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_external_id_for_direct_movie_lookup bangumi_provider_uses_query_external_id_for_direct_subject_lookup tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup --no-fail-fast`: PASS 3. Proves the parsed IDs still reach TMDB, Bangumi, and TMDB IMDb direct lookup paths.

### 2026-05-24 — Query Payload External ID Trim Addendum

- `cargo nextest run -p nako-metadata-scraper metadata_query_trims_external_ids_and_skips_empty_entries --no-fail-fast`: expected RED before implementation, then PASS 1. Proves parsed query external ID providers and values are trimmed and empty entries are skipped.
- `cargo nextest run -p nako-metadata-scraper metadata_query_trims_external_ids_and_skips_empty_entries metadata_query_parses_external_id_array_object_value_aliases ranking_evidence_metadata_query_parses_external_ids metadata_query_parses_external_id_object_arrays --no-fail-fast`: PASS 4. Proves trim normalization preserves existing object, object-array, and array-of-object parsing behavior.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_external_id_for_direct_movie_lookup bangumi_provider_uses_query_external_id_for_direct_subject_lookup tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup --no-fail-fast`: PASS 3. Proves normalized IDs still reach TMDB, Bangumi, and TMDB IMDb direct lookup paths.

### 2026-05-24 — Query Payload Top-Level External ID Alias Addendum

- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_top_level_external_id_aliases metadata_query_preserves_external_ids_before_top_level_aliases --no-fail-fast`: expected RED before implementation, then PASS 2. Proves top-level `tmdb_id`, `imdb_id`, and `bangumi_id` aliases parse into query external IDs after explicit `external_ids`.
- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_top_level_external_id_aliases tmdb_provider_uses_query_external_id_for_direct_movie_lookup bangumi_provider_uses_query_external_id_for_direct_subject_lookup tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup --no-fail-fast`: PASS 4. Proves top-level aliases feed the same TMDB direct lookup, Bangumi direct lookup, and TMDB IMDb find paths as explicit query external IDs.

### 2026-05-24 — Query Payload Numeric External ID Addendum

- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_numeric_top_level_external_id_aliases metadata_query_parses_numeric_external_id_values --no-fail-fast`: expected RED before implementation, then PASS 2. Proves integer JSON values are accepted for top-level ID aliases, object-form `external_ids`, and object-form external ID arrays.
- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_numeric_top_level_external_id_aliases metadata_query_parses_numeric_external_id_values tmdb_provider_uses_query_external_id_for_direct_movie_lookup bangumi_provider_uses_query_external_id_for_direct_subject_lookup --no-fail-fast`: PASS 4. Proves numeric ID parsing composes with the existing TMDB and Bangumi direct lookup paths.
- `cargo nextest run -p nako-metadata-scraper metadata_query_parses_external_id_array_object_value_aliases metadata_query_parses_numeric_top_level_external_id_aliases metadata_query_parses_numeric_external_id_values --no-fail-fast`: PASS 3. Proves array-of-object non-string values remain ignored while provider-keyed numeric IDs are accepted.

### 2026-05-24 — Direct Lookup Zero-ID Guard Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_query_movie_ids_ignores_zero_and_invalid_values tmdb_find_response_ignores_zero_movie_ids bangumi_query_subject_ids_ignores_zero_and_invalid_values --no-fail-fast`: PASS 3. Proves TMDB and Bangumi skip zero-valued direct lookup IDs, continue to later positive IDs, preserve duplicate suppression, and ignore zero movie IDs returned by TMDB IMDb find responses.

### 2026-05-24 — Query Payload Non-Positive External ID Addendum

- `cargo nextest run -p nako-metadata-scraper metadata_query_skips_non_positive_numeric_external_ids metadata_query_parses_numeric_external_id_values metadata_query_parses_numeric_top_level_external_id_aliases --no-fail-fast`: PASS 3. Proves query payload parsing skips non-positive numeric `tmdb`, `bangumi`, and `imdb` external IDs while preserving positive numeric object-form and top-level alias parsing.
