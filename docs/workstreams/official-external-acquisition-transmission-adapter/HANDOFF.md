# Official External Acquisition Transmission Adapter - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. OETA-010 defines the first production adapter work after
the materialization boundary closed. OETA-020 added opt-in Transmission profile
configuration, redaction-safe debug/diagnostics, manifest example updates, and
the matching official catalog configuration/secret schema in `../nako`.

The adapter should consume `official-external-acquisition-materialization`
rather than reopening raw action input. The fixture profile remains the default
for local smoke and contract tests.

## Active Task

- Task ID: OETA-030
- Owner: codex
- Status: READY
- Scope: Transmission RPC client boundary and fake RPC harness.

## Next Recommended Action

Run OETA-030:

1. Read the current Transmission RPC spec for method names and session-id
   retry behavior.
2. Add a typed client boundary with a fake transport.
3. Cover add, duplicate, get, start, stop, session-id retry, and redacted
   error behavior.
4. Keep raw RPC payloads and credentials out of public errors and Debug output.
5. Do not wire runner enqueue until the client harness is independently green.

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
