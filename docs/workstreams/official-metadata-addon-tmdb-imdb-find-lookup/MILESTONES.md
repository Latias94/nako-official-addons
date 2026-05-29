# Official Metadata Addon TMDB IMDb Find Lookup — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Policy Freeze

Exit criteria:

- Lookup, fallback, and non-goal semantics are documented.
- The lane explicitly uses official TMDB API behavior and does not copy repo-ref implementation.

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`

## M1 — TMDB IMDb Find Slice

Exit criteria:

- Query IMDb ID calls `/find/{imdb_id}` before title search.
- Find movie result reuses existing detail enrichment.
- Empty or failed find falls back to title search.
- Native TMDB query ID still wins over IMDb find.

Evidence:

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup tmdb_provider_falls_back_to_search_when_query_imdb_find_is_empty tmdb_provider_uses_query_external_id_for_direct_movie_lookup --no-fail-fast`

## M2 — Closeout

Exit criteria:

- Targeted TMDB/ranking/title gate passes.
- Package and workspace tests pass.
- Formatting and whitespace gates pass.

Evidence:

- `cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## M3 — Query IMDb Multi-ID Addendum

Exit criteria:

- Multiple valid query IMDb IDs are tried in payload order.
- Empty TMDB find results continue to later IMDb IDs before title search.
- Failed TMDB find requests continue to later IMDb IDs before title search.
- Native TMDB query ID still wins over IMDb find.
- Package, workspace, format, and whitespace gates pass.

Evidence:

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_later_imdb_external_id_when_first_find_is_empty tmdb_provider_uses_later_imdb_external_id_when_first_find_fails --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## M4 — Query IMDb Case Normalization Addendum

Exit criteria:

- Uppercase or mixed-case `tt` prefixes are accepted.
- The TMDB find path is normalized to lowercase `tt`.
- Non-digit IMDb suffixes remain invalid.
- Package, workspace, format, and whitespace gates pass.

Evidence:

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_normalizes_query_imdb_external_id_case_for_find_lookup --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## M5 — Query IMDb Duplicate Request Addendum

Exit criteria:

- Repeated normalized query IMDb IDs are requested at most once.
- Later distinct IMDb IDs are still tried after an earlier distinct ID is empty or fails.
- Title search remains the fallback after all distinct IMDb IDs are exhausted.
- Package, workspace, format, and whitespace gates pass.

Evidence:

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_deduplicates_query_imdb_external_ids_before_find_lookup --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
