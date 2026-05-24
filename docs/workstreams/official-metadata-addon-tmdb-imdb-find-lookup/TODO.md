# Official Metadata Addon TMDB IMDb Find Lookup — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Lookup Policy

- [x] OMITF-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-tmdb-imdb-find-lookup]
  Goal: Freeze TMDB IMDb find lookup behavior, fallbacks, and gate set.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-tmdb-imdb-find-lookup/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is TMDB IMDb find lookup.

## M1 — TMDB IMDb Find Lookup

- [x] OMITF-020 [owner=codex] [deps=OMITF-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Use query IMDb IDs to resolve a TMDB movie through `/find/{imdb_id}` before title search.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_query_imdb_external_id_for_find_movie_lookup tmdb_provider_falls_back_to_search_when_query_imdb_find_is_empty tmdb_provider_uses_query_external_id_for_direct_movie_lookup --no-fail-fast
  Review: Native TMDB ID precedence remains intact; failed find requests fall back to search; no raw provider errors are exposed in payload.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Done on 2026-05-24. TMDB now resolves query IMDb IDs through `/find/{imdb_id}` before title search, reusing existing detail enrichment and preserving fallback behavior.

## M2 — Closeout

- [x] OMITF-030 [owner=planner] [deps=OMITF-020] [scope=docs/workstreams/official-metadata-addon-tmdb-imdb-find-lookup]
  Goal: Verify and close the lane or split follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: Confirm public payload shape and HTTP policy did not change.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md
  Handoff: Completed on 2026-05-24. Lane closed with targeted, package, workspace, format, and whitespace gates passing.

## M3 — Query IMDb Multi-ID Addendum

- [x] OMITF-040 [owner=codex] [deps=OMITF-030] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Keep trying later valid query IMDb IDs when an earlier TMDB `/find/{imdb_id}` request is empty or fails.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb_provider_uses_later_imdb_external_id_when_first_find_is_empty tmdb_provider_uses_later_imdb_external_id_when_first_find_fails --no-fail-fast
  Review: Native TMDB ID precedence remains intact; title search remains the fallback after all IMDb IDs are exhausted.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Done on 2026-05-24. TMDB IMDb find now follows the same multi-ID continuation policy as provider-native direct lookup.

## M4 — Query IMDb Case Normalization Addendum

- [x] OMITF-050 [owner=codex] [deps=OMITF-040] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Accept uppercase or mixed-case `tt` prefixes in query IMDb IDs and normalize the TMDB find path to lowercase `tt`.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb_provider_normalizes_query_imdb_external_id_case_for_find_lookup --no-fail-fast
  Review: Keep digit validation strict and preserve title-search fallback for malformed IDs.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Done on 2026-05-24. Query IMDb IDs now tolerate common case variation without changing response shape.

## M5 — Query IMDb Duplicate Request Addendum

- [x] OMITF-060 [owner=codex] [deps=OMITF-050] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Deduplicate repeated normalized query IMDb IDs before TMDB find lookup to avoid duplicate external requests.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb_provider_deduplicates_query_imdb_external_ids_before_find_lookup --no-fail-fast
  Review: Preserve first-seen order, later distinct IMDb ID continuation, and title-search fallback after all distinct IMDb IDs are exhausted.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Done on 2026-05-24. TMDB IMDb find now skips repeated normalized IMDb IDs before calling `/find/{imdb_id}`.
