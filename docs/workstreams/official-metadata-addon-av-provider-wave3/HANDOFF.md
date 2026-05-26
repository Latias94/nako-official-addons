# Official Metadata Addon AV Provider Wave 3 - Handoff

Status: Complete
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

## Closeout

- Task ID: OMAV3-070
- Status: DONE
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed
  with 220 tests; `npm --prefix addons/browser-worker test` passed with 4
  tests; `cargo fmt -p nako-metadata-scraper -- --check`,
  `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave3/WORKSTREAM.json`,
  and `git diff --check` passed.
- Review: No provider or protection work remains hidden in handoff notes. The
  lane is complete; future provider breadth should be new scope.

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

## Follow-Ups

- ThePornDB, Jav321, region-specific fallbacks, and additional FC2 sources can
  be opened as new provider-breadth lanes if needed.
- Nako core refresh/locked-field/local metadata/local artwork priority remains
  outside this provider lane.
- User-facing review UI, NFO/rename, and actor-image workflows remain outside
  this provider lane.
