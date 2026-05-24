# Official Metadata Addon Bulk Task Design - Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The official metadata addon now declares `bulk-metadata-scrape` and serves
`/tasks/bulk-metadata-scrape`. The task endpoint uses a bounded batch planner,
crate-local task request/response DTOs that mirror the host-owned task
contract, and the same explicit metadata payload shape used by `POST
/metadata`. The checked-in manifest example matches runtime output, and fresh
manifest/bulk/workspace/clippy gates passed in this workspace.

## Completed Tasks

- OMAB-060: addon-side bulk manifest declaration, task endpoint, and bounded
  batch planner.
- OMAB-070: docs, verification, and closeout finalization.

## Decisions Since Last Update

- Nako owns task scheduling, execution records, retry, cancellation, and
  diagnostics.
- The addon sidecar owns only the bounded batch planner and request
  translation.
- Task envelopes are implemented locally in this crate to mirror the
  host-owned contract.
- Bulk items reuse the explicit `metadata_write` and `artwork_write`
  side-effect paths.
- Batch size is clamped to a bounded minimum and maximum.
- Redaction safety still applies to payloads, outputs, and diagnostics.

## Blockers

- None.

## Next Recommended Action

- None. The lane is closed. Open a follow-on lane only if task-progress
  diagnostics, partial result warnings, or new bulk semantics become a separate
  priority.
