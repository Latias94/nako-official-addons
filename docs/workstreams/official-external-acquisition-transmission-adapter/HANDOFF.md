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
errors. OETA-040 wired Transmission enqueue through host materialization,
mapped duplicate add to `AlreadyExists`, rejected unsupported material safely,
and returned only `transmission:<hash_string>` plus safe facts.

The adapter should consume `official-external-acquisition-materialization`
rather than reopening raw action input. The fixture profile remains the default
for local smoke and contract tests.

## Active Task

- Task ID: OETA-050
- Owner: codex
- Status: READY
- Scope: Transmission status, pause, resume, and cancel from runner job refs.

## Next Recommended Action

Run OETA-050:

1. Parse and validate `transmission:<hash_string>` runner job refs.
2. Map query status through `torrent-get`.
3. Map pause/resume through stop/start without materialization.
4. Decide whether cancel should stop only or remove later; first slice should
   not delete data unless a separate remove policy is accepted.
5. Prove status/control operations do not call materialization.

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
