# Official Metadata Addon Provider Degraded Candidates — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Degraded Candidate Policy Freeze

- [x] OMPDC-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-degraded-candidates]
  Goal: Freeze degraded-candidate policy, non-goals, and evidence anchors.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-degraded-candidates/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is provider degraded candidates.

## M1 — Provider Degraded Candidate Proof

- [x] OMPDC-020 [owner=codex] [deps=OMPDC-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Return degraded TMDB/Bangumi candidates from search-result facts when detail enrichment fails after HTTP runtime policy is exhausted.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs, crates/nako-metadata-scraper/src/providers/bangumi.rs
  Result: DONE 2026-05-24.
  Handoff: TMDB and Bangumi both return degraded candidates with redaction-safe provider notes
  while preserving search-level failures as provider failures.

## M2 — Closeout

- [x] OMPDC-030 [owner=planner] [deps=OMPDC-020] [scope=docs/workstreams/official-metadata-addon-provider-degraded-candidates]
  Goal: Close the lane or split remaining degraded/error reporting work into follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Result: DONE 2026-05-24.
  Handoff: Lane complete. Live provider network checks, broader provider coverage, and richer
  operator-facing partial-result warnings remain separate follow-on scope.
