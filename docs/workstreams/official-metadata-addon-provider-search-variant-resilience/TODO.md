# Official Metadata Addon Provider Search Variant Resilience — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Variant Failure Policy Freeze

- [x] OMPSVR-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-search-variant-resilience]
  Goal: Freeze the partial search-variant failure policy, non-goals, and evidence anchors.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-search-variant-resilience/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is TMDB search-variant resilience.

## M1 — TMDB Search Variant Resilience

- [x] OMPSVR-020 [owner=codex] [deps=OMPSVR-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Preserve usable TMDB search results when a later title-variant search fails, while still propagating all-search-failed provider errors.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Completed on 2026-05-24. TMDB now keeps earlier title-variant results when a later variant search fails, and still propagates provider errors when all variants fail.

## M2 — Bangumi Search Variant Resilience

- [x] OMPSVR-030 [owner=codex] [deps=OMPSVR-020] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Preserve usable Bangumi search results when a later title-variant search fails, while still propagating all-search-failed provider errors.
  Validation: cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Completed on 2026-05-24. Bangumi now keeps earlier title-variant results when a later variant search fails, and still propagates provider errors when all variants fail.

## M3 — Closeout

- [x] OMPSVR-040 [owner=planner] [deps=OMPSVR-030] [scope=docs/workstreams/official-metadata-addon-provider-search-variant-resilience]
  Goal: Close the lane or split remaining partial-provider-error work into follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Completed on 2026-05-24. Lane closed with targeted, package, workspace, format, and whitespace gates passing; payload-visible partial-search warnings remain follow-on scope.
