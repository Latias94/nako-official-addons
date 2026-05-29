# Official External Acquisition Runner - Design

Status: Active
Last updated: 2026-05-29

## Problem

Nako can discover acquisition resources through read-only resource search and
can record host-owned selected-link or intake candidate references. The missing
boundary is the next explicit action: enqueueing, cancelling, pausing, resuming,
or querying an external runner such as qBittorrent, Transmission, aria2, an
ed2k handler, or an HTTP downloader.

That action must not be hidden inside `resource_search` or browser-submitted raw
URLs. It needs its own addon contract, host authorization, idempotency, audit,
progress, terminal states, and redaction rules.

## Target State

- External acquisition actions consume host-owned opaque references, not raw
  browser URLs or passwords.
- Nako core owns policy: which selected link can be acted on, which runner
  profile may be used, idempotency keys, cancellation authority, and audit.
- The official runner sidecar owns runner-specific external integration,
  request/response mapping, safe failure classification, and profile
  diagnostics.
- First implementation is contract/fixture-first. Real runner adapters are added
  only after the host/action envelope is stable.
- The read-only resource-search scope remains read-only.

## Scope

- Official addon protocol and catalog contract for external acquisition actions.
- A new official runner sidecar crate or equivalent official-addon package.
- Nako host action intake, dispatch, idempotency, progress, and audit surfaces.
- Safe diagnostics and failure taxonomy for runner calls.
- Workstream docs, gates, and closeout evidence.

## Non-Goals

- Cloud-drive save, transfer, copy, or provider-account operations.
- Durable password/code secret storage beyond host-owned opaque references.
- Browser resubmission of raw selected links or provider passwords.
- Site-specific search providers or resource-search expansion.
- Full Addon Manager lifecycle, package signing, installation, or supervision.
- Completing the separate `../nako` web acquisition intake lane.
- Multiple production runner adapters before the fixture/no-op contract is
  proven.

## Dependencies

- ADR: `../nako/docs/adr/0050-acquisition-resource-action-boundaries.md`.
- Follow-on contracts:
  `docs/workstreams/official-media-extension-addons/FOLLOW_ON_CONTRACTS.md` and
  `docs/workstreams/official-resource-search-first-class-protocol/FOLLOW_ON_CONTRACTS.md`.
- Admin intake UI is a related but separate lane:
  `../nako/docs/workstreams/web-admin-acquisition-intake`.

## Architecture Direction

The first durable boundary should be an Addon Task action contract, not a
downloader client or Addon Resource.

Host-to-runner request shape should include:

- `target_ref` as an opaque `selected_link_ref`, `intake_candidate_ref`, or
  `runner_job_ref`;
- `runner_profile_id`;
- `idempotency_key`;
- `operation` for enqueue, cancel, pause, resume, or status query;
- optional opaque `audit_ref`.

Runner response shape should include:

- accepted/rejected action status;
- runner-owned opaque job reference;
- redaction-safe message and safe facts;
- retryability;
- terminal/progress state when available;
- safe diagnostics for profile configuration.

The sidecar should begin with a fixture/no-op profile that proves idempotency,
redaction, cancellation, and progress semantics without touching external
services. Real runner adapters can then be added behind the same interface.

## Risk Plan

- Raw URL leakage: require host-owned references and redaction tests.
- Scope creep into cloud-drive transfer: keep cloud-drive actions out of this
  lane.
- Duplicate enqueue on retry: idempotency must be host-owned and test-covered.
- Runner credential leakage: profile diagnostics must report safe facts only.
- Web lane coupling: Admin UI route work remains separate; this lane exposes
  backend/action contracts first.
