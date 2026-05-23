# Official Metadata Addon Provider Breadth and Localization — TODO

Status: Active
Last updated: 2026-05-23

## M0 — Scope And Evidence Freeze

- [x] OMPB-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-breadth]
  Goal: Freeze provider breadth problem, target state, non-goals, and evidence anchors.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json exist and agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-breadth/DESIGN.md
  Handoff: Completed on 2026-05-23. The follow-on lane is open.

## M1 — Artwork Selection Proof

- [x] OMPB-020 [owner=codex] [deps=OMPB-010] [scope=crates/nako-metadata-scraper/src/engine/artwork.rs,crates/nako-metadata-scraper/src/engine/mod.rs]
  Goal: Make artwork candidate selection choose the best poster or backdrop across all providers rather than the first matching candidate.
  Validation: cargo nextest run -p nako-metadata-scraper artwork --no-fail-fast
  Review: review-workstream before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/engine/artwork.rs
  Handoff: Completed on 2026-05-23. Provider-local artwork selection now prefers higher-confidence and higher-resolution candidates.

## M2 — Alias And Localization Proof

- [ ] OMPB-030 [owner=codex] [deps=OMPB-020] [scope=crates/nako-metadata-scraper/src/engine/ranking.rs,crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Deepen provider-local alias and localized title coverage so TMDB and Bangumi candidates keep matching when the surface title is not the best search key.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast
  Review: review-workstream for workstream compliance and code quality.
  Evidence: crates/nako-metadata-scraper/src/engine/ranking.rs
  Handoff: Split any broader provider breadth into a follow-on if needed.

## M3 — Closeout

- [ ] OMPB-040 [owner=planner] [deps=OMPB-030] [scope=docs/workstreams/official-metadata-addon-provider-breadth]
  Goal: Close the lane or create a narrower follow-on.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Summarize remaining risks in HANDOFF.md.
