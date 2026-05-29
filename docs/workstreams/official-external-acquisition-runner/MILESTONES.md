# Official External Acquisition Runner - Milestones

Status: Active
Last updated: 2026-05-29

## M0 - Scope And Contract Freeze

Exit criteria:

- Workstream docs exist and agree.
- ADR/follow-on contract dependencies are recorded.
- Non-goals prevent resource-search, cloud-drive, and Addon Manager drift.

## M1 - Host And Protocol Action Contract

Status: Complete via OEAR-020.

Exit criteria:

- Dedicated acquisition action scope and schemas exist.
- Official catalog descriptor exposes the new action surface.
- Tests prove the action envelope consumes host-owned opaque references.

## M2 - Fixture Runner Sidecar

Status: Complete via OEAR-030.

Exit criteria:

- Fixture/no-op runner sidecar validates manifest, health, action submission,
  status query, cancellation, idempotency, and redaction.
- No external runner network calls are introduced.

## M3 - Host Dispatch And Audit

Status: Complete via OEAR-040.

Exit criteria:

- Nako can dispatch approved actions to the sidecar.
- Idempotency, retry-safety, cancellation, status refresh, and audit are
  test-covered.
- Browser clients never submit raw runner payloads.

## M4 - Real Adapter Decision

Status: Complete via OEAR-050.

Exit criteria:

- First production adapter choice is explicit.
- Adapter work is either implemented as a bounded task or split into its own
  lane.

## M5 - Closeout

Exit criteria:

- Fresh focused gates are recorded.
- Residual risks and follow-ons are explicit.
- Dirty worktree constraints are respected.
