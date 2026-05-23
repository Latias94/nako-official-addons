# Official Metadata Addon Provider Search Merge — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Policy Freeze

- [x] OMPSM-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-search-merge]
  Goal: Freeze the search-merge problem, budget policy, non-goals, and evidence anchors.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-search-merge/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is provider search-result merging.

## M1 — Provider Search Merge Proof

- [x] OMPSM-020 [owner=codex] [deps=OMPSM-010] [scope=crates/nako-metadata-scraper/src/engine/title.rs,crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Merge non-empty TMDB and Bangumi search results across title variants while preserving a provider-local enrichment budget and provider ID deduplication.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/engine/title.rs, crates/nako-metadata-scraper/src/providers/tmdb.rs, crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Completed on 2026-05-24. TMDB and Bangumi now collect search-title variant results before enrichment, dedupe provider IDs, cap enrichment at three candidates, and skip case-only duplicate search keys.

## M2 — Closeout

- [x] OMPSM-030 [owner=planner] [deps=OMPSM-020] [scope=docs/workstreams/official-metadata-addon-provider-search-merge]
  Goal: Close the lane or split remaining localization/search work into follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Completed on 2026-05-24. Lane closed with package and hygiene gates passing; residual
  transliteration and merged-result ranking are deferred follow-ons.
