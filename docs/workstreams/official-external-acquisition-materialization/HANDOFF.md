# Official External Acquisition Materialization - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

The lane is open. OEAR closed with a safe external acquisition action envelope
and fixture runner, but production adapters remain blocked because sidecars can
only see opaque `selected_link_ref` or `intake_candidate_ref` values. This lane
defines how Nako materializes those references for an approved runner action
without putting raw URLs, passwords, provider tokens, or local paths into
browser APIs or task JSON.

## Active Task

- Task ID: OEAM-020
- Owner: codex
- Files: `../nako/docs/adr`, `../nako/crates/nako-addon-protocol`,
  `../nako/crates/nako-api`
- Validation: `cargo nextest run -p nako-addon-protocol external_acquisition --no-fail-fast`;
  focused `nako-api` serialization tests if DTOs are added
- Status: READY
- Review: Contract must not expose raw acquisition material through task
  input/output, debug output, browser-visible DTOs, or diagnostics.
- Evidence: ADR or contract note plus protocol/API tests.

## Decisions Since Open

- Materialization is a separate runtime boundary, not extra raw fields on
  `AddonExternalAcquisitionActionRequest`.
- Only `enqueue` can materialize selected-link or intake-candidate targets.
- Status/cancel/pause/resume operations use `runner_job_ref` and do not
  materialize link data.
- The official sidecar remains fixture-only until the host materialization
  contract is implemented and verified.

## Blockers

- No blockers for OEAM-020.
- Transmission adapter work remains blocked until this lane closes or records a
  concrete remaining materialization blocker.

## Next Recommended Action

Implement OEAM-020: add or update an ADR for the materialization contract, then
define the stable protocol/API DTOs and redaction tests in `../nako`.
