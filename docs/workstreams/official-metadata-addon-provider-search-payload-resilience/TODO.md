# Official Metadata Addon Provider Search Payload Resilience — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Salvage Policy Freeze

- [x] OMPSP-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-search-payload-resilience]
  Goal: Freeze tolerant search-item parsing behavior, non-goals, and evidence anchors.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-search-payload-resilience/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is TMDB search payload resilience.

## M1 — TMDB Search Payload Resilience

- [x] OMPSP-020 [owner=codex] [deps=OMPSP-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Skip malformed individual TMDB search result items while preserving valid results from the same response.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb ranking --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Done on 2026-05-24. TMDB search response parsing now skips malformed individual result items while preserving valid results. Gate passed.

## M2 — Bangumi Search Payload Resilience

- [x] OMPSP-030 [owner=codex] [deps=OMPSP-020] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Skip malformed individual Bangumi search subject items while preserving valid subjects from the same response.
  Validation: cargo nextest run -p nako-metadata-scraper bangumi ranking --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Done on 2026-05-24. Bangumi search response parsing now skips malformed individual subject items while preserving valid subjects. Gate passed.

## M3 — Closeout

- [x] OMPSP-040 [owner=planner] [deps=OMPSP-030] [scope=docs/workstreams/official-metadata-addon-provider-search-payload-resilience]
  Goal: Close the lane or split remaining live-provider drift checks into follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Completed on 2026-05-24. Lane closed with targeted, package, workspace, format, and whitespace gates passing; live provider drift checks remain follow-on scope.
