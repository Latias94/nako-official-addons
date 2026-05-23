# Official Metadata Addon Provider Breadth and Localization — Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The follow-on lane is open and scoped. The previous provider hardening lane is
closed. Artwork selection proof is complete. This lane will next focus on
provider-local alias and localized title coverage.

## Active Task

- Task ID: OMPB-030
- Owner: planner
- Files: `docs/workstreams/official-metadata-addon-provider-breadth`
- Validation: `verify-rust-workstream` and `review-workstream`
- Status: NEEDS_CONTEXT
- Review: Pending
- Evidence: `docs/workstreams/official-metadata-addon-provider-breadth/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Reference repositories are read-only inspiration only.
- CheckTMDB remains a reference for operator DNS/hosts workarounds, not code.
- Browser automation, Douban, and host task runtime stay outside this lane.
- Artwork selection now prefers the best matching candidate across providers.
- Provider-local ranking will deepen alias and localization next.

## Blockers

- None.

## Next Recommended Action

- Start the alias and localization proof next.

## Follow-ons

- broader provider localization
- alias and title breadth
- browser-worker and Douban remain separate lanes
