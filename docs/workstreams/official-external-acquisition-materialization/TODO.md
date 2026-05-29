# Official External Acquisition Materialization - TODO

Status: Active
Last updated: 2026-05-29

Task IDs use the `OEAM` prefix.

## M0 - Scope And Contract Freeze

- [x] OEAM-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-external-acquisition-materialization]
  Goal: Open the workstream and freeze problem statement, target state,
  non-goals, task order, and evidence anchors.
  Validation: `python -m json.tool docs/workstreams/official-external-acquisition-materialization/WORKSTREAM.json`;
  `git diff --check -- docs/workstreams/official-external-acquisition-materialization`.
  Review: Confirm this lane only defines host-to-runner materialization and
  does not implement a production downloader adapter.
  Evidence: initial workstream docs.
  Handoff: Continue with OEAM-020.

## M1 - Materialization Contract And ADR

- [ ] OEAM-020 [owner=codex] [deps=OEAM-010] [scope=../nako/docs/adr,../nako/crates/nako-addon-protocol,../nako/crates/nako-api]
  Goal: Define the host runtime materialization request/response contract,
  schema IDs, authorization context, redaction semantics, and stable wire names.
  Validation: `cargo nextest run -p nako-addon-protocol external_acquisition --no-fail-fast`;
  focused API serialization tests if `nako-api` DTOs are added.
  Review: The contract must not add raw URLs, passwords, or provider tokens to
  browser-visible request payloads or task input/output.
  Evidence: ADR or contract note plus protocol/API tests.
  Handoff: Continue with OEAM-030 after the contract is stable.

## M2 - Host Resolver And Policy Gate

- [ ] OEAM-030 [owner=codex] [deps=OEAM-020] [scope=../nako/crates/nako-server/src/app/addons,../nako/crates/nako-server/src/app/acquisition_intake,../nako/crates/nako-server/src/app/resource_search]
  Goal: Implement Nako-side materialization resolution for approved
  `selected_link_ref` and `intake_candidate_ref` targets, including TTL,
  operation checks, audit binding, profile binding, and redaction.
  Validation: `cargo nextest run -p nako-server external_acquisition_materialization --no-fail-fast`;
  `cargo nextest run -p nako-server acquisition_intake --no-fail-fast` if intake
  resolver behavior changes.
  Review: `enqueue` may resolve link material; `cancel`, `pause`, `resume`, and
  `query_status` must not.
  Evidence: server tests proving allowed, expired, mismatched, and redacted
  cases.
  Handoff: Continue with OEAM-040.

## M3 - Official Runner Materialization Client

- [ ] OEAM-040 [owner=codex] [deps=OEAM-020,OEAM-030] [scope=crates/nako-external-acquisition-runner,addons/external-acquisition-runner]
  Goal: Add a materialization client boundary to the official fixture runner
  and prove the runner can request host material without exposing it in logs,
  diagnostics, or task output.
  Validation: `cargo nextest run -p nako-external-acquisition-runner materialization --no-fail-fast`;
  `cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings`.
  Review: Keep the runner fixture-only; do not add Transmission or other
  downloader network calls.
  Evidence: runner tests and local smoke update if the public smoke contract
  changes.
  Handoff: Continue with OEAM-050.

## M4 - End-To-End Contract Proof

- [ ] OEAM-050 [owner=codex] [deps=OEAM-030,OEAM-040] [scope=../nako/crates/nako-server/src/http/tests,crates/nako-external-acquisition-runner]
  Goal: Prove the full flow from approved action dispatch to sidecar
  materialization and redacted host task completion using a fake host/sidecar
  boundary.
  Validation: `cargo nextest run -p nako-server addon_external_acquisition_action --no-fail-fast`;
  `cargo nextest run -p nako-external-acquisition-runner --no-fail-fast`.
  Review: Task JSON, diagnostics, and admin-visible responses must remain free
  of raw acquisition material.
  Evidence: integration tests and evidence log entries.
  Handoff: Continue with OEAM-060.

## M5 - Closeout And Adapter Handoff

- [ ] OEAM-060 [owner=planner] [deps=OEAM-020,OEAM-030,OEAM-040,OEAM-050] [scope=docs/workstreams/official-external-acquisition-materialization]
  Goal: Verify the lane, record residual risks, and open or recommend the
  Transmission adapter lane.
  Validation: fresh focused gates from `EVIDENCE_AND_GATES.md`.
  Review: Run `review-workstream` before closeout; run
  `verify-rust-workstream` for final evidence.
  Evidence: `EVIDENCE_AND_GATES.md`, `HANDOFF.md`, `WORKSTREAM.json`, and
  closeout journal.
  Handoff: Start `official-external-acquisition-transmission-adapter` only
  after this lane closes or explicitly records the remaining blocker.
