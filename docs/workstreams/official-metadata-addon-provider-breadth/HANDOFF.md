# Official Metadata Addon Provider Breadth and Localization — Handoff

Status: Complete
Last updated: 2026-05-23

## Current State

The follow-on lane is closed. The previous provider hardening lane is closed.
Artwork selection proof is complete. Provider-local alias and localized title
proof is complete. Search fallback proof is complete.

## Active Task

- Task ID: OMPB-040
- Owner: planner
- Files: `docs/workstreams/official-metadata-addon-provider-breadth`
- Validation: `verify-rust-workstream` and `review-workstream`
- Status: DONE
- Review: PASS, no blocking findings
- Evidence: `docs/workstreams/official-metadata-addon-provider-breadth/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Reference repositories are read-only inspiration only.
- CheckTMDB remains a reference for operator DNS/hosts workarounds, not code.
- Browser automation, Douban, and host task runtime stay outside this lane.
- Artwork selection now prefers the best matching candidate across providers.
- Ranking now consumes provider-neutral alternate title facts.
- TMDB maps official alternative titles into candidate facts.
- Bangumi maps localized names and title-like infobox aliases into candidate facts.
- TMDB and Bangumi retry a normalized search key only when the raw title search returns no
  candidates.

## Blockers

- None for OMPB-030.

## Next Recommended Action

- Open a new follow-on lane only when one of the deferred scopes becomes the active priority.

## Follow-ons

- broader provider localization
- alias and title breadth
- provider-specific transliteration and romanization
- non-empty multi-search result merging with request-budget and deduplication policy
- browser-worker and Douban remain separate lanes
