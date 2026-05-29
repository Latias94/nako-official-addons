# Official Metadata Addon Provider Enrichment Resilience — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Failure Policy Freeze

- [x] OMPER-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-enrichment-resilience]
  Goal: Freeze candidate-level failure isolation policy, non-goals, and evidence anchors.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-enrichment-resilience/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is candidate enrichment failure isolation.

## M1 — Candidate Enrichment Failure Isolation

- [x] OMPER-020 [owner=codex] [deps=OMPER-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Isolate failed TMDB/Bangumi candidate enrichments after HTTP runtime policy is exhausted while returning other usable candidates.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb bangumi --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs, crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Completed on 2026-05-24. TMDB and Bangumi now isolate candidate-level enrichment failures
  and keep returning other usable candidates; search-level failures remain provider-level failures.
  The later degraded-candidates lane upgrades final behavior from skip-only to degraded candidates.

## M2 — Closeout

- [x] OMPER-030 [owner=planner] [deps=OMPER-020] [scope=docs/workstreams/official-metadata-addon-provider-enrichment-resilience]
  Goal: Close the lane or split remaining network resilience work into follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Completed on 2026-05-24. Lane closed with target, package, format, and whitespace gates
  passing; payload-visible partial warnings are deferred follow-on scope.
