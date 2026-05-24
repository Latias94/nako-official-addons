# Official Metadata Addon Bulk Task Design - TODO

Status: Complete
Last updated: 2026-05-24

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

- [x] OMAB-020 [owner=planner/core] [deps=OMAB-010] [scope=../nako/docs,../nako/crates/nako-addon-protocol,../nako/crates/nako-server]
  Goal: Define or implement the Nako-owned Addon Task runtime contract:
  request/response envelope, invocation route, durable job ownership,
  cancellation, retry, progress, and redaction-safe diagnostics.
  Validation: host workstream evidence and focused Nako tests.
  Review: The official Addon must not own scheduler state.
  Result: DEFERRED 2026-05-24.
  Handoff: Host runtime work is outside this repository and must continue in
  `../nako` before addon task declaration or endpoint implementation resumes.

## M2 - Manifest Declaration

- [x] OMAB-030 [owner=codex] [deps=OMAB-020] [scope=crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/manifest.example.json]
  Goal: Add `bulk-metadata-scrape` to the official Addon manifest after the
  host executor can actually invoke Addon Tasks.
  Validation: manifest tests and checked-in example manifest parity.
  Review: Required scopes must be declared at manifest level and remain
  bounded to the task.
  Result: DEFERRED 2026-05-24.
  Handoff: Correct current-release behavior is to keep `tasks: []`; reopen a
  new implementation lane after the host task runtime is executable.

## M3 - Task Endpoint And Batch Planner

- [x] OMAB-040 [owner=codex] [deps=OMAB-020,OMAB-030] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/routes.rs,crates/nako-metadata-scraper/src/providers]
  Goal: Implement the Addon task endpoint and batch planner using provider
  suggestions plus explicit `metadata_write` and `artwork_write` side effects.
  Validation: fake transport tests for bounded batches, idempotency, failure
  summaries, and no hidden background work.
  Review: Bulk scrape must be cancellable and resumable by Nako, not by local
  sidecar state.
  Result: DEFERRED 2026-05-24.
  Handoff: Do not add a hidden sidecar scheduler. Implement only after host
  task invocation/progress/outcome ownership exists.

## M4 - Closeout

- [x] OMAB-050 [owner=planner] [deps=OMAB-040] [scope=docs/workstreams/official-metadata-addon-bulk-task-design,README.md,addons/metadata-scraper]
  Goal: Update operator docs, run final gates, and close or split follow-ons.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Use verify-rust-workstream before marking this lane complete.
  Result: DONE 2026-05-24.
  Handoff: Lane closed for the current release with bulk task implementation
  explicitly deferred to a future host-runtime-backed workstream.

## M5 - Reopened Addon Implementation

- [x] OMAB-060 [owner=codex] [deps=OMAB-050] [scope=crates/nako-metadata-scraper/src/engine/bulk.rs,crates/nako-metadata-scraper/src/routes.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/manifest.example.json]
  Goal: Reopen the lane now that the host Addon Task runtime exists and wire
  the official addon manifest, task endpoint, and bounded batch planner for
  `bulk-metadata-scrape`.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper manifest bulk --no-fail-fast`; `cargo nextest run --workspace --no-fail-fast`; `cargo clippy -p nako-metadata-scraper --all-targets -- -D warnings`; `git diff --check`.
  Review: The addon-side task DTOs mirror the host-owned envelope, keep payloads redaction-safe, and avoid a hidden scheduler or direct DB access.
  Evidence: engine bulk module, manifest, route tests, example manifest.
  Result: DONE 2026-05-24.
  Handoff: Continue with OMAB-070 verification, docs, and closeout.

- [x] OMAB-070 [owner=planner] [deps=OMAB-060] [scope=README.md,addons/metadata-scraper,docs/workstreams/official-metadata-addon-bulk-task-design]
  Goal: Refresh operator docs, record fresh gates, and close or split remaining
  follow-ons after the reopened bulk task implementation lands.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper manifest bulk --no-fail-fast`; `cargo nextest run --workspace --no-fail-fast`; `cargo clippy -p nako-metadata-scraper --all-targets -- -D warnings`; `git diff --check`.
  Review: Closeout evidence is fresh and the shipped behavior matches the docs.
  Evidence: README.md, addons/metadata-scraper/README.md, addons/metadata-scraper/manifest.example.json, docs/workstreams/official-metadata-addon-bulk-task-design/EVIDENCE_AND_GATES.md, docs/workstreams/official-metadata-addon-bulk-task-design/HANDOFF.md, docs/workstreams/official-metadata-addon-bulk-task-design/WORKSTREAM.json
  Handoff: Lane closed for the current release.
