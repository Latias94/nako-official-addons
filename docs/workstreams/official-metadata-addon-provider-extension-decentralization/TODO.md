# Official Metadata Addon Provider Extension Decentralization - TODO

Status: Active
Last updated: 2026-05-25

## M0 - Scope And Gates

- [x] OMAPED-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-extension-decentralization]
  Goal: Freeze the provider extension decentralization target state, task order, non-goals, and gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Review: Confirm this lane does not reopen release/smoke work or the already closed architecture-deepening lane.
  Evidence: `python -m json.tool docs/workstreams/official-metadata-addon-provider-extension-decentralization/WORKSTREAM.json`
  Handoff: DONE. Workstream opened and validated; start OMAPED-020.

## M1 - Provider Config Decentralization

- [x] OMAPED-020 [owner=codex] [deps=OMAPED-010] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/manifest.rs]
  Goal: Replace the provider config optional-field matrix with a typed provider config Interface that prevents invalid rows and moves provider-local config structs closer to provider adapters.
  Validation: cargo nextest run -p nako-metadata-scraper config manifest provider registry --no-fail-fast
  Review: Confirm public env vars, manifest defaults, secret references, and provider enablement remain compatible.
  Evidence: `cargo nextest run -p nako-metadata-scraper config manifest provider registry --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Handoff: DONE. `ProviderConfig` now uses typed provider config variants and provider-local config structs live in provider modules while env vars, manifest defaults, secret references, and provider enablement remain compatible.

## M2 - Provider-Owned External ID Aliases

- [x] OMAPED-030 [owner=codex] [deps=OMAPED-020] [scope=crates/nako-metadata-scraper/src/engine/query.rs,crates/nako-metadata-scraper/src/engine/runtime.rs,crates/nako-metadata-scraper/src/providers/registry.rs,crates/nako-metadata-scraper/src/providers]
  Goal: Move top-level external ID alias declarations and known numeric validation into provider-owned descriptors or a provider extension seam.
  Validation: cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker --no-fail-fast
  Review: Confirm existing payload aliases still parse and query parsing does not import provider implementation details directly.
  Evidence: `cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Handoff: DONE. Query parsing now receives descriptor-provided external ID aliases from the provider registry; adding a top-level provider alias no longer requires editing query parsing logic.

## M3 - Rendered Page Support Semantics

- [ ] OMAPED-040 [owner=codex] [deps=OMAPED-020] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/providers/rendered_page.rs,crates/nako-metadata-scraper/src/providers/browser_worker.rs,crates/nako-metadata-scraper/src/providers/douban.rs,crates/nako-metadata-scraper/src/providers/douban]
  Goal: Make rendered-page support config and naming explicit for Douban and browser_worker while preserving existing browser-worker env vars.
  Validation: cargo nextest run -p nako-metadata-scraper browser_worker douban rendered --no-fail-fast
  Review: Confirm Douban is represented as a browser-rendered provider and `browser_worker` remains a real default-off metadata provider only for explicit rendered-page URL extraction.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Shared rendered-page support is ready for another browser-rendered provider.

## M4 - Cleanup And Integration

- [ ] OMAPED-050 [owner=codex] [deps=OMAPED-030,OMAPED-040] [scope=crates/nako-metadata-scraper,addons/metadata-scraper,docs/workstreams/official-metadata-addon-provider-extension-decentralization]
  Goal: Clean stale tests/docs discovered during the refactor and run the full metadata scraper package gate.
  Validation: cargo nextest run -p nako-metadata-scraper --no-fail-fast
  Review: review-workstream before accepting completion.
  Evidence: EVIDENCE_AND_GATES.md
  Handoff: Prepare closeout.

## M5 - Closeout

- [ ] OMAPED-060 [owner=planner] [deps=OMAPED-050] [scope=docs/workstreams/official-metadata-addon-provider-extension-decentralization]
  Goal: Close the lane or split concrete follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Summarize remaining risks in HANDOFF.md.
