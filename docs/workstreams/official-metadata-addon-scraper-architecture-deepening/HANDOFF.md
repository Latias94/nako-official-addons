# Official Metadata Addon Scraper Architecture Deepening - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane is newly opened. The previous AV native writeback/provider wave 2 lane
is closed. The metadata scraper package currently passes full tests from the
previous closeout, and this lane starts from a clean official-addons worktree.

The architecture review identified six deepening candidates. This workstream
will solve all six unless a task reveals that a candidate should be split into a
separate durable lane.

## Active Task

- Task ID: OMSAD-020
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/engine/runtime.rs`, `crates/nako-metadata-scraper/src/engine/response.rs`, `crates/nako-metadata-scraper/src/engine/bulk.rs`, `crates/nako-metadata-scraper/src/engine/orchestration.rs`
- Validation: `cargo nextest run -p nako-metadata-scraper bulk runtime metadata_endpoint --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm public response rendering is a projection and bulk no longer
  parses provider execution from `AddonResourceResponse` JSON.
- Evidence:

## Decisions

- Use fearless refactoring: compatibility shims are not required when a cleaner
  Interface or schema-versioned output is better.
- Start with typed scrape outcome because bulk JSON round-trip is the highest
  leverage shallow Seam.
- Keep provider-specific site quirks local to provider adapters.
- Keep browser-worker as the browser/proxy/session/wait owner.
- Do not implement Nako core refresh policy in this addon lane.

## Blockers

- None.

## Next Recommended Action

- Execute OMSAD-020 with a narrow red/green refactor around `bulk.rs`,
  `runtime.rs`, and `response.rs`.
