# Official External Acquisition Transmission Adapter - Handoff

Status: Active
Last updated: 2026-05-29

## Current State

This lane is open. OETA-010 defines the first production adapter work after
the materialization boundary closed.

The adapter should consume `official-external-acquisition-materialization`
rather than reopening raw action input. The fixture profile remains the default
for local smoke and contract tests.

## Active Task

- Task ID: OETA-020
- Owner: codex
- Status: READY
- Scope: Transmission profile configuration, secret policy, manifest/config
  schema, and redaction-safe diagnostics.

## Next Recommended Action

Run OETA-020:

1. Inspect existing config and manifest schema.
2. Add opt-in Transmission profile configuration.
3. Add redacted Debug and environment parsing tests.
4. Update the checked-in manifest example only after runtime manifest tests
   agree.
5. Keep fixture mode as the default.

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
