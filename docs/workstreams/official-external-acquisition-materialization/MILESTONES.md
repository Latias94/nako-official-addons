# Official External Acquisition Materialization - Milestones

Status: Complete
Last updated: 2026-05-29

## M0 - Scope And Contract Freeze

Status: Complete via OEAM-010.

Exit criteria:

- Workstream docs exist and agree.
- Dependencies on ADR 0050, OEAR, and resource-search follow-on contracts are
  recorded.
- Non-goals prevent drift into production downloader adapters, cloud-drive
  transfer, browser raw-link submission, or Addon Manager lifecycle.

## M1 - Materialization Contract And ADR

Status: Complete via OEAM-020.

Exit criteria:

- The stable request/response vocabulary is explicit.
- Authorization binds addon identity, task/job identity, operation, target
  reference, runner profile, idempotency, and audit.
- Redaction rules cover debug, logs, task output, diagnostics, and admin APIs.

Primary gates:

- `cargo nextest run -p nako-addon-protocol external_acquisition --no-fail-fast`
- focused `nako-api` serialization tests if DTOs are added

## M2 - Host Resolver And Policy Gate

Status: Complete via OEAM-030.

Exit criteria:

- Nako can materialize approved `selected_link_ref` and `intake_candidate_ref`
  targets for enqueue.
- Expired, mismatched, wrong-operation, wrong-profile, and wrong-audit requests
  fail safely.
- Raw material stays out of persisted task input/output and public/admin API
  responses.

Primary gates:

- `cargo nextest run -p nako-server external_acquisition_materialization --no-fail-fast`
- `cargo nextest run -p nako-server acquisition_intake --no-fail-fast` when
  intake behavior changes

## M3 - Official Runner Materialization Client

Status: Complete via OEAM-040.

Exit criteria:

- The fixture runner has a materialization client abstraction.
- Tests prove materialized data is used only inside the enqueue attempt.
- Runner diagnostics remain redaction-safe.

Primary gates:

- `cargo nextest run -p nako-external-acquisition-runner materialization --no-fail-fast`
- `cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings`

## M4 - End-To-End Contract Proof

Status: Complete via OEAM-050.

Exit criteria:

- Host dispatch, sidecar materialization, and task output mapping work together.
- Idempotent retry does not re-materialize or enqueue inconsistent material.
- Evidence proves no raw URL/password/provider token leaks through observed
  responses.

Primary gates:

- `cargo nextest run -p nako-server addon_external_acquisition_action --no-fail-fast`
- `cargo nextest run -p nako-external-acquisition-runner --no-fail-fast`

## M5 - Closeout And Adapter Handoff

Status: Complete via OEAM-060.

Exit criteria:

- Fresh focused gates are recorded.
- Residual risks and follow-ons are explicit.
- `WORKSTREAM.json` status is updated.
- Transmission adapter work is unblocked or blocked for a concrete recorded
  reason.
