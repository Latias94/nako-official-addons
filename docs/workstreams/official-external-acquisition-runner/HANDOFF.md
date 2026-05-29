# Official External Acquisition Runner - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

OEAR-020 is complete. The protocol now has a dedicated External Acquisition
Action task contract, and the official Nako catalog exposes an
`nako.official.external-acquisition-runner` descriptor without adding a real
runner adapter.

This lane starts contract/fixture-first. It should not begin with qBittorrent,
Transmission, aria2, or HTTP downloader code until the host-owned action
envelope and fixture runner semantics are stable.

## Active Task

- Task ID: OEAR-030
- Owner: codex
- Files: `crates/nako-external-acquisition-runner`,
  `addons/external-acquisition-runner`
- Validation: package tests for manifest, health, action envelope,
  idempotency, cancellation, status, and redaction
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

## Blockers

- None for OEAR-030.
- Product UI may depend on the separate web acquisition intake lane later.

## Next Recommended Action

Start OEAR-030 by creating the fixture/no-op runner sidecar that implements the
new action contract. It should accept only opaque target refs, preserve
idempotency, support cancellation/status transitions, and avoid external
downloader network calls.
