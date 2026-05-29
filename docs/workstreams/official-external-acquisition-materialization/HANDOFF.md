# Official External Acquisition Materialization - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

OEAM-030 is complete. OEAR closed with a safe external acquisition action
envelope and fixture runner, but production adapters remained blocked because
sidecars could only see opaque `selected_link_ref` or `intake_candidate_ref`
values. ADR 0054 now records the materialization boundary,
`nako-addon-protocol` defines the route and DTOs, and `nako-server` exposes the
runtime route with host-side action-context validation and candidate resolution.

## Active Task

- Task ID: OEAM-040
- Owner: codex
- Files: `crates/nako-external-acquisition-runner`,
  `addons/external-acquisition-runner`
- Validation: `cargo nextest run -p nako-external-acquisition-runner materialization --no-fail-fast`;
  `cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings`
- Status: READY
- Review: Keep the sidecar fixture-only. It may call a fake materialization
  client in tests but must not add Transmission or other downloader network
  calls.
- Evidence: runner tests proving materialized data is used only inside enqueue
  and not exposed through diagnostics or task output.

## Decisions Since Open

- Materialization is a separate runtime boundary, not extra raw fields on
  `AddonExternalAcquisitionActionRequest`.
- Only `enqueue` can materialize selected-link or intake-candidate targets.
- Status/cancel/pause/resume operations use `runner_job_ref` and do not
  materialize link data.
- The official sidecar remains fixture-only until the host materialization
  contract is implemented and verified.
- ADR 0054 defines materialization as `POST /addon/v1/acquisition/materialize`.
- Protocol schema IDs are
  `nako.addon.external_acquisition_materialization.request.v1` and
  `nako.addon.external_acquisition_materialization.response.v1`.
- `AddonExternalAcquisitionOperation::can_materialize_target()` is true only
  for `enqueue`.
- Nako runtime route is `POST /addon/v1/acquisition/materialize`.
- Materialization requires a running external acquisition action task owned by
  the caller addon token.
- The runtime request must match stored action task context: declaration,
  target ref, runner profile, idempotency key, operation, and audit ref.
- `selected_link_ref` resolves only to resource-search-selection intake
  candidates. `intake_candidate_ref` resolves by intake candidate id.
- External runner materialization rejects cloud-drive link types; allowed first
  link types are `magnet`, `ed2k`, and `web`.

## Blockers

- No blockers for OEAM-040.
- Transmission adapter work remains blocked until this lane closes or records a
  concrete remaining materialization blocker.

## Next Recommended Action

Implement OEAM-040: add a materialization client boundary to the fixture runner
and prove it does not leak materialized URI/password data.
