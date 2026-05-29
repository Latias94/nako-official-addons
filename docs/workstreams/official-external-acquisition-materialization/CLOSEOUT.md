# Official External Acquisition Materialization - Closeout

Date: 2026-05-29

Status: Complete

## Result

The host-to-runner external acquisition materialization boundary is implemented
and verified.

The lane delivered:

- ADR 0054 and protocol DTOs for materialization.
- Nako runtime route `POST /addon/v1/acquisition/materialize`.
- Host resolver and policy gate for selected-link and intake-candidate targets.
- Shared `nako-addon-client` runtime client support.
- Official runner materializer boundary with host-runtime, unavailable, and
  fixture-local implementations.
- Server e2e test proving direct dispatch, sidecar materialization, and redacted
  task completion compose without exposing raw material.

## Closeout Review

No blocking findings.

Workstream compliance:

- All TODO tasks OEAM-010 through OEAM-060 are complete.
- Target state in `DESIGN.md` is met for the pre-Transmission materialization
  contract.
- Non-goals were preserved: no Transmission, qBittorrent, aria2, cloud-drive
  transfer, browser raw URL submission, or Addon Manager lifecycle work was
  added.

Code quality:

- Runtime materialization belongs to shared `nako-addon-client`, not ad hoc
  runner HTTP.
- Nako server owns resolution and policy checks.
- The official runner treats materialization as an injected dependency and keeps
  raw material out of task output, diagnostics, and Debug paths.

Missing gates:

- None for the changed scope. Full workspace nextest was intentionally skipped;
  focused package gates cover the touched protocol/client/server/runner surface.

## Fresh Verification

- `cargo nextest run -p nako-server addon_external_acquisition --no-fail-fast`
  passed: 7 tests.
- `cargo check -p nako-server --tests` passed.
- `cargo nextest run -p nako-external-acquisition-runner --no-fail-fast`
  passed: 16 tests.
- `python -m json.tool docs/workstreams/official-external-acquisition-materialization/WORKSTREAM.json`
  passed.
- `git diff --check -- docs/workstreams/official-external-acquisition-materialization`
  passed.

## Follow-On

Open `official-external-acquisition-transmission-adapter` next.

That lane should consume the materialization boundary and stay scoped to the
first production downloader adapter. Cloud-drive transfer, cloud-save provider
flows, and broader provider password/code storage remain separate policy lanes.

## Residual Risks

- Materialization is currently bounded by running task context and short TTL,
  but not persisted as a single-use token.
- Password/code materialization is not modeled as a separate field yet.
- Only `magnet`, `ed2k`, and `web` link types are accepted for external runner
  materialization.
