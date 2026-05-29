# Official Metadata Addon Side Effect Writer - TODO

Status: Complete
Last updated: 2026-05-23

Task IDs use the `OMASE` prefix.

## M0 - Scope

- [x] OMASE-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-side-effect-writer]
  Goal: Open the workstream for the 1/2/3 next-step plan: side-effect writer,
  artwork candidate flow, and Bulk Metadata Scrape / Addon Task evaluation.
  Validation: Workstream docs agree.
  Evidence: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, HANDOFF.md.
  Result: DONE 2026-05-23.
  Handoff: Continue with OMASE-020.

## M1 - Outbound Nako Runtime Client

- [x] OMASE-020 [owner=codex] [deps=OMASE-010] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/nako_runtime.rs,crates/nako-metadata-scraper/src/lib.rs]
  Goal: Add disabled-by-default Nako runtime configuration and a testable
  outbound client for `/addon/v1/access-check` and `/addon/v1/side-effects`.
  Validation: `cargo nextest run -p nako-metadata-scraper nako_runtime config --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: The Addon Token must only appear in the Authorization header, never
  in request body, diagnostics, logs, or response payloads.
  Evidence: fake transport tests and config tests.
  Result: DONE 2026-05-23. Added disabled-by-default Nako runtime config,
  a fake-transport-testable outbound client, and redaction-safe request
  validation for access-check and side-effect submission.
  Handoff: Continue with OMASE-030.

## M2 - Explicit Metadata Write Submission

- [x] OMASE-030 [owner=codex] [deps=OMASE-020] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/routes.rs,addons/metadata-scraper/smoke.local.ps1]
  Goal: Allow an explicit metadata request payload to submit the selected
  candidate patch as a Nako `metadata_write` Addon Side Effect when runtime
  writes are configured and enabled.
  Validation: `cargo nextest run -p nako-metadata-scraper side_effect metadata --no-fail-fast`; direct sidecar smoke still passes; `git diff --check`.
  Review: Ordinary `/metadata` calls must remain suggestion-only. Missing
  runtime config must produce a redaction-safe skipped outcome, not a mutation.
  Evidence: runtime tests and smoke docs/script changes.
  Result: DONE 2026-05-23. Added explicit `payload.writeback` handling for
  selected-candidate `metadata_write`, access-check preflight, redaction-safe
  skipped/failed summaries, smoke script support, and docs.
  Handoff: Continue with OMASE-040.

## M3 - Typed Artwork Candidate Flow

- [x] OMASE-040 [owner=codex] [deps=OMASE-020,OMASE-030] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/nako_runtime.rs,addons/metadata-scraper]
  Goal: Promote provider image facts from metadata tags into typed artwork
  candidates and support explicit `artwork_write` Addon Side Effect submission.
  Validation: `cargo nextest run -p nako-metadata-scraper artwork side_effect --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Provider URLs may be submitted to Nako as artwork candidates only
  through the Nako-owned `artwork_write` path. They must not become public
  artwork or selected artwork inside the Addon.
  Evidence: TMDB/Bangumi provider tests and runtime tests.
  Result: DONE 2026-05-23. Promoted provider image facts into typed artwork
  candidates, added explicit `artwork_write` submission through the runtime
  client, and verified the full artwork side-effect flow with focused and
  package-level tests.
  Handoff: Continue with OMASE-050.

## M4 - Bulk Metadata Scrape / Addon Task Evaluation

- [x] OMASE-050 [owner=planner] [deps=OMASE-030] [scope=docs/workstreams/official-metadata-addon-side-effect-writer,crates/nako-metadata-scraper/src/manifest.rs]
  Goal: Evaluate whether the current Nako host exposes enough Addon Task seam
  for Bulk Metadata Scrape. Declare or defer task manifest changes without
  adding hidden Addon background work.
  Validation: docs and manifest tests; `git diff --check`.
  Review: If host Addon Task execution is not ready, split a follow-on instead
  of adding an unowned scheduler.
  Evidence: EVIDENCE_AND_GATES.md and HANDOFF.md.
  Result: DONE 2026-05-23. Nako currently supports Addon Task declarations
  and routing plans, but not a generic task scheduler/invoker. The official
  addon manifest keeps `tasks: []`, and follow-on design lane
  `official-metadata-addon-bulk-task-design` is open.
  Handoff: Continue with OMASE-060.

## M5 - Docs And Closeout

- [x] OMASE-060 [owner=planner] [deps=OMASE-040,OMASE-050] [scope=README.md,addons/metadata-scraper,docs/workstreams/official-metadata-addon-side-effect-writer]
  Goal: Update operator docs, run final gates, and close or split follow-ons.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Use verify-rust-workstream before marking complete.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Result: DONE 2026-05-23. Operator docs now describe explicit metadata and
  artwork writeback plus deferred Bulk Metadata Scrape task behavior, and the
  side-effect writer lane is closed with the bulk-task follow-on split into the
  dedicated OMAB design lane.
  Handoff: Summarize follow-ons for Addon Task host support, provider breadth,
  and managed artwork ingest workers.
