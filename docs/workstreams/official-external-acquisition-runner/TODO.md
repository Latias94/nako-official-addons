# Official External Acquisition Runner - TODO

Status: Active
Last updated: 2026-05-29

Task IDs use the `OEAR` prefix.

## M0 - Scope And Contract Freeze

- [x] OEAR-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-external-acquisition-runner]
  Goal: Open the workstream, freeze boundaries, dependencies, non-goals, and
  validation gates.
  Validation: `python -m json.tool docs/workstreams/official-external-acquisition-runner/WORKSTREAM.json`;
  `git diff --check -- docs/workstreams/official-external-acquisition-runner`.
  Review: Confirm this lane is not resource search, cloud-drive transfer, or
  Addon Manager lifecycle.
  Evidence: initial workstream docs.
  Handoff: Continue with OEAR-020.

## M1 - Host And Protocol Action Contract

- [x] OEAR-020 [owner=codex] [deps=OEAR-010] [scope=../nako/crates/nako-addon-protocol,../nako/crates/nako-api,../nako/crates/nako-server,../nako/crates/nako-official-addon-catalog]
  Goal: Define the external acquisition action envelope, scopes, response
  schema, terminal/progress facts, and official catalog descriptor without
  adding a real runner adapter.
  Validation: focused protocol/catalog/server contract tests.
  Review: Browser clients must not submit raw URLs or passwords; action inputs
  must be host-owned opaque references.
  Evidence: protocol/catalog/server focused gates in EVIDENCE_AND_GATES.md.
  Handoff: Continue with OEAR-030.

## M2 - Fixture Runner Sidecar

- [x] OEAR-030 [owner=codex] [deps=OEAR-020] [scope=crates/nako-external-acquisition-runner,addons/external-acquisition-runner]
  Goal: Add a fixture/no-op official sidecar that implements manifest, health,
  action submission, status query, cancellation, and safe diagnostics for the
  new contract.
  Validation: package tests for manifest, action envelope, idempotency,
  cancellation, status, and redaction.
  Review: No external downloader network calls in this task.
  Evidence: command transcript in EVIDENCE_AND_GATES.md.
  Handoff: Continue with OEAR-040.

## M3 - Host Dispatch And Audit

- [x] OEAR-040 [owner=codex] [deps=OEAR-020,OEAR-030] [scope=../nako/crates/nako-server/src/app/addons,../nako/crates/nako-server/src/http]
  Goal: Let Nako core dispatch approved acquisition actions to the sidecar with
  host-owned idempotency, cancellation, progress/status refresh, and audit
  records.
  Validation: `nako-server` contract tests for action authorization,
  idempotency, redaction, cancellation, and retry-safe dispatch.
  Review: Do not mix this with Admin UI route implementation.
  Evidence: command transcript in EVIDENCE_AND_GATES.md.
  Handoff: Continue with OEAR-050.

## M4 - First Real Runner Adapter Decision

- [ ] OEAR-050 [owner=planner] [deps=OEAR-040] [scope=docs/workstreams/official-external-acquisition-runner]
  Goal: Decide whether the first real adapter is qBittorrent, Transmission,
  aria2, or HTTP downloader, based on config/security surface and testability.
  Validation: decision note and adapter-specific follow-on task or lane.
  Review: Do not add multiple production adapters without a proven common
  runner profile interface.
  Evidence: decision entry in JOURNAL.
  Handoff: Continue with OEAR-060 or split adapter work.

## M5 - Closeout

- [ ] OEAR-060 [owner=planner] [deps=OEAR-020,OEAR-030,OEAR-040,OEAR-050] [scope=docs/workstreams/official-external-acquisition-runner]
  Goal: Verify the lane, record residual risks, and split real runner adapter
  work if it is not completed in this lane.
  Validation: final focused gates pass or blockers are explicit external
  constraints.
  Review: Close only when docs, contracts, sidecar, host dispatch, and evidence
  agree.
  Evidence: EVIDENCE_AND_GATES.md, HANDOFF.md, WORKSTREAM.json, and closeout
  journal.
  Handoff: None after closeout.
