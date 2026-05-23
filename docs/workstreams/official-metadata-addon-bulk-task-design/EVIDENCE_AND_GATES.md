# Official Metadata Addon Bulk Task Design - Evidence And Gates

Status: Active
Last updated: 2026-05-23

## Gates

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-metadata-scraper manifest bulk --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`

Focused host-side gates must be recorded in the relevant `../nako`
workstream before OMAB-030 or later implementation work starts.

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-23 | OMAB-010 | Opened design line after OMASE-050 host assessment; added manifest test that keeps `tasks` empty until host execution exists; `cargo nextest run -p nako-metadata-scraper manifest bulk --no-fail-fast`; `python -m json.tool docs/workstreams/official-metadata-addon-bulk-task-design/WORKSTREAM.json`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check` | Pass. |

## Host Assessment

- `AddonTaskDeclaration` exists in `nako-addon-protocol`.
- Nako Admin validates task declarations during registration.
- Nako can build Addon routing plans targeting `AddonTaskJob`.
- Nako docs and runtime handoffs still identify the full Addon Task
  scheduler/runtime as deferred.
- No generic task invocation route or task progress/outcome contract exists
  for the official Addon to implement against today.

## Safety Requirements

- Do not add a hidden sidecar scheduler.
- Do not declare a user-visible bulk scrape task before Nako can execute it.
- Do not bypass `metadata_write` and `artwork_write` side-effect authority.
- Do not serialize Addon Tokens, provider tokens, raw source locators, local
  paths, or raw provider payloads in progress diagnostics.
