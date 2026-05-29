# Official External Acquisition Transmission Adapter - TODO

Status: Active
Last updated: 2026-05-29

Task IDs use the `OETA` prefix.

## M0 - Scope And Evidence Freeze

- [x] OETA-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-external-acquisition-transmission-adapter]
  Goal: Open the workstream with boundaries, non-goals, task order, and evidence anchors for the first production Transmission adapter.
  Validation: `python -m json.tool docs/workstreams/official-external-acquisition-transmission-adapter/WORKSTREAM.json`; `git diff --check -- docs/workstreams/official-external-acquisition-transmission-adapter`.
  Review: Confirm this lane consumes OEAM materialization and does not reopen browser raw URL submission, cloud-drive transfer, or Addon Manager lifecycle.
  Evidence: initial workstream docs.
  Handoff: DONE. Continue with OETA-020.

## M1 - Transmission Profile And Secret Policy

- [x] OETA-020 [owner=codex] [deps=OETA-010] [scope=crates/nako-external-acquisition-runner,addons/external-acquisition-runner,../nako/crates/nako-official-addon-catalog]
  Goal: Add opt-in Transmission profile configuration, redacted debug behavior, manifest/config schema representation, and diagnostics that show profile readiness without exposing credentials.
  Validation: `cargo nextest run -p nako-external-acquisition-runner config manifest diagnostics --no-fail-fast`; `cargo fmt -p nako-external-acquisition-runner -- --check`.
  Review: Credentials, bearer tokens, endpoint auth, and RPC session ids must not appear in Debug, diagnostics, task output, checked-in manifests, or smoke output.
  Evidence: config/manifest tests and documentation updates.
  Handoff: DONE. Transmission config, secret reference schema, manifest example, official catalog schema, and redaction-safe diagnostics are in place. Continue with OETA-030.

## M2 - Transmission RPC Client Harness

- [x] OETA-030 [owner=codex] [deps=OETA-020] [scope=crates/nako-external-acquisition-runner/src/transmission.rs]
  Goal: Add a typed Transmission RPC client boundary with fake transport tests for session-id retry, torrent add, duplicate, get, start, stop, and redacted error behavior.
  Validation: `cargo nextest run -p nako-external-acquisition-runner transmission --no-fail-fast`; `cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings`.
  Review: Keep raw RPC payloads and credentials out of public errors and Debug output.
  Evidence: fake RPC tests and client module.
  Handoff: DONE. Transmission RPC client and fake transport tests cover session-id retry, add/duplicate, get, start, stop, and redacted errors. Continue with OETA-040.

## M3 - Enqueue Through Materialization

- [ ] OETA-040 [owner=codex] [deps=OETA-020,OETA-030] [scope=crates/nako-external-acquisition-runner/src/runner.rs,crates/nako-external-acquisition-runner/src/transmission.rs]
  Goal: Route `enqueue` for the Transmission profile through host materialization and Transmission add, returning `transmission:<hash_string>` with safe facts.
  Validation: `cargo nextest run -p nako-external-acquisition-runner transmission enqueue materialization --no-fail-fast`.
  Review: Do not materialize unsupported operations; reject unsupported link types safely; do not echo raw URI, password, idempotency key, materialization ref, endpoint, username, or password.
  Evidence: runner tests for accepted, duplicate, unsupported, and redaction cases.
  Handoff: Continue with OETA-050.

## M4 - Status And Control Operations

- [ ] OETA-050 [owner=codex] [deps=OETA-040] [scope=crates/nako-external-acquisition-runner/src/runner.rs,crates/nako-external-acquisition-runner/src/transmission.rs]
  Goal: Map `query_status`, `cancel`, `pause`, and `resume` to Transmission hash operations from `runner_job_ref`.
  Validation: `cargo nextest run -p nako-external-acquisition-runner transmission status cancel pause resume --no-fail-fast`.
  Review: These operations must not call materialization and must fail safely for non-Transmission or malformed runner job refs.
  Evidence: runner operation tests and fake RPC transcripts.
  Handoff: Continue with OETA-060.

## M5 - Route, Smoke, And Redaction Integration

- [ ] OETA-060 [owner=codex] [deps=OETA-050] [scope=crates/nako-external-acquisition-runner,addons/external-acquisition-runner]
  Goal: Ensure routes, health diagnostics, README, and local smoke behavior clearly distinguish fixture and Transmission profiles.
  Validation: `cargo nextest run -p nako-external-acquisition-runner --no-fail-fast`; `pwsh -File addons/external-acquisition-runner/smoke.local.ps1`.
  Review: Default local smoke must remain fixture-only and must not require a live Transmission daemon.
  Evidence: route tests, smoke output notes, and README.
  Handoff: Continue with OETA-070.

## M6 - Closeout

- [ ] OETA-070 [owner=planner] [deps=OETA-020,OETA-030,OETA-040,OETA-050,OETA-060] [scope=docs/workstreams/official-external-acquisition-transmission-adapter]
  Goal: Verify the lane, record residual risks, and split any follow-on adapter breadth.
  Validation: final focused gates from `EVIDENCE_AND_GATES.md`; `git diff --check`; workstream JSON validation.
  Review: No blocking workstream, redaction, or adapter boundary findings remain.
  Evidence: `CLOSEOUT.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md`.
  Handoff: Continue to Android ACFH-090 after this lane closes.
