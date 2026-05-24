# Official Metadata Addon Bangumi Year Air Date Filter — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Filter Policy

- [x] OMBYF-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-bangumi-year-air-date-filter]
  Goal: Freeze Bangumi query-year to air-date filter behavior and gate set.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-bangumi-year-air-date-filter/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is Bangumi air-date filtering.

## M1 — Bangumi Air-Date Filter

- [x] OMBYF-020 [owner=codex] [deps=OMBYF-010] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Add query-year `filter.air_date` constraints to Bangumi subject search requests.
  Validation: cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_query_year_as_air_date_search_filter bangumi_provider_omits_air_date_search_filter_when_query_year_is_missing --no-fail-fast
  Review: Preserve subject type and NSFW filters, title-variant behavior, and no-query-year behavior.
  Evidence: crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Done on 2026-05-24. Bangumi search now maps query year to official `filter.air_date` constraints while omitting the field when year is absent.

## M2 — Closeout

- [x] OMBYF-030 [owner=planner] [deps=OMBYF-020] [scope=docs/workstreams/official-metadata-addon-bangumi-year-air-date-filter]
  Goal: Verify and close the lane or split follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: Confirm public payload shape and HTTP policy did not change.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md
  Handoff: Completed on 2026-05-24. Lane closed with targeted, package, workspace, format, and whitespace gates passing.
