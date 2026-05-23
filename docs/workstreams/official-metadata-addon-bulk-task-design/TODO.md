# Official Metadata Addon Bulk Task Design - TODO

Status: Active
Last updated: 2026-05-23

Task IDs use the `OMAB` prefix.

## M0 - Lane Opened

- [x] OMAB-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-bulk-task-design,docs/workstreams/official-metadata-addon-side-effect-writer,crates/nako-metadata-scraper/src/manifest.rs]
  Goal: Open the Bulk Metadata Scrape / Addon Task design line and record why
  the official Addon manifest must not declare a task yet.
  Validation: `cargo nextest run -p nako-metadata-scraper manifest bulk --no-fail-fast`; `git diff --check`.
  Review: Host Task declarations and routing plans are not enough to ship a
  user-visible bulk scrape task without a host-owned executor.
  Evidence: DESIGN.md, manifest test, OMASE-050 notes.
  Result: DONE 2026-05-23.
  Handoff: Continue with OMAB-020 when ready to design or implement the host
  Addon Task runtime contract.

## M1 - Host Task Runtime Contract

- [ ] OMAB-020 [owner=planner/core] [deps=OMAB-010] [scope=../nako/docs,../nako/crates/nako-addon-protocol,../nako/crates/nako-server]
  Goal: Define or implement the Nako-owned Addon Task runtime contract:
  request/response envelope, invocation route, durable job ownership,
  cancellation, retry, progress, and redaction-safe diagnostics.
  Validation: host workstream evidence and focused Nako tests.
  Review: The official Addon must not own scheduler state.
  Handoff: Continue with OMAB-030 only after the host contract is executable.

## M2 - Manifest Declaration

- [ ] OMAB-030 [owner=codex] [deps=OMAB-020] [scope=crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/manifest.example.json]
  Goal: Add `bulk-metadata-scrape` to the official Addon manifest after the
  host executor can actually invoke Addon Tasks.
  Validation: manifest tests and checked-in example manifest parity.
  Review: Required scopes must be declared at manifest level and remain
  bounded to the task.
  Handoff: Continue with OMAB-040.

## M3 - Task Endpoint And Batch Planner

- [ ] OMAB-040 [owner=codex] [deps=OMAB-020,OMAB-030] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/routes.rs,crates/nako-metadata-scraper/src/providers]
  Goal: Implement the Addon task endpoint and batch planner using provider
  suggestions plus explicit `metadata_write` and `artwork_write` side effects.
  Validation: fake transport tests for bounded batches, idempotency, failure
  summaries, and no hidden background work.
  Review: Bulk scrape must be cancellable and resumable by Nako, not by local
  sidecar state.
  Handoff: Continue with OMAB-050.

## M4 - Closeout

- [ ] OMAB-050 [owner=planner] [deps=OMAB-040] [scope=docs/workstreams/official-metadata-addon-bulk-task-design,README.md,addons/metadata-scraper]
  Goal: Update operator docs, run final gates, and close or split follow-ons.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Use verify-rust-workstream before marking this lane complete.
  Handoff: Summarize host/runtime follow-ons.
