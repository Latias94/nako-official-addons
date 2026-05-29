# Official External Acquisition Materialization - Design

Status: Complete
Last updated: 2026-05-29

## Why This Lane Exists

`official-external-acquisition-runner` closed with a fixture runner and host
dispatch guard, but production runners still receive only opaque
`selected_link_ref` or `intake_candidate_ref` values. That is the correct action
boundary for browser and task input safety. A real runner adapter now needs a
separate host-owned materialization boundary that turns one approved opaque
reference into short-lived acquisition material without letting browsers or
addons resubmit raw URLs, passwords, provider tokens, or local paths.

## Relevant Authority

- ADR: `../nako/docs/adr/0050-acquisition-resource-action-boundaries.md`
- ADR: `../nako/docs/adr/0054-external-acquisition-materialization-boundary.md`
- Closed runner lane:
  `docs/workstreams/official-external-acquisition-runner`
- First-class resource-search follow-on contracts:
  `docs/workstreams/official-resource-search-first-class-protocol/FOLLOW_ON_CONTRACTS.md`
- Media extension follow-on contracts:
  `docs/workstreams/official-media-extension-addons/FOLLOW_ON_CONTRACTS.md`

This lane changed a hard-to-change host/runner contract. ADR 0054 records the
boundary, Nako server validates and resolves materialization requests, the
official runner has a materialization client boundary, and the fake
host/sidecar e2e test proves the redaction contract before production adapter
work starts.

## Problem

The runner action envelope intentionally carries opaque target references. That
prevents browser raw-link replay, but it also means Transmission, qBittorrent,
aria2, or an HTTP downloader cannot enqueue a real resource yet. If a production
adapter bypasses this by accepting raw material in task input, the system loses
the guarantees established by ADR 0050:

- host policy no longer owns which selected link may be acted on;
- retry and idempotency can enqueue different external material under the same
  task;
- audit records cannot prove which host-owned reference was materialized;
- secrets and provider access codes are likely to leak through logs, task JSON,
  diagnostics, or browser-visible API responses.

## Target State

- The sidecar can request materialization only for the target reference already
  approved in the current external acquisition action task.
- Nako core owns resolution from `selected_link_ref` or `intake_candidate_ref`
  to acquisition material.
- Materialized data is short-lived, redacted by default, and never persisted in
  task input/output, catalog responses, or browser-visible admin responses.
- Authorization binds materialization to addon identity, task/job identity,
  declaration identity, operation, runner profile, idempotency key, and audit
  reference.
- `enqueue` may materialize selected-link or intake-candidate targets.
  `cancel`, `pause`, `resume`, and `query_status` operate on `runner_job_ref`
  and do not materialize acquisition links.
- The official fixture runner proves the materialization client boundary without
  adding Transmission or any other production adapter.

## In Scope

- A host-to-sidecar materialization request/response contract.
- Runtime authorization and validation rules in `../nako`.
- Host resolver behavior for `selected_link_ref` and `intake_candidate_ref`.
- TTL, single-use or bounded-use policy, audit anchors, and redaction rules.
- Official fixture runner client support and tests proving the boundary.
- Workstream evidence, handoff, and closeout docs.

## Out Of Scope

- Transmission, qBittorrent, aria2, ed2k, or HTTP downloader adapter code.
- Cloud-drive save, transfer, copy, or provider-account operations.
- Browser API routes that accept raw URLs or passwords.
- Durable provider password/code storage beyond existing host-owned reference
  policy.
- Addon Manager installation, signing, lifecycle, or supervision.
- Product UI route work in the web acquisition intake lane.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Existing external acquisition actions stay task-based. | High | OEAR closed with `acquisition_action_run` and dispatch tests. | A non-task action model would require reopening OEAR-level protocol design. |
| Materialization belongs to Nako core, not the sidecar. | High | ADR 0050 keeps resource references host-owned. | Letting sidecars resolve raw links directly would reintroduce unsafe browser/addon authority. |
| The first production adapter remains Transmission after this lane. | Medium | OEAR-050 decision note. | If adapter priority changes, materialization still remains a prerequisite for any real adapter. |
| `selected_link_ref` may be more transient than `intake_candidate_ref`. | Medium | Resource-search selection and intake lanes have different lifetimes. | The resolver may need to promote selected links into an intake/materialization store before dispatch. |
| Existing addon runtime authentication can be the first implementation base. | Medium | Nako already has addon task dispatch and runtime identity concepts. | If the runtime token model is insufficient, OEAM-020 must introduce a narrower materialization token or ADR. |

## Architecture Direction

The materialization API should be a Nako-owned runtime capability, not an
extension to browser admin APIs and not additional raw fields in
`AddonExternalAcquisitionActionRequest`.

The sidecar request should include enough context for Nako to prove that the
request matches the current approved action:

- `task_run_ref` or equivalent host job identity;
- `declaration_id`;
- `runner_profile_id`;
- `operation` with `enqueue` as the only materializing operation;
- `target_ref`;
- `idempotency_key`;
- `audit_ref`;
- optional materialization purpose such as `external_acquisition_enqueue`.

The host response should return a short-lived materialization result:

- `materialization_ref`;
- `expires_at`;
- normalized link material such as magnet URI, ed2k URI, HTTP URL, or torrent
  bytes reference, depending on what the host has approved;
- optional access-code facts only when policy allows the sidecar to use them;
- redaction-safe facts for diagnostics;
- no raw material in debug output or task output.

The sidecar should treat materialization as a runner-internal dependency. It
may cache the result only inside the current enqueue attempt and must not expose
it through status, diagnostics, logs, or smoke-test output.

## Closeout Condition

This lane can close when:

- the ADR or contract note records the materialization boundary;
- Nako exposes and tests the materialization runtime contract;
- host resolution covers both selected-link and intake-candidate references or
  explicitly splits the missing source as a follow-on;
- the official fixture runner uses the materialization client boundary in tests;
- evidence gates pass freshly;
- Transmission adapter work is unblocked and remains split from this lane.
