# Task Ledger

Prefix: AVW4

## Active

- None.

## Pending

- None.

## Completed

- [x] AVW4-010 [owner=codex] [deps=-] [scope=docs/workstreams/official-metadata-addon-av-provider-wave4]
  Goal: Open the provider Wave 4 lane and record the MDCx reference boundary.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave4/WORKSTREAM.json`
  Review: Ledger keeps provider choices, GPL guardrails, and closeout gates explicit.
  Evidence: PASS on 2026-05-26: workstream docs created and JSON validated.
  Handoff: DONE. AVW4-020 is active.

- [x] AVW4-020 [owner=codex] [deps=AVW4-010] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/config.rs]
  Goal: Add reusable rendered-search AV provider base and AirAV/AVSox/XCity thin provider modules.
  Validation: `cargo nextest run -p nako-metadata-scraper airav avsox xcity --no-fail-fast`
  Review: Provider modules define identity; shared runtime owns rendering/search/detail parsing.
  Evidence: PASS on 2026-05-26: AirAV/AVSox/XCity provider tests passed.
  Handoff: DONE. AVW4-030 is complete.

- [x] AVW4-030 [owner=codex] [deps=AVW4-020] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,crates/nako-metadata-scraper/tests/live_provider_drift.rs,addons/metadata-scraper/manifest.example.json]
  Goal: Wire new providers into config, presets, manifest, external IDs, field policy, diagnostics, and drift support.
  Validation: `cargo nextest run -p nako-metadata-scraper av_drift config manifest airav avsox xcity registry --no-fail-fast`
  Review: Provider IDs/env vars are public and documented; defaults remain opt-in except presets.
  Evidence: PASS on 2026-05-26: targeted config/manifest/drift/registry/provider gate passed with 36 tests.
  Handoff: DONE. AVW4-040 is active.

- [x] AVW4-040 [owner=codex] [deps=AVW4-030] [scope=README.md,addons/metadata-scraper/README.md,docs/workstreams/official-metadata-addon-av-provider-wave4]
  Goal: Verify package gates, update docs/evidence, and close or split follow-ups.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `git diff --check`
  Review: No remaining Wave 4 decision lives only in chat.
  Evidence: PASS on 2026-05-26: full package gate passed with 230 tests; fmt, JSON, and diff hygiene passed.
  Handoff: DONE. Workstream complete.

## Follow-Up Candidates

- Add ThePornDB as a token/hash provider with explicit secret and no raw payload persistence.
- Add more MDCx long-tail providers on top of the rendered-search base when they fit GET search plus detail rendering.
- Add optional live drift case examples outside CI once the user has stable redaction-safe case IDs.
