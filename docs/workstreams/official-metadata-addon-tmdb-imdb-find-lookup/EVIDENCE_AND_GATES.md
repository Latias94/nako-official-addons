# Official Metadata Addon TMDB IMDb Find Lookup — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup --no-fail-fast
```

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup tmdb_provider_falls_back_to_search_when_query_imdb_find_is_empty tmdb_provider_uses_query_external_id_for_direct_movie_lookup --no-fail-fast
cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast
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

- `docs/workstreams/official-metadata-addon-tmdb-imdb-find-lookup/DESIGN.md`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- Official TMDB developer documentation for `/3/find/{external_id}` with `external_source=imdb_id`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## Evidence Log

### 2026-05-24 — OMITF-010 Scope And Lookup Policy

- Workstream opened to resolve query IMDb IDs through TMDB's official `/find/{external_id}` API before title search.
- Native TMDB query ID precedence, title-search fallback, public payload shape, and HTTP runtime policy remain unchanged.

### 2026-05-24 — OMITF-020 TMDB IMDb Find Lookup

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup --no-fail-fast`: expected RED before implementation, then PASS 1. The RED failure proved the provider still attempted to parse a TMDB find response as a title-search response; the PASS proves query IMDb IDs now call `/find/{imdb_id}` and reuse existing detail enrichment.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup tmdb_provider_falls_back_to_search_when_query_imdb_find_is_empty tmdb_provider_uses_query_external_id_for_direct_movie_lookup --no-fail-fast`: PASS 3. Proves IMDb find lookup, empty-find fallback to title search, and native TMDB query ID precedence.

### 2026-05-24 — OMITF-030 Closeout

- `cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast`: PASS 36. Proves TMDB IMDb find lookup composes with existing TMDB provider behavior, ranking evidence, and title variants.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 91. Proves the package-level metadata scraper surface after TMDB IMDb find lookup.
- `cargo nextest run --workspace --no-fail-fast`: PASS 91. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable after rustfmt.
- `git diff --check`: PASS. Proves the diff has no whitespace errors before closeout documentation.

### 2026-05-24 — OMITF-040 Query IMDb Multi-ID Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_later_imdb_external_id_when_first_find_is_empty tmdb_provider_uses_later_imdb_external_id_when_first_find_fails --no-fail-fast`: expected RED before implementation, then PASS 2. Proves TMDB keeps trying later valid query IMDb IDs when an earlier find result is empty or the find request fails.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup tmdb_provider_falls_back_to_search_when_query_imdb_find_is_empty tmdb_provider_uses_query_external_id_for_direct_movie_lookup --no-fail-fast`: PASS 3. Proves the addendum preserves single-ID IMDb find, all-empty fallback to title search, and native TMDB ID precedence.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 97. Proves the package-level metadata scraper surface after TMDB IMDb multi-ID continuation.
- `cargo nextest run --workspace --no-fail-fast`: PASS 97. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting remains stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

### 2026-05-24 — OMITF-050 Query IMDb Case Normalization Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_normalizes_query_imdb_external_id_case_for_find_lookup --no-fail-fast`: expected RED before implementation, then PASS 1. Proves uppercase query IMDb IDs are accepted and normalized to lowercase `tt` for TMDB find lookup.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup tmdb_provider_uses_later_imdb_external_id_when_first_find_is_empty tmdb_provider_uses_later_imdb_external_id_when_first_find_fails tmdb_provider_falls_back_to_search_when_query_imdb_find_is_empty --no-fail-fast`: PASS 4. Proves case normalization preserves existing single-ID, multi-ID, and title-search fallback behavior.

### 2026-05-24 — OMITF-060 Query IMDb Duplicate Request Addendum

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_deduplicates_query_imdb_external_ids_before_find_lookup --no-fail-fast`: expected RED before implementation, then PASS 1. Proves repeated normalized query IMDb IDs are only requested once before later distinct IDs are tried.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_later_imdb_external_id_when_first_find_is_empty tmdb_provider_uses_later_imdb_external_id_when_first_find_fails --no-fail-fast`: PASS 2. Proves IMDb find dedupe preserves later distinct ID continuation after an earlier distinct find is empty or fails.
