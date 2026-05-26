# Official Metadata Addon AV Provider Wave 3 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane has completed its shared rendered AV fixture harness. `DMM`, `FC2`,
`JavDB`, `JavBus`, `JavLibrary`, and `MGStage` tests now use
`RenderedAvFixtureTransport` for browser-worker render request/response
contracts, and their provider-local fake transports were removed. The previous
scraper architecture lane remains the foundation: typed scrape outcomes, render
intent, rendered AV flow, provider quality descriptors, resolver/fusion split,
and shared side-effect writeback.

## Active Task

- Task ID: OMAV3-030
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/engine`, `crates/nako-metadata-scraper/src/providers`, `addons/metadata-scraper/README.md`, `crates/nako-metadata-scraper/README.md`
- Validation: `cargo nextest run -p nako-metadata-scraper provider_guard bulk runtime provider_execution --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm protection state is explicit and does not introduce hidden
  scheduler memory that Nako cannot reason about.
- Evidence: OMAV3-020 passed rendered AV/provider-fixture/AV tests, fmt, and
  diff hygiene gates.

## Decisions

- Start with a shared rendered AV fixture harness before adding more providers.
- Reuse `RenderedAvFixtureTransport` for rendered AV provider tests instead of
  provider-local fake transports.
- Keep all new providers disabled by default.
- Keep MDCx as reference-only; do not copy code, selectors, fixtures, regex
  tables, comments, or structure.
- Treat provider protection as explicit policy/state, not hidden scheduler
  memory.

## Blockers

- None.

## Next Recommended Action

- Execute OMAV3-030: make provider execution protection explicit through
  visible budgets, bounded cache/cooldown policy, and redaction-safe reporting.
