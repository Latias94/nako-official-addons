# Official External Acquisition Transmission Adapter - Design

Status: Active
Last updated: 2026-05-29

## Why This Lane Exists

`official-external-acquisition-runner` closed with a fixture runner and
`official-external-acquisition-materialization` closed with a host-owned
materialization boundary. A production downloader adapter can now be added
without weakening the action contract: the runner still receives only opaque
host references in task input, then materializes the approved target through
Nako runtime before calling downloader software.

Transmission is the first production adapter because its RPC API can return a
stable torrent hash after add or duplicate handling. That lets the runner expose
`runner_job_ref = transmission:<hash_string>` and keep status, pause, resume,
and cancel operations keyed by an opaque job reference instead of raw download
material.

## Relevant Authority

- Closed runner lane: `docs/workstreams/official-external-acquisition-runner`
- Closed materialization lane:
  `docs/workstreams/official-external-acquisition-materialization`
- Host materialization ADR:
  `../nako/docs/adr/0054-external-acquisition-materialization-boundary.md`
- Acquisition boundary ADR:
  `../nako/docs/adr/0050-acquisition-resource-action-boundaries.md`
- Transmission RPC reference:
  `https://github.com/transmission/transmission/blob/main/docs/rpc-spec.md`

## Problem

The fixture runner proves task dispatch and materialization, but it does not
enqueue real work. If a Transmission adapter is added as a quick network call
inside the fixture runner, several boundaries would collapse:

- fixture/no-op behavior would become mixed with production downloader behavior;
- Transmission credentials could leak through debug output or manifest config;
- raw magnet/URL/password material could appear in task output or diagnostics;
- idempotency could enqueue duplicate torrents if duplicate handling is not
  keyed by Transmission hash;
- status/cancel/pause/resume could depend on browser-visible raw material
  instead of the runner job reference.

## Target State

- The sidecar exposes at least two runner profile kinds:
  - `fixture`: local no-op profile for smoke and contract tests;
  - `transmission`: production profile backed by Transmission RPC.
- Transmission profile configuration is explicit and redaction-safe:
  endpoint, timeout, optional basic auth/user policy, and TLS policy are parsed
  from environment or secret references, never from task payload.
- Enqueue materializes only the host-approved target for the running action
  task, then submits supported material to Transmission.
- Transmission duplicate or existing-torrent behavior is idempotent and maps to
  `AlreadyExists` with the same `runner_job_ref` where possible.
- `runner_job_ref` is stable and opaque:
  `transmission:<hash_string>`.
- Status, cancel, pause, and resume operate only on `runner_job_ref` and never
  rematerialize target links.
- Runner responses expose safe facts such as profile kind, link type, hash
  presence, and RPC outcome category, but not magnet URIs, HTTP URLs,
  passwords, credentials, session IDs, or raw RPC payloads.
- Tests use a fake Transmission RPC service; normal tests do not require a real
  Transmission daemon or network.

## In Scope

- Transmission profile config and manifest/config schema updates.
- A small Transmission RPC client boundary with session-id handshake support.
- Fake Transmission RPC test harness for add, duplicate, status, pause, resume,
  cancel, authentication/error, and redaction behavior.
- Runner profile routing from `runner_profile_id` to fixture or Transmission.
- Enqueue/status/cancel/pause/resume mapping for the first supported material
  shape.
- Documentation, smoke notes, and workstream closeout evidence.

## Out Of Scope

- qBittorrent, aria2, Usenet, RSS, cloud-drive, or generic HTTP downloader
  adapters.
- Browser routes that accept raw URLs, passwords, or downloader credentials.
- Persistent password/code storage beyond existing host materialization policy.
- Addon Manager lifecycle, package signing, process supervision, or automatic
  sidecar installation.
- Managed Import promotion apply or Media Library mutation.
- Cloud-drive save/transfer semantics.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Transmission can be the first production adapter. | High | OEAR-050 selected Transmission after comparing qBittorrent, aria2, and HTTP downloader. | Re-score adapter choice before OETA-030. |
| Host materialization returns enough material for Transmission enqueue. | Medium | OEAM materializes `magnet`, `ed2k`, and `web` link types. | OETA must reject unsupported link types safely and may need a later torrent-bytes materialization policy. |
| A stable Transmission hash is available after add or duplicate. | Medium | Transmission RPC exposes torrent hash facts for added/duplicate torrents. | Runner job refs need a different adapter-owned opaque id. |
| Current runner can support multiple profile kinds without a separate crate. | Medium | Existing `FixtureRunner` owns profile checks and materialization. | Split a profile router module before adapter code grows. |
| Fake RPC tests are sufficient for normal CI. | High | A live Transmission daemon is an integration environment, not a unit-test prerequisite. | Add a separate live-smoke script, not a required CI gate. |

## Architecture Direction

Separate three concepts that are currently adjacent in the fixture runner:

- `ExternalAcquisitionRunner`: task operation handler that maps protocol request
  and response shapes.
- `RunnerProfile`: adapter selected by `runner_profile_id`.
- `TransmissionClient`: RPC transport boundary with redacted errors and typed
  request/response mapping.

The first implementation may evolve `FixtureRunner` in place, but the adapter
must not let fixture-only state dictate Transmission behavior. If adding
Transmission makes `runner.rs` harder to reason about, split profile routing
before implementing the RPC client.

Transmission enqueue should use materialization once per accepted enqueue
attempt. Status-like operations should parse `transmission:<hash_string>`, call
Transmission by hash, and return safe state/progress facts. Unsupported material
types should fail with safe error codes such as
`transmission_link_type_unsupported`.

## Closeout Condition

This lane can close when:

- the Transmission profile is opt-in and redaction-safe;
- fake RPC tests prove enqueue, duplicate replay, status, pause, resume, cancel,
  and safe errors;
- materialization remains the only source of raw acquisition material;
- fixture mode still works by default;
- focused package tests, formatting, clippy, and docs checks pass;
- live Transmission smoke remains optional and clearly documented.
