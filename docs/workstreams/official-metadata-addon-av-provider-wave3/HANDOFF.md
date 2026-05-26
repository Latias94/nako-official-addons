# Official Metadata Addon AV Provider Wave 3 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane has completed its shared rendered AV fixture harness, explicit
provider execution protection, and first wave 3 provider. `DMM`, `FC2`,
`JavDB`, `JavBus`, `JavLibrary`, and `MGStage` tests now use
`RenderedAvFixtureTransport`. Provider execution now has a dedicated
policy/reporting module, request/config-visible provider budgets, bounded bulk
reuse/cache and cooldown resume state, and boolean-only browser render
proxy/session diagnostics. `Prestige` is now a disabled-by-default official JSON
API provider for censored AV search/direct lookup with `prestige_id`,
`prestige_url`, and `av_number` external IDs plus direct proxy support. The
previous scraper architecture lane remains the foundation: typed scrape
outcomes, render intent, rendered AV flow, provider quality descriptors,
resolver/fusion split, and shared side-effect writeback.

## Active Task

- Task ID: OMAV3-050
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers`, `crates/nako-metadata-scraper/src/config.rs`, `crates/nako-metadata-scraper/src/manifest.rs`, `addons/metadata-scraper/README.md`, `crates/nako-metadata-scraper/README.md`
- Validation: `cargo nextest run -p nako-metadata-scraper fc2 av config registry manifest --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm the selected FC2 long-tail provider does not duplicate the
  existing FC2 official source and improves fallback coverage.
- Evidence: OMAV3-040 passed prestige/config/registry/manifest/AV tests and
  dedicated Prestige provider fixture tests for search, direct lookup, URL
  lookup, route skip, mapping, artwork/trailer, and proxy runtime config.

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

- Execute OMAV3-050: evaluate FC2PPVDB, FC2Hub, and FC2Club for testable
  fallback value, then add one FC2 long-tail provider if it improves coverage
  without duplicating the existing FC2 official source.
