# Official Metadata Addon Scraper Architecture Deepening - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane is active. The previous AV native writeback/provider wave 2 lane is
closed. OMSAD-020 is complete: runtime now builds a typed
`MetadataScrapeOutcome`, response rendering projects from it, and bulk fresh
scrape consumes typed AV facts, provider execution, failure reason, and provider
suppression facts without parsing public response JSON.

The architecture review identified six deepening candidates. This workstream
will solve all six unless a task reveals that a candidate should be split into a
separate durable lane.

## Active Task

- Task ID: OMSAD-030
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers/rendered_page.rs`, `addons/browser-worker/src/app.mjs`, `addons/browser-worker/src/extract.mjs`, `addons/browser-worker/test`
- Validation: `cargo nextest run -p nako-metadata-scraper rendered_page browser_worker douban javbus javlibrary mgstage --no-fail-fast`; `npm --prefix addons/browser-worker test`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm providers can declare render behavior without duplicating
  browser-worker payload assembly.
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

- Execute OMSAD-030 by adding Render Intent around `RenderedPageRuntime`, then
  thread wait/proxy/session payload coverage through Rust provider tests and
  browser-worker validation.
