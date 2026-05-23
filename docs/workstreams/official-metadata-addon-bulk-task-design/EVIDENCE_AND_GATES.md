# Official Metadata Addon Bulk Task Design - Evidence And Gates

Status: Complete
Last updated: 2026-05-24

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
| 2026-05-24 | OMAB-020/030/040/050 | Closed current addon-side bulk task lane as host-runtime-blocked/deferred. The official addon must keep `tasks: []` until `../nako` owns generic Addon Task invocation, durable records, cancellation, retry, progress, and redaction-safe outcomes. | Pass for current-release boundary; future implementation deferred by design. |

## 2026-05-24 Closeout Evidence

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper manifest bulk --no-fail-fast`: PASS, 6 tests passed
  and 64 skipped. Proves the runtime manifest and checked-in example manifest keep `tasks: []`
  until host task execution exists.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 70 tests passed. Proves the
  package-level metadata scraper surface after bulk-task closeout.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 70 tests passed. Proves the full workspace
  test suite remains green for the current release scope.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

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
