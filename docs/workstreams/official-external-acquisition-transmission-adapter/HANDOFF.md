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
OETA-050 mapped `query_status`, `pause`, `resume`, and `cancel` from
`transmission:<hash_string>` without rematerializing target links.

The adapter should consume `official-external-acquisition-materialization`
rather than reopening raw action input. The fixture profile remains the default
for local smoke and contract tests.

## Active Task

- Task ID: OETA-060
- Owner: codex
- Status: READY
- Scope: Route, smoke, README, full package regression, and integration
  documentation.

## Next Recommended Action

Run OETA-060:

1. Run the full runner package test suite.
2. Run local fixture smoke and keep it fixture-only by default.
3. Update README/smoke notes if they do not clearly describe Transmission as
   opt-in and fake-RPC-tested.
4. Record that live Transmission smoke is optional.
5. Prepare closeout inputs for OETA-070.

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
