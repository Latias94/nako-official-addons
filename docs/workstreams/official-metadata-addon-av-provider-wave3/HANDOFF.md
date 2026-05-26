# Official Metadata Addon AV Provider Wave 3 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane has completed its shared rendered AV fixture harness, explicit
provider execution protection, Prestige, and FC2PPVDB. `DMM`, `FC2`, `JavDB`,
`JavBus`, `JavLibrary`, and `MGStage` tests now use `RenderedAvFixtureTransport`.
Provider execution now has a dedicated policy/reporting module,
request/config-visible provider budgets, bounded bulk reuse/cache and cooldown
resume state, and boolean-only browser render proxy/session diagnostics.
`Prestige` is a disabled-by-default official JSON API provider for censored AV
search/direct lookup with `prestige_id`, `prestige_url`, and `av_number`
external IDs plus direct proxy support. `FC2PPVDB` is a disabled-by-default FC2
long-tail fallback using deterministic article URLs and browser-worker rendered
HTML, with `fc2ppvdb_id`, `fc2ppvdb_url`, and `av_number` external IDs. The
previous scraper architecture lane remains the foundation: typed scrape
outcomes, render intent, rendered AV flow, provider quality descriptors,
resolver/fusion split, and shared side-effect writeback.

## Active Task

- Task ID: OMAV3-060
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers`, `crates/nako-metadata-scraper/src/config.rs`, `crates/nako-metadata-scraper/src/manifest.rs`, `addons/metadata-scraper/README.md`, `crates/nako-metadata-scraper/README.md`
- Validation: `cargo nextest run -p nako-metadata-scraper caribbean 1pondo 10musume av config registry manifest --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm each uncensored provider has independent fixtures, route
  gates, external IDs, field quality descriptors, and docs.
- Evidence: OMAV3-050 passed fc2/AV/config/registry/manifest tests and
  dedicated FC2PPVDB provider fixture tests for FC2 route search, direct ID
  lookup, direct URL lookup, route skip, mapping, artwork, and trailer
  separation.

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
- FC2PPVDB was selected over FC2Hub/FC2Club because it has deterministic article
  URLs and richer long-tail release/runtime/actor/tag/seller/trailer fallback
  value for this slice; FC2Hub and FC2Club remain follow-up candidates if later
  live drift evidence justifies them.

## Blockers

- None.

## Next Recommended Action

- Execute OMAV3-060: add Caribbeancom, 1Pondo, and 10Musume as official
  uncensored providers, or split the trio if one site needs a separate lane.
