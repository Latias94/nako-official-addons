# Official Metadata Addon Bangumi Year Air Date Filter — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Policy Freeze

Exit criteria:

- Year filter semantics are documented.
- The lane cites official Bangumi OpenAPI behavior and does not copy repo-ref implementation.

Evidence:

- `DESIGN.md`
- `TODO.md`
- `WORKSTREAM.json`

## M1 — Bangumi Filter Slice

Exit criteria:

- Query year maps to `filter.air_date` range.
- Yearless queries omit `air_date`.
- Existing `type` and `nsfw` filters remain present.

Evidence:

- `cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_query_year_as_air_date_search_filter bangumi_provider_omits_air_date_search_filter_when_query_year_is_missing --no-fail-fast`

## M2 — Closeout

Exit criteria:

- Targeted Bangumi/ranking/title gate passes.
- Package and workspace tests pass.
- Formatting and whitespace gates pass.

Evidence:

- `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`
