# Official Metadata Addon Bulk Task Design - Design

Status: Complete
Last updated: 2026-05-24

## Problem

The official metadata Addon now supports explicit `metadata_write` and
`artwork_write` side effects, and Nako now owns the generic Addon Task runtime.
Bulk Metadata Scrape is larger than a single resource call, so the addon side
needs a bounded task endpoint and batch planner instead of a hidden worker.

## Target State

- Nako owns Addon Task scheduling, execution records, cancellation, retry, and
  diagnostics.
- The official metadata Addon declares `bulk-metadata-scrape` and serves the
  task path now that the host task runtime contract exists.
- Bulk scrape work uses existing suggestion, ranking, `metadata_write`, and
  `artwork_write` seams rather than direct filesystem, database, or storage
  mutation.
- The task contract is safe for large libraries: bounded batch size,
  idempotency, grant checks, provider rate limits, and redaction-safe task
  summaries.

## Current Host Assessment

- `nako-addon-protocol` has `AddonTaskDeclaration`, `AddonTaskRequest`, and
  `AddonTaskResponse`.
- Nako Admin registration validates task declarations and required scopes.
- Nako can build Addon routing plans with `AddonRoutingPlanTarget::AddonTaskJob`
  and `JobKind::AddonTask`, and direct task-path dispatch is available.
- The official Addon can now expose a real task path without a hidden worker.

Therefore the official Addon manifest can declare `bulk-metadata-scrape` and
the addon repository can implement the bounded batch planner behind it.

## Scope

- Declare the official addon task and keep the manifest/example manifest in
  sync.
- Implement the host-facing task endpoint and bounded batch planner.
- Reuse existing metadata/artwork side-effect APIs inside the batch planner.
- Keep task payloads, outputs, and progress summaries redaction-safe.

## Non-Goals

- Hidden background scheduling inside the Addon sidecar.
- Declaring a manifest task that Nako cannot execute end-to-end.
- Direct Nako database, filesystem, or media storage access from the Addon.
- Provider-specific crawler breadth such as Douban.

## Architecture Direction

The Addon should remain an Adapter that translates Nako task input into bounded
provider calls and explicit Addon Side Effects. Nako still owns the durable
task record and host-facing run state, while the Addon processes a bounded batch
of `/metadata`-style payloads and returns a redaction-safe summary. The Addon
may report progress and candidate summaries, but Nako remains the owner of
library mutation, artwork ingest, retries, cancellation, and operator-visible
state.

## Related Evidence

- `docs/workstreams/official-metadata-addon-side-effect-writer/`
- `../nako/docs/guides/ADDON_AUTHOR_GUIDE.md`
- `../nako/docs/api/HTTP_API.md`
- `../nako/docs/workstreams/addon-runtime-and-distribution/`
- `../nako/crates/nako-addon-protocol/src/lib.rs`
- `../nako/crates/nako-server/src/app/addons.rs`

## Closeout

Completed on 2026-05-24. The addon manifest now declares `bulk-metadata-scrape`, the
`/tasks/bulk-metadata-scrape` endpoint and bounded batch planner are implemented in this
repository, and the checked-in example manifest matches runtime output. Future bulk-task progress
semantics or diagnostics should open a fresh follow-on lane.
