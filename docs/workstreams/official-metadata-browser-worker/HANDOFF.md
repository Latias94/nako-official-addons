# Official Metadata Browser Worker - Handoff

Status: Active
Last updated: 2026-05-23

## Current State

This lane has been opened for a dedicated browser worker that will support
Douban and similar anti-bot metadata sources. The public metadata addon remains
the only Nako-facing addon surface.

## Active Task

- Task ID: OMBW-010
- Owner: planner
- Files: `docs/workstreams/official-metadata-browser-worker/*`
- Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json exist and agree.
- Status: NEEDS_CONTEXT
- Review: Pending
- Evidence: `docs/workstreams/official-metadata-browser-worker/DESIGN.md`

## Decisions Since Last Update

- The browser automation lane is separate from the existing side-effect writer lane.
- The browser worker should be an internal companion service, not a public addon requirement.
- Docker Compose is the expected local/self-hosted deployment mechanism.

## Blockers

- None yet.

## Next Recommended Action

- Implement OMBW-020: scaffold the worker service and prove one local rendered-page extraction path.
