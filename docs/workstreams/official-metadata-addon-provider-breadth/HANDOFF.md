# Official Metadata Addon Provider Breadth and Localization — Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The follow-on lane is open and scoped. The previous provider hardening lane is
closed. Artwork selection proof is complete. Provider-local alias and localized
title proof is complete. This lane is ready for closeout planning.

## Active Task

- Task ID: OMPB-040
- Owner: planner
- Files: `docs/workstreams/official-metadata-addon-provider-breadth`
- Validation: `verify-rust-workstream` and `review-workstream`
- Status: READY
- Review: Pending
- Evidence: `docs/workstreams/official-metadata-addon-provider-breadth/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Reference repositories are read-only inspiration only.
- CheckTMDB remains a reference for operator DNS/hosts workarounds, not code.
- Browser automation, Douban, and host task runtime stay outside this lane.
- Artwork selection now prefers the best matching candidate across providers.
- Ranking now consumes provider-neutral alternate title facts.
- TMDB maps official alternative titles into candidate facts.
- Bangumi maps localized names and title-like infobox aliases into candidate facts.

## Blockers

- None for OMPB-030.
- `cargo fmt --all -- --check` is currently blocked by unrelated untracked
  `crates/nako-metadata-scraper/src/engine/title.rs` formatting outside OMPB-030.

## Next Recommended Action

- Review and close the lane with OMPB-040, or split broader localization/search-key work into a
  follow-on.

## Follow-ons

- broader provider localization
- alias and title breadth
- browser-worker and Douban remain separate lanes
