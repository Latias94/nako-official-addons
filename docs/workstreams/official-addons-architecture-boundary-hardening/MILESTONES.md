# Official Addons Architecture Boundary Hardening - Milestones

Status: Complete
Last updated: 2026-05-29

## M0 - Scope And Evidence Freeze

Exit criteria:

- The lane has DESIGN, TODO, MILESTONES, EVIDENCE_AND_GATES, WORKSTREAM, and
  HANDOFF records.
- The task ledger is split by independently verifiable boundaries.
- Cross-repo dirty-worktree constraints are recorded.

## M1 - Manifest Source Of Truth

Exit criteria:

- Resource-search, subtitle-provider, and DLNA sidecar manifests reuse catalog
  builders for shared official facts.
- Runtime-specific config remains local to the sidecar.
- Checked-in example manifests still match runtime container manifests.
- Main catalog tests still validate official descriptors.

## M2 - Addon App Service Boundary Deepening

Exit criteria:

- `../nako/crates/nako-server/src/app/addons.rs` no longer owns unrelated
  user-facing workflows directly in the parent module.
- Extracted modules preserve existing behavior and redaction.
- Focused addon service tests pass.

## M3 - Provider HTTP Operation Policy

Exit criteria:

- Provider operation policy is explicit and test-covered.
- Retry-after behavior is honored for opted-in retryable responses.
- Any cache/throttle behavior is bounded, in-memory, and provider-operation
  scoped.
- No persistent cache or scheduler semantics are introduced.

## M4 - Notification Bridge Route Locality

Exit criteria:

- Route handlers remain thin adapters.
- Provider send orchestration and diagnostics rendering are tested as local
  modules.
- Public route responses remain compatible.

## M5 - Docs Cleanup

Exit criteria:

- Mature provider model research docs no longer claim completed resolver,
  capability, or field-policy work is future P0.
- Remaining follow-ons are accurate and point to current code boundaries.

## M6 - Closeout

Exit criteria:

- Fresh focused gates are recorded.
- Residual risks and follow-ons are explicit.
- No unrelated worktree changes are staged or reverted.
