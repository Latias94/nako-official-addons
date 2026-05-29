# Official External Acquisition Materialization - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

OEAM-040 is complete. OEAR closed with a safe external acquisition action
envelope and fixture runner, but production adapters remained blocked because
sidecars could only see opaque `selected_link_ref` or `intake_candidate_ref`
values. ADR 0054 records the materialization boundary, `nako-addon-protocol`
defines the route and DTOs, `nako-server` exposes the runtime route with
host-side action-context validation and candidate resolution, and the official
fixture runner now uses a materialization client abstraction without adding a
production downloader adapter.

## Active Task

- Task ID: OEAM-050
- Owner: codex
- Files: `../nako/crates/nako-server/src/http/tests`,
  `crates/nako-external-acquisition-runner`
- Validation: `cargo nextest run -p nako-server addon_external_acquisition_action --no-fail-fast`;
  `cargo nextest run -p nako-external-acquisition-runner --no-fail-fast`
- Status: READY
- Review: Prove the host dispatch, sidecar materialization request, and redacted
  task completion compose across the fake host/sidecar boundary. Do not add
  Transmission or external downloader calls.
- Evidence: integration tests proving raw URI, password, materialization ref,
  selected-link refs, and idempotency keys do not leak through observed task
  responses.

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
- `nako-addon-client` exposes
  `NakoRuntimeClient::materialize_external_acquisition()` for sidecars instead
  of requiring each runner to hand-roll runtime HTTP calls.
- The fixture runner owns an `ExternalAcquisitionMaterializer` boundary with
  `nako_runtime`, `unavailable`, and `fixture_local` implementations.
- Fixture local materialization remains the default so local smoke tests still
  run without a Nako host. Enabling host materialization requires
  `NAKO_EXTERNAL_ACQUISITION_RUNNER_MATERIALIZATION_ENABLED=true`,
  `NAKO_EXTERNAL_ACQUISITION_RUNNER_NAKO_BASE_URL`, and
  `NAKO_EXTERNAL_ACQUISITION_RUNNER_ADDON_TOKEN`.
- Runner responses expose only safe materialization facts: client kind, link
  type, password presence, and materialization-ref presence.

## Blockers

- No blockers for OEAM-050.
- Transmission adapter work remains blocked until this lane closes or records a
  concrete remaining materialization blocker.

## Next Recommended Action

Implement OEAM-050: prove the full approved-action-to-sidecar materialization
flow across Nako server and official runner test boundaries.
