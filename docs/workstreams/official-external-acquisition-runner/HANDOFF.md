# Official External Acquisition Runner - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

OEAR-040 is complete. The protocol now has a dedicated External Acquisition
Action task contract, the official Nako catalog exposes an
`nako.official.external-acquisition-runner` descriptor, and this repository has
a fixture/no-op runner sidecar that implements manifest, health, action
submission, idempotent enqueue replay, status query, cancellation, progress,
safe diagnostics, and a local smoke script. Nako core now guards dispatch of
the external acquisition action task before job input is persisted.

This lane starts contract/fixture-first. It should not begin with qBittorrent,
Transmission, aria2, or HTTP downloader code until the host-owned action
envelope and fixture runner semantics are stable.

## Active Task

- Task ID: OEAR-050
- Owner: planner
- Files: `docs/workstreams/official-external-acquisition-runner`
- Validation: decision note and adapter-specific follow-on task or lane
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
- Nako host dispatch treats the external acquisition action as a typed
  contract, not arbitrary task JSON: it requires direct dispatch, rejects
  unknown raw URL/password-shaped payloads before storage, aligns host and
  runner idempotency keys, requires audit refs for mutating operations, and
  maps runner `rejected`/`failed`/`not_found` responses to failed host task
  records with safe error codes.

## Blockers

- None for OEAR-050.
- Product UI may depend on the separate web acquisition intake lane later.
- `cargo clippy -p nako-server --tests -- -D warnings` is blocked by existing
  unrelated lint debt and was not used as a completion gate for OEAR-040.

## Next Recommended Action

Start OEAR-050 by deciding the first real runner adapter. Compare qBittorrent,
Transmission, aria2, and a plain HTTP downloader against config surface,
secret handling, cancellation/status APIs, testability, and deployment risk.
