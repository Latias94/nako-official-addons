# Official Metadata Addon Provider Relevance Budget — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Budget Policy Freeze

- [x] OMPRB-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-relevance-budget]
  Goal: Freeze the provider relevance-budget problem, non-goals, and evidence anchors.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-relevance-budget/DESIGN.md
  Handoff: Opened on 2026-05-24. First executable task is TMDB relevance-budget proof.

## M1 — TMDB Relevance-Budget Proof

- [x] OMPRB-020 [owner=codex] [deps=OMPRB-010] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs]
  Goal: Select the TMDB detail-enrichment budget from all deduped merged search results by relevance instead of first-seen provider order.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Completed on 2026-05-24. TMDB now collects all deduped merged search results, ranks search-result candidates with provider-neutral facts, and enriches the strongest three.

## M2 — Bangumi Relevance-Budget Proof

- [x] OMPRB-030 [owner=codex] [deps=OMPRB-020] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Select the Bangumi detail-enrichment budget from all deduped merged search results by relevance instead of first-seen provider order.
  Validation: cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast
  Review: review-workstream for scope and code quality before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/bangumi.rs
  Handoff: Completed on 2026-05-24. Bangumi now collects all deduped merged search results, ranks search-result candidates with provider-neutral facts, and enriches the strongest three.

## M3 — Closeout

- [x] OMPRB-040 [owner=planner] [deps=OMPRB-030] [scope=docs/workstreams/official-metadata-addon-provider-relevance-budget]
  Goal: Close the lane or split remaining provider relevance work into follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Completed on 2026-05-24. Lane closed with targeted, package, workspace, format, and whitespace gates passing; live provider drift checks remain follow-on scope.
