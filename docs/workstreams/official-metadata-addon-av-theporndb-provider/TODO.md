# Task Ledger

Prefix: AVTPDB

## Active

- None.

## Pending

- None.

## Completed

- [x] AVTPDB-010 [owner=codex] [deps=-] [scope=docs/workstreams/official-metadata-addon-av-theporndb-provider]
  Goal: Open the ThePornDB provider lane with GPL guardrails, API assumptions, gates, and handoff state.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-theporndb-provider/WORKSTREAM.json`
  Review: Confirmed this is a token/API provider lane, not a rendered-page provider wave.
  Evidence: PASS, 2026-05-26.
  Handoff: Complete.

- [x] AVTPDB-020 [owner=codex] [deps=AVTPDB-010] [scope=crates/nako-metadata-scraper/src/providers/theporndb.rs,crates/nako-metadata-scraper/src/providers/mod.rs]
  Goal: Add ThePornDB HTTP provider with token auth, scene search/direct lookup, serde mapping, artwork, field facts, and synthetic API tests.
  Validation: `cargo nextest run -p nako-metadata-scraper theporndb --no-fail-fast`
  Review: Token must never appear in diagnostics, test names, failure messages, or committed fixtures.
  Evidence: PASS, 2026-05-26.
  Handoff: Complete.

- [x] AVTPDB-030 [owner=codex] [deps=AVTPDB-020] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,crates/nako-metadata-scraper/tests/live_provider_drift.rs,addons/metadata-scraper/manifest.example.json]
  Goal: Wire provider config, presets, secret reference, diagnostics, external-id aliases, field policy, manifest schema/example, and live drift provider lists.
  Validation: `cargo nextest run -p nako-metadata-scraper config registry manifest av_provider_preset field_policy av_drift --no-fail-fast`
  Review: Enabled provider without token must be unavailable rather than silently built.
  Evidence: PASS, 2026-05-26.
  Handoff: Complete.

- [x] AVTPDB-040 [owner=codex] [deps=AVTPDB-030] [scope=README.md,addons/metadata-scraper/README.md,docs/workstreams/official-metadata-addon-av-theporndb-provider]
  Goal: Document token/proxy/preset behavior, record verification evidence, and close or split follow-ups.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `git diff --check`
  Review: Remaining hash or movie-route work must be explicit follow-up, not hidden in chat.
  Evidence: PASS, 2026-05-26.
  Handoff: Complete.

## Follow-Up Candidates

- Add explicit `file_hash` query facts and route them to `/scenes/hash/{hash}` and `/movies/hash/{hash}`.
- Add movie-route search and direct movie lookup once the provider contract can distinguish scene and movie intent.
- Add route-specific western provider budgets when provider selection becomes route-preset aware.
