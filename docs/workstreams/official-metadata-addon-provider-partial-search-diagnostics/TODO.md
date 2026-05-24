# Official Metadata Addon Provider Partial Search Diagnostics — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Diagnostic Policy

- [x] OMPSD-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-partial-search-diagnostics]
  Goal: Freeze redaction-safe partial-search diagnostic policy and gate set.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-partial-search-diagnostics/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is TMDB partial-search provider notes.

## M1 — TMDB Partial Search Provider Notes

- [x] OMPSD-020 [owner=codex] [deps=OMPSD-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Surface a safe TMDB provider note when a later title-variant search fails but earlier search results still produce candidates.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb_provider_preserves_search_results_when_later_title_variant_search_fails tmdb_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast
  Review: Note must not include raw provider error bodies, URLs, tokens, or query text.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Done on 2026-05-24. TMDB preserved candidates now compose a safe partial title-variant search failure note with existing provider notes.

## M2 — Bangumi Partial Search Provider Notes

- [x] OMPSD-030 [owner=codex] [deps=OMPSD-020] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Surface a safe Bangumi provider note when a later title-variant search fails but earlier search results still produce candidates.
  Validation: cargo nextest run -p nako-metadata-scraper bangumi_provider_preserves_search_results_when_later_title_variant_search_fails bangumi_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast
  Review: Note must not include raw provider error bodies, URLs, tokens, or query text.
  Evidence: crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Done on 2026-05-24. Bangumi preserved candidates now compose a safe partial title-variant search failure note with existing provider notes.

## M3 — Closeout

- [x] OMPSD-040 [owner=planner] [deps=OMPSD-030] [scope=docs/workstreams/official-metadata-addon-provider-partial-search-diagnostics]
  Goal: Verify and close the lane or split follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: Confirm payload shape and retry policy did not change.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md
  Handoff: Completed on 2026-05-24. Lane closed with targeted, package, workspace, format, and whitespace gates passing.
