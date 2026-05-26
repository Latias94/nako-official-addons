# Task Ledger

Prefix: OMAVM

## Active

- None.

## Completed

- [x] OMAVM-010 [owner=codex] [deps=-] [scope=docs/workstreams/official-metadata-addon-av-mdcx-parity]
  Goal: Open the durable AV MDCx parity lane with scope, guardrails, tasks, gates, and handoff state.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-mdcx-parity/WORKSTREAM.json`
  Review: Confirmed this is a new phase rather than reopening the completed MDCx-style and AV policy workstreams.
  Evidence: PASS on 2026-05-26.
  Handoff: DONE. Workstream docs are present and JSON-valid.

- [x] OMAVM-020 [owner=codex] [deps=OMAVM-010] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/providers/{javdb,dmm,fc2}]
  Goal: Add structured AV candidate facts for actors, all actors, directors, series, studio, publisher, maker, label, wanted count, thumb, trailer, and extra fanart; expose them in responses and field-source evidence.
  Validation: `cargo nextest run -p nako-metadata-scraper av field_policy resolver javdb dmm fc2 --no-fail-fast`
  Review: Confirmed `AddonMetadataPatch` remains portable and existing tag compatibility is preserved.
  Evidence: PASS on 2026-05-26; 31 related tests passed before provider expansion, then 34 with JavBus included.
  Handoff: DONE. AV facts are response-side and field-policy-aware.

- [x] OMAVM-030 [owner=codex] [deps=OMAVM-010] [scope=addons/browser-worker/src,addons/browser-worker/test,crates/nako-metadata-scraper/src/providers/rendered_page.rs]
  Goal: Add browser-worker proxy/session/wait request contract and env-based proxy configuration with redaction-safe health output.
  Validation: `npm --prefix addons/browser-worker test`; `cargo nextest run -p nako-metadata-scraper rendered browser_worker javdb dmm fc2 --no-fail-fast`
  Review: Confirmed proxy URLs are not echoed in health diagnostics.
  Evidence: PASS on 2026-05-26; browser-worker 4 tests passed and rendered-page Rust contract 18 tests passed.
  Handoff: DONE. Browser-worker owns proxy/wait/session intent.

- [x] OMAVM-040 [owner=codex] [deps=OMAVM-020,OMAVM-030] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper]
  Goal: Add the first disabled-by-default MDCx-inspired AV fallback provider and wire it through config, registry, manifest, docs, and synthetic rendered-HTML tests.
  Validation: `cargo nextest run -p nako-metadata-scraper config registry manifest av --no-fail-fast`
  Review: Confirmed route support, external IDs, and selectors are independently implemented.
  Evidence: PASS on 2026-05-26; 48 tests passed.
  Handoff: DONE. `javbus` provider is wired and disabled by default.

- [x] OMAVM-050 [owner=codex] [deps=OMAVM-020,OMAVM-030,OMAVM-040] [scope=docs/workstreams/official-metadata-addon-av-mdcx-parity,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md,addons/browser-worker/README.md]
  Goal: Verify the lane, document shipped behavior and protocol limits, record evidence, and close or split remaining provider parity.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `npm --prefix addons/browser-worker test`; `python -m json.tool docs/workstreams/official-metadata-addon-av-mdcx-parity/WORKSTREAM.json`; `python -m json.tool addons/metadata-scraper/manifest.example.json`; `git diff --check`
  Review: Confirmed follow-ups are explicit and MDCx remained reference-only.
  Evidence: PASS on 2026-05-26; 183 Rust tests passed with 2 skipped, 4 browser-worker tests passed, JSON checks and diff hygiene passed.
  Handoff: DONE. Workstream closed; provider/protocol parity remains split follow-up work.

## Follow-Up Candidates

- Nako protocol/server companion lane for addon writeback of credits, studios,
  collections, ratings, external IDs, and thumbnail images.
- Provider wave 2: JavLibrary, FC2PPVDB/FC2Hub/FC2Club, MGStage, Prestige,
  ThePornDB, Jav321, and region-specific fallbacks.
- Batch scrape controls: per-provider concurrency, cooldowns, permanent failure
  suppression, and manual retry classes.
