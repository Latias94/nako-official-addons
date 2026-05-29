# Official Metadata Addon Provider Architecture Deepening - TODO

Status: Complete
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

- [x] OMAPAD-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-architecture-deepening]
  Goal: Freeze the five-refactor target state, task order, non-goals, and evidence gates.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: `python -m json.tool docs/workstreams/official-metadata-addon-provider-architecture-deepening/WORKSTREAM.json > $null`
  Handoff: DONE. Workstream docs are opened and agree on the five-refactor target state.

## M1 - Provider Descriptor And Assembly Depth

- [x] OMAPAD-020 [owner=codex] [deps=OMAPAD-010] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/manifest.rs]
  Goal: Move provider config/default/secret/capability/build facts toward provider-owned descriptors so adding a provider no longer requires broad central config edits.
  Validation: cargo nextest run -p nako-metadata-scraper provider registry config addon_manifest --no-fail-fast
  Review: Check that provider descriptors own provider-local facts and that public manifest defaults remain compatible.
  Evidence: `cargo nextest run -p nako-metadata-scraper provider registry config addon_manifest --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Handoff: DONE. Provider catalog entries now own default enablement, enablement env vars, provider config loading, and provider-local proxy facts.

- [x] OMAPAD-030 [owner=codex] [deps=OMAPAD-020] [scope=crates/nako-metadata-scraper/src/providers/registry.rs,crates/nako-metadata-scraper/src/routes.rs]
  Goal: Assemble providers once and derive ready adapters, diagnostics, and route health facts from the same provider assembly.
  Validation: cargo nextest run -p nako-metadata-scraper provider health_endpoint diagnostics --no-fail-fast
  Review: Confirm routes remain HTTP adapters and no provider-specific health facts leak from Config into routes.
  Evidence: `cargo nextest run -p nako-metadata-scraper provider health_endpoint diagnostics --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Handoff: DONE. ProviderRegistry now exposes one assembly result for ready adapters, diagnostics, and provider-owned network policy facts; routes consume that assembly.

## M2 - Shared Provider Search Policy And Outcomes

- [x] OMAPAD-040 [owner=codex] [deps=OMAPAD-030] [scope=crates/nako-metadata-scraper/src/providers/tmdb,crates/nako-metadata-scraper/src/providers/bangumi,crates/nako-metadata-scraper/src/engine/ranking.rs]
  Goal: Extract a shared search-enrichment policy Module for direct ID attempts, title variants, dedupe, relevance budget, partial-search preservation, and degraded fallback while keeping raw provider parsing provider-local.
  Validation: cargo nextest run -p nako-metadata-scraper tmdb bangumi relevance partial degraded --no-fail-fast
  Review: Confirm the new Module hides policy complexity without pulling raw provider schemas across the Seam.
  Evidence: `cargo nextest run -p nako-metadata-scraper tmdb bangumi relevance partial degraded --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Handoff: DONE. `providers/search_policy.rs` now owns direct lookup, title-variant search, dedupe, ranking-budget selection, partial-search preservation, and degraded fallback orchestration; TMDB/Bangumi keep raw parsing, endpoint calls, ID extraction, and mapping provider-local.

- [x] OMAPAD-050 [owner=codex] [deps=OMAPAD-040] [scope=crates/nako-metadata-scraper/src/engine/ranking.rs,crates/nako-metadata-scraper/src/providers]
  Goal: Replace provider-local diagnostic prose with typed provider outcome facts rendered through one redaction-safe Module.
  Validation: cargo nextest run -p nako-metadata-scraper provider_note redaction ranking tmdb bangumi douban --no-fail-fast
  Review: Confirm public payload compatibility or document any intentional schema change.
  Evidence: `cargo nextest run -p nako-metadata-scraper provider_note redaction ranking tmdb bangumi douban --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Handoff: DONE. `engine/outcome.rs` now owns redaction-safe provider note rendering from typed `ProviderOutcome` facts; providers emit outcomes instead of provider-local diagnostic prose, with legacy `provider_note` preserved only as a compatibility fallback.

## M3 - Rendered Page Support Seam

- [x] OMAPAD-060 [owner=codex] [deps=OMAPAD-030] [scope=crates/nako-metadata-scraper/src/providers/browser_worker.rs,crates/nako-metadata-scraper/src/providers/douban.rs,crates/nako-metadata-scraper/src/providers/douban]
  Goal: Split browser-worker support into a deep rendered-page runtime Module and clarify whether `browser_worker` remains a real metadata provider or is removed from the provider catalog.
  Validation: cargo nextest run -p nako-metadata-scraper browser_worker douban --no-fail-fast
  Review: Confirm support dependencies are not presented as provider identity unless they provide real metadata semantics.
  Evidence: `cargo nextest run -p nako-metadata-scraper browser_worker douban --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Handoff: DONE. Browser-worker protocol details now live in `providers/rendered_page.rs`; Douban uses that support runtime for rendered HTML, while `browser_worker` remains a default-off metadata provider only for explicit rendered-page URL extraction.

## M4 - Integration, Docs, And Gates

- [x] OMAPAD-070 [owner=codex] [deps=OMAPAD-050,OMAPAD-060] [scope=crates/nako-metadata-scraper,addons/metadata-scraper,docs/workstreams/official-metadata-addon-provider-architecture-deepening]
  Goal: Integrate all provider architecture refactors, update docs/examples if config or provider strategy changed, and record fresh targeted evidence.
  Validation: cargo nextest run -p nako-metadata-scraper --no-fail-fast
  Review: review-workstream before accepting completion.
  Evidence: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Handoff: DONE. Full metadata-scraper package gate passed; user-facing metadata scraper docs already describe browser-worker rendered-page extraction and Douban provider strategy, so no public config doc change was required.

## M5 - Closeout

- [x] OMAPAD-080 [owner=planner] [deps=OMAPAD-070] [scope=docs/workstreams/official-metadata-addon-provider-architecture-deepening]
  Goal: Close the lane or split any remaining architecture candidates into narrower follow-ons.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-provider-architecture-deepening/WORKSTREAM.json`; `git diff --check`
  Handoff: DONE. Workstream closed with no blocking review findings and no split follow-ons.
