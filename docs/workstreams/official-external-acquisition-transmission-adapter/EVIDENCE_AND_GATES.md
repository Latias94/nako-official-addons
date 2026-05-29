# Official External Acquisition Transmission Adapter - Evidence And Gates

Status: Active
Last updated: 2026-05-29

## Gate Policy

Run focused gates at each slice and record fresh evidence before claiming a task
done. Normal CI must not require a live Transmission daemon.

## Required Gates

| Gate | Purpose | Required before closeout |
| --- | --- | --- |
| `python -m json.tool docs/workstreams/official-external-acquisition-transmission-adapter/WORKSTREAM.json` | Workstream metadata validity | Yes |
| `git diff --check -- docs/workstreams/official-external-acquisition-transmission-adapter` | Documentation whitespace check | Yes |
| `cargo nextest run -p nako-external-acquisition-runner config manifest transmission --no-fail-fast` | Profile/config/RPC-focused tests | Yes after OETA-030 |
| `cargo nextest run -p nako-external-acquisition-runner transmission enqueue materialization --no-fail-fast` | Enqueue/materialization tests | Yes after OETA-040 |
| `cargo nextest run -p nako-external-acquisition-runner transmission status cancel pause resume --no-fail-fast` | Status/control tests | Yes after OETA-050 |
| `cargo nextest run -p nako-external-acquisition-runner --no-fail-fast` | Full runner package regression | Yes |
| `cargo fmt -p nako-external-acquisition-runner -- --check` | Formatting | Yes |
| `cargo clippy -p nako-external-acquisition-runner --tests -- -D warnings` | Lint gate | Yes unless blocked by unrelated upstream debt |
| `pwsh -File addons/external-acquisition-runner/smoke.local.ps1` | Default fixture smoke remains usable | Yes after route/smoke changes |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-29 | OETA-010 | `python -m json.tool docs/workstreams/official-external-acquisition-transmission-adapter/WORKSTREAM.json`; `git diff --check -- docs/workstreams/official-external-acquisition-transmission-adapter` | Pass |
| 2026-05-29 | OETA-020 | `cargo nextest run -p nako-external-acquisition-runner config manifest diagnostics --no-fail-fast`; `cargo nextest run -p nako-official-addon-catalog external_acquisition_runner_default_manifest_matches_official_catalog_facts --no-fail-fast`; `cargo fmt -p nako-external-acquisition-runner -- --check`; `cargo fmt -p nako-official-addon-catalog -- --check` | Pass |

## Live Smoke Policy

A live Transmission daemon may be used for manual validation after fake RPC
tests pass, but it is not a required closeout gate for this lane. If used,
record only safe facts: endpoint scheme/host category, profile id, operation,
hash presence, state category, and redacted error code. Do not record raw
magnet URIs, HTTP URLs, passwords, credentials, RPC session ids, or full RPC
payloads.
