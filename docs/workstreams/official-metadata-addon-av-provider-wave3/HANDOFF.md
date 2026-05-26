# Official Metadata Addon AV Provider Wave 3 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane has completed its shared rendered AV fixture harness and explicit
provider execution protection. `DMM`, `FC2`, `JavDB`, `JavBus`, `JavLibrary`,
and `MGStage` tests now use `RenderedAvFixtureTransport`. Provider execution
now has a dedicated policy/reporting module, request/config-visible provider
budgets, bounded bulk reuse/cache and cooldown resume state, and boolean-only
browser render proxy/session diagnostics. The previous scraper architecture
lane remains the foundation: typed scrape outcomes, render intent, rendered AV
flow, provider quality descriptors, resolver/fusion split, and shared
side-effect writeback.

## Active Task

- Task ID: OMAV3-040
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers`, `crates/nako-metadata-scraper/src/config.rs`, `crates/nako-metadata-scraper/src/manifest.rs`, `addons/metadata-scraper/README.md`, `crates/nako-metadata-scraper/README.md`
- Validation: `cargo nextest run -p nako-metadata-scraper prestige config registry manifest av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm the provider is disabled by default, emits declared external
  IDs, supports only correct AV routes, and uses independent parser fixtures.
- Evidence: OMAV3-030 passed provider_guard/bulk/runtime/provider_execution
  tests, full package nextest, fmt, and diff hygiene gates.

## Decisions

- Start with a shared rendered AV fixture harness before adding more providers.
- Reuse `RenderedAvFixtureTransport` for rendered AV provider tests instead of
  provider-local fake transports.
- Keep all new providers disabled by default.
- Keep MDCx as reference-only; do not copy code, selectors, fixtures, regex
  tables, comments, or structure.
- Treat provider protection as explicit policy/state, not hidden scheduler
  memory.
- Keep provider budgets and bulk cache/cooldown state visible in request/task
  payloads and outputs; do not persist hidden sidecar scheduler state.
- Browser render proxy/session diagnostics are boolean-only and must not expose
  proxy URLs, credentials, or session key values.

## Blockers

- None.

## Next Recommended Action

- Execute OMAV3-040: add the first wave 3 AV provider, starting with Prestige
  if its synthetic fixture and route behavior are stable.
