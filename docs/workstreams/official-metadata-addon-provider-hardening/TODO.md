# Official Metadata Addon Provider Hardening — TODO

Status: Complete
Last updated: 2026-05-23

## M0 — Scope And Evidence Freeze

- [x] OMPH-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-hardening]
  Goal: Freeze problem, target state, non-goals, and evidence anchors.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json exist and agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-hardening/DESIGN.md
  Handoff: Planner owns this before workers start.

## M1 — Network Policy Proof

- [x] OMPH-020 [owner=codex] [deps=OMPH-010] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/providers/http_runtime.rs,crates/nako-metadata-scraper/src/routes.rs]
  Goal: Surface provider network policy and diagnostics as a first-class addon seam, including proxy-aware behavior and safe failure reporting.
  Validation: cargo nextest run -p nako-metadata-scraper provider_http_runtime config routes --no-fail-fast
  Review: review-workstream before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/http_runtime.rs
  Handoff: Completed on 2026-05-23. Keep route handlers thin and do not duplicate transport logic outside the runtime.

## M2 — Provider Quality Proof

- [x] OMPH-030 [owner=codex] [deps=OMPH-020] [scope=crates/nako-metadata-scraper/src/engine/ranking.rs,crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Teach ranking to use original/sort title variants so TMDB and Bangumi candidates are not penalized when the primary surface title differs.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast
  Review: review-workstream for workstream compliance and code quality.
  Evidence: crates/nako-metadata-scraper/src/providers/tmdb.rs
  Handoff: Completed on 2026-05-23. Split any broader provider-quality breadth into a follow-on if needed.

## M3 — Closeout

- [x] OMPH-040 [owner=planner] [deps=OMPH-030] [scope=docs/workstreams/official-metadata-addon-provider-hardening]
  Goal: Close the lane or split the remaining work into a narrower follow-on.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Completed on 2026-05-23. Remaining breadth work is split into follow-on lanes.
