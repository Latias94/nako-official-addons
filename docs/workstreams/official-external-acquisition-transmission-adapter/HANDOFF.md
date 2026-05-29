# Official External Acquisition Transmission Adapter - Handoff

Status: Complete
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
OETA-060 added route-level Transmission enqueue coverage with fake
materialization/RPC and verified full package tests plus fixture-only local
smoke. OETA-070 closed the lane with residual risks and follow-ons explicit.

The adapter should consume `official-external-acquisition-materialization`
rather than reopening raw action input. The fixture profile remains the default
for local smoke and contract tests.

## Active Task

- Task ID: none
- Owner: planner
- Status: DONE
- Scope: none

## Next Recommended Action

No active task remains.

Next goal item: continue to `../nako` Android ACFH-090 closeout.

## Residual Risks

- Live Transmission daemon smoke was not required or run; fake RPC tests are the
  completion gate.
- `cancel` currently stops the Transmission torrent but does not remove torrent
  metadata or data. Deletion/removal policy should be a separate lane.
- Password-bearing material is rejected for Transmission until a policy for
  adapter-safe password/code use exists.
- qBittorrent, aria2, HTTP downloader, Usenet, and cloud-drive behavior remain
  separate follow-ons.

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
