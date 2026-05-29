# Official External Acquisition Materialization - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

OEAM-020 is complete. OEAR closed with a safe external acquisition action
envelope and fixture runner, but production adapters remained blocked because
sidecars could only see opaque `selected_link_ref` or `intake_candidate_ref`
values. ADR 0054 now records the materialization boundary, and
`nako-addon-protocol` defines the route, schema constants, request/response
DTOs, operation helper, and redaction tests.

## Active Task

- Task ID: OEAM-030
- Owner: codex
- Files: `../nako/crates/nako-server/src/app/addons`,
  `../nako/crates/nako-server/src/app/acquisition_intake`,
  `../nako/crates/nako-server/src/app/resource_search`
- Validation: `cargo nextest run -p nako-server external_acquisition_materialization --no-fail-fast`;
  `cargo nextest run -p nako-server acquisition_intake --no-fail-fast` if
  intake resolver behavior changes
- Status: READY
- Review: `enqueue` may resolve selected-link or intake-candidate material;
  status/control operations must not. Raw material must stay out of persisted
  task JSON and browser-visible responses.
- Evidence: server tests proving allowed, expired, mismatched, and redacted
  cases.

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

## Blockers

- No blockers for OEAM-030.
- Transmission adapter work remains blocked until this lane closes or records a
  concrete remaining materialization blocker.

## Next Recommended Action

Implement OEAM-030: add the Nako-side runtime resolver and policy gate for
materialization requests.
