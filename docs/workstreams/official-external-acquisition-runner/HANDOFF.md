# Official External Acquisition Runner - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

OEAR-030 is complete. The protocol now has a dedicated External Acquisition
Action task contract, the official Nako catalog exposes an
`nako.official.external-acquisition-runner` descriptor, and this repository has
a fixture/no-op runner sidecar that implements manifest, health, action
submission, idempotent enqueue replay, status query, cancellation, progress,
safe diagnostics, and a local smoke script.

This lane starts contract/fixture-first. It should not begin with qBittorrent,
Transmission, aria2, or HTTP downloader code until the host-owned action
envelope and fixture runner semantics are stable.

## Active Task

- Task ID: OEAR-040
- Owner: codex
- Files: `../nako/crates/nako-server/src/app/addons`,
  `../nako/crates/nako-server/src/http`
- Validation: `nako-server` contract tests for action authorization,
  idempotency, redaction, cancellation, and retry-safe dispatch
- Status: READY

## Decisions Since Open

- External acquisition actions are separate from `resource_search`.
- Inputs must be host-owned opaque references, not browser-submitted raw URLs.
- Cloud-drive transfer remains out of scope.
- Admin UI route work remains separate from this backend/action contract lane.
- Begin with fixture/no-op runner behavior before choosing a production adapter.
- External acquisition actions are Addon Tasks, not Addon Resources.
- The action task uses `acquisition_action_run` and task input/output schema
  IDs so catalog and surfaces can advertise the contract.
- Protocol manifests now allow task-only sidecars as valid addon surfaces.
- The official sidecar uses an in-memory fixture runner only. It never calls
  downloader software or external network services.
- The sidecar rejects raw URL/password-shaped payloads through the protocol
  schema and does not echo unsafe payload facts in task errors or diagnostics.
- The checked-in addon manifest must match the runtime container manifest.

## Blockers

- None for OEAR-040.
- Product UI may depend on the separate web acquisition intake lane later.

## Next Recommended Action

Start OEAR-040 in `../nako` by wiring host dispatch to the approved action
sidecar. Keep authorization, idempotency, retry safety, cancellation, progress
refresh, audit records, and redaction as the primary test surface.
