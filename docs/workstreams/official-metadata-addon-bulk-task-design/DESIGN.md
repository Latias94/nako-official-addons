# Official Metadata Addon Bulk Task Design - Design

Status: Complete
Last updated: 2026-05-24

## Problem

The official metadata Addon now supports explicit `metadata_write` and
`artwork_write` side effects, but Bulk Metadata Scrape is larger than a single
resource call. It needs host-owned scheduling, cancellation, retries, progress,
and result accountability.

Nako currently validates Addon Task declarations and can build routing plans
for them, but the generic Addon Task scheduler/invoker is still deferred. The
official Addon must not fill that gap with a hidden sidecar background worker.

## Target State

- Nako owns Addon Task scheduling, execution records, cancellation, retry, and
  diagnostics.
- The official metadata Addon declares `bulk-metadata-scrape` only after the
  host task runtime contract is available.
- Bulk scrape work uses existing suggestion, ranking, `metadata_write`, and
  `artwork_write` seams rather than direct filesystem, database, or storage
  mutation.
- The task contract is safe for large libraries: bounded batch size,
  idempotency, grant checks, provider rate limits, and redaction-safe progress.

## Current Host Assessment

- `nako-addon-protocol` has `AddonTaskDeclaration`.
- Nako Admin registration validates task declarations and required scopes.
- Nako can build Addon routing plans with `AddonRoutingPlanTarget::AddonTaskJob`
  and `JobKind::AddonTask`.
- Nako docs still state that full Addon Task scheduler/runtime breadth is
  deferred.
- There is no generic Admin or Addon runtime endpoint that invokes an Addon
  Task path and records task progress/outcome.

Therefore the official Addon manifest must keep `tasks: []` until the host
runtime seam exists.

## Scope

- Preserve the official addon manifest as task-free until host execution exists.
- Define the host prerequisites for `bulk-metadata-scrape`.
- Define the future task request/response semantics and side-effect workflow.
- Add implementation tasks only after the host contract is owned by Nako.

## Non-Goals

- Hidden background scheduling inside the Addon sidecar.
- Declaring a manifest task that Nako cannot execute end-to-end.
- Direct Nako database, filesystem, or media storage access from the Addon.
- Provider-specific crawler breadth such as Douban.

## Architecture Direction

The Addon should remain an Adapter that translates Nako task input into bounded
provider calls and explicit Addon Side Effects. Nako must own the durable task
record and call the Addon task endpoint when it is ready. The Addon may report
progress and candidate summaries, but Nako remains the owner of library
mutation, artwork ingest, retries, cancellation, and operator-visible state.

## Related Evidence

- `docs/workstreams/official-metadata-addon-side-effect-writer/`
- `../nako/docs/guides/ADDON_AUTHOR_GUIDE.md`
- `../nako/docs/api/HTTP_API.md`
- `../nako/docs/workstreams/addon-runtime-and-distribution/`
- `../nako/crates/nako-addon-protocol/src/lib.rs`
- `../nako/crates/nako-server/src/app/addons.rs`

## Closeout Summary

Closed for the current official metadata addon release on 2026-05-24. The addon-side design is
complete: `bulk-metadata-scrape` must remain undeclared until Nako owns a generic Addon Task
scheduler/invoker, durable task records, cancellation, retry, progress, and redaction-safe outcome
reporting. Implementation of the host runtime belongs in `../nako`; this repository intentionally
keeps the official addon manifest task-free.
