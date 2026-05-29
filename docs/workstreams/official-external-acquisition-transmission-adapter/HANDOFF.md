# Official External Acquisition Transmission Adapter - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. OETA-010 defines the first production adapter work after
the materialization boundary closed. OETA-020 added opt-in Transmission profile
configuration, redaction-safe debug/diagnostics, manifest example updates, and
the matching official catalog configuration/secret schema in `../nako`.
OETA-030 added the typed Transmission RPC client boundary and fake transport
tests for session-id retry, add/duplicate, get, start, stop, and redacted
errors.

The adapter should consume `official-external-acquisition-materialization`
rather than reopening raw action input. The fixture profile remains the default
for local smoke and contract tests.

## Active Task

- Task ID: OETA-040
- Owner: codex
- Status: READY
- Scope: Runner enqueue through host materialization and Transmission add.

## Next Recommended Action

Run OETA-040:

1. Split runner profile routing if `runner.rs` becomes hard to reason about.
2. Route Transmission `enqueue` through `ExternalAcquisitionMaterializer`.
3. Accept supported material types and reject unsupported types with safe
   errors.
4. Return `transmission:<hash_string>` only after Transmission add succeeds.
5. Ensure duplicate add maps to a stable idempotent response without leaking
   raw material or materialization refs.

## Guardrails

- Do not accept raw URL, password, or downloader credential fields in task
  payload.
- Do not call Transmission before host materialization succeeds.
- Do not materialize status/cancel/pause/resume operations.
- Do not commit live credentials, session ids, or raw RPC payloads.
- Do not add qBittorrent, aria2, cloud-drive, or generic downloader behavior in
  this lane.

## Blockers

None known.

## Follow-Ons

- qBittorrent adapter after Transmission proves the common profile boundary.
- aria2 or HTTP downloader only after separate storage and log-safety policy.
- Cloud-drive transfer/save remains separate from external acquisition runner
  adapters.
