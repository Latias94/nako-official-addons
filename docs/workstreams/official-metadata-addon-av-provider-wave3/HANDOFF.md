# Official Metadata Addon AV Provider Wave 3 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane has completed its shared rendered AV fixture harness, explicit
provider execution protection, Prestige, FC2PPVDB, and the official uncensored
provider trio. `DMM`, `FC2`, `JavDB`, `JavBus`, `JavLibrary`, and `MGStage`
tests now use `RenderedAvFixtureTransport`; Caribbean, 1Pondo, and 10Musume use
a shared deep official-uncensored implementation with independent site
descriptors and synthetic rendered HTML fixtures. Provider execution now has a
dedicated policy/reporting module, request/config-visible provider budgets,
bounded bulk reuse/cache and cooldown resume state, and boolean-only browser
render proxy/session diagnostics. `Prestige` is a disabled-by-default official
JSON API provider for censored AV search/direct lookup with `prestige_id`,
`prestige_url`, and `av_number` external IDs plus direct proxy support.
`FC2PPVDB` is a disabled-by-default FC2 long-tail fallback using deterministic
article URLs and browser-worker rendered HTML, with `fc2ppvdb_id`,
`fc2ppvdb_url`, and `av_number` external IDs. `Caribbean`, `1Pondo`, and
`10Musume` are disabled-by-default official uncensored providers for date-style
IDs and expose provider-specific ID/URL aliases plus `av_number`. The previous
scraper architecture lane remains the foundation: typed scrape outcomes,
render intent, rendered AV flow, provider quality descriptors, resolver/fusion
split, and shared side-effect writeback.

## Active Task

- Task ID: OMAV3-070
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers`, `addons/metadata-scraper/README.md`, `crates/nako-metadata-scraper/README.md`, `docs/workstreams/official-metadata-addon-av-provider-wave3`
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `npm --prefix addons/browser-worker test`; `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave3/WORKSTREAM.json`; `git diff --check`
- Status: READY
- Review: Confirm no provider or protection work remains hidden in handoff
  notes and decide whether remaining provider candidates should be follow-up
  lanes rather than blockers.
- Evidence: OMAV3-060 passed 63 caribbean/1pondo/10musume/AV/config/registry/
  manifest tests and fmt check; the trio covers route gates, direct ID/URL
  lookup, mapping, artwork/trailer facts, aliases, manifest schema, and field
  descriptors.

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
- Caribbean, 1Pondo, and 10Musume share the same implementation because their
  official pages expose similar rendered detail structure and date-style IDs;
  the site descriptors keep URL shape, provider IDs, env vars, and field quality
  independent.

## Blockers

- None.

## Next Recommended Action

- Execute OMAV3-070: run full package validation, browser-worker validation,
  workstream JSON/diff hygiene, then close or split any remaining provider
  candidates.
