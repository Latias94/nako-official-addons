# Official Metadata Addon Provider Partial Search Diagnostics — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Policy Freeze

Exit criteria:

- Diagnostic semantics are documented.
- The lane commits to `provider_note` instead of adding a new payload field.
- Raw provider error details remain out of payload scope.

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`

## M1 — TMDB Diagnostic Slice

Exit criteria:

- TMDB preserved candidates mention partial title-variant search failure.
- TMDB all-search-failed behavior still propagates a provider error.
- TMDB degraded and partial-enrichment notes remain meaningful.

Evidence:

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_preserves_search_results_when_later_title_variant_search_fails tmdb_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast`

## M2 — Bangumi Diagnostic Slice

Exit criteria:

- Bangumi preserved candidates mention partial title-variant search failure.
- Bangumi all-search-failed behavior still propagates a provider error.
- Bangumi degraded notes remain meaningful.

Evidence:

- `cargo nextest run -p nako-metadata-scraper bangumi_provider_preserves_search_results_when_later_title_variant_search_fails bangumi_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast`

## M3 — Closeout

Exit criteria:

- Targeted TMDB/Bangumi/ranking/title gate passes.
- Package and workspace tests pass.
- Formatting and whitespace gates pass.
- Workstream evidence records what each gate proves.

Evidence:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
