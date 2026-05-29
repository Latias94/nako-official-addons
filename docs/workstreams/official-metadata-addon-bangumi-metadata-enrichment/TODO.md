# Official Metadata Addon Bangumi Metadata Enrichment — TODO

Status: Complete
Last updated: 2026-05-26

## M0 — Scope And Evidence Freeze

- [x] OMBME-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment]
  Goal: Freeze problem, target state, non-goals, and evidence anchors.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json exist and agree.
  Evidence: docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment/DESIGN.md
  Handoff: Planner owns this before workers start.

## M1 — Subject Fact Enrichment

- [x] OMBME-020 [owner=codex] [deps=OMBME-010] [scope=crates/nako-metadata-scraper/src/providers/bangumi/{parser.rs,mapper.rs},crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Parse and map official Bangumi subject facts (`nsfw`, `locked`, `volumes`, `air_weekday`) plus selected infobox facts into deterministic provider tags and patch fields where protocol-compatible.
  Validation: cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast
  Review: review-workstream before accepting completion.
  Evidence: crates/nako-metadata-scraper/src/providers/bangumi.rs tests
  Handoff: Final status must be DONE, DONE_WITH_CONCERNS, BLOCKED, or NEEDS_CONTEXT.

## M2 — Reference Findings And Docs

- [x] OMBME-030 [owner=codex] [deps=OMBME-020] [scope=docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment,crates/nako-metadata-scraper/README.md,addons/metadata-scraper/README.md]
  Goal: Record reference findings, license boundaries, and visible Bangumi enrichment behavior.
  Validation: cargo fmt --all -- --check; cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast
  Review: review-workstream for workstream compliance and code quality.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Split follow-on work if scope expands.

## M3 — Closeout

- [x] OMBME-040 [owner=planner] [deps=OMBME-030] [scope=docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment]
  Goal: Close the lane or create a narrower follow-on.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Summarize remaining risks in HANDOFF.md.
