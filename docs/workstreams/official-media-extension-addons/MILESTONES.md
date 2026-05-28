# Official Media Extension Addons - Milestones

Status: Active
Last updated: 2026-05-28

## M0 - Scope Freeze

Exit criteria:

- Workstream docs exist and agree.
- Subtitle Provider is read-only.
- DLNA Renderer is plan-only.
- External Acquisition Runner is a follow-on, not implementation scope.

## M1 - Subtitle Provider Foundation

Exit criteria:

- `nako-subtitle-provider` builds as a workspace package.
- Manifest declares one `subtitle` resource with `subtitle_read`.
- `/subtitle` returns deterministic safe fixture subtitle candidates.
- Health and diagnostics are redaction-safe.
- Checked-in manifest and local smoke docs exist.

## M2 - DLNA Renderer Foundation

Exit criteria:

- `nako-dlna-renderer` builds as a workspace package.
- Manifest declares one `renderer_adapter` resource with renderer scopes.
- Readiness and manual target discovery work without live network operations.
- Command dispatch validates envelopes and returns plan-only safe results.
- Checked-in manifest and local smoke docs exist.

## M3 - Follow-On Contract

Exit criteria:

- External Acquisition Runner requirements are recorded.
- Docs explicitly exclude download execution, cloud-drive transfer, and
  password/code persistence from this lane.

## M4 - Closeout

Exit criteria:

- Focused package gates pass.
- Workspace formatting and diff hygiene pass.
- Catalog sync is completed or split with an explicit handoff.
