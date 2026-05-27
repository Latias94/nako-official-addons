# Official Metadata Addon Execution Locality Deepening - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

- [x] OMAELD-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-execution-locality-deepening]
  Goal: Freeze the fearless-refactor target state for Bulk execution, provider catalog locality, and render drift locality.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Evidence: docs/workstreams/official-metadata-addon-execution-locality-deepening/DESIGN.md
  Handoff: DONE. Workstream opened as a follow-on to closed architecture lanes after code review found residual shallow Seams.

## M1 - Bulk Provider Execution Typed Facts

- [x] OMAELD-020 [owner=codex] [deps=OMAELD-010] [scope=crates/nako-metadata-scraper/src/engine]
  Goal: Remove Bulk Metadata Scrape's provider execution policy JSON tunnel and route Bulk through typed provider execution facts.
  Validation: cargo nextest run -p nako-metadata-scraper bulk provider_execution --no-fail-fast
  Review: Confirm Bulk no longer mutates request JSON for provider execution, while public scrape request parsing still behaves intentionally.
  Evidence: crates/nako-metadata-scraper/src/engine/bulk.rs; crates/nako-metadata-scraper/src/engine/runtime.rs; crates/nako-metadata-scraper/src/engine/provider_execution.rs
  Handoff: DONE. Bulk now builds a typed ProviderRunPolicy overlay and calls a typed runtime scrape path; the old payload mutation helper was removed.

## M2 - Provider Catalog Residual Locality

- [x] OMAELD-030 [owner=codex] [deps=OMAELD-020] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/providers]
  Goal: Reduce remaining provider-specific facts in central config/registry paths when provider-owned descriptors can own them more deeply.
  Validation: cargo nextest run -p nako-metadata-scraper provider registry config --no-fail-fast
  Review: Confirm central Modules compose/query provider facts instead of duplicating provider Implementation knowledge.
  Evidence: crates/nako-metadata-scraper/src/config.rs; crates/nako-metadata-scraper/src/providers/registry.rs
  Handoff: DONE. Rendered-page proxy/session configuration facts now live in provider catalog entries; Config delegates to ProviderRegistry instead of matching every rendered provider kind.

## M3 - Render Drift Case Locality

- [x] OMAELD-040 [owner=codex] [deps=OMAELD-030] [scope=crates/nako-metadata-scraper/src/providers/render_drift.rs,crates/nako-metadata-scraper/src/providers]
  Goal: Move provider-specific render drift sample/case selection toward provider-owned descriptors and keep the runner as sample lookup plus serialization.
  Validation: cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast
  Review: Confirm adding a rendered provider no longer requires broad central drift routing edits.
  Evidence: crates/nako-metadata-scraper/src/providers/render_drift.rs
  Handoff: DONE. Render drift cases now come from provider catalog descriptors that own sample env, fallback, order, and case builder facts; the runner no longer hard-codes per-provider branches.

## M4 - Integration And Closeout

- [x] OMAELD-050 [owner=codex] [deps=OMAELD-020,OMAELD-030,OMAELD-040] [scope=crates/nako-metadata-scraper,docs/workstreams/official-metadata-addon-execution-locality-deepening]
  Goal: Run package gates, update docs/evidence, and close or split any remaining residuals.
  Validation: cargo nextest run -p nako-metadata-scraper --no-fail-fast
  Review: review-workstream before accepting completion.
  Evidence: EVIDENCE_AND_GATES.md; WORKSTREAM.json; HANDOFF.md
  Handoff: DONE. Full metadata-scraper package gate, format check, JSON check, and diff hygiene gate passed. Review found no blocking issues; the default field-provider preference table is a possible future field-policy locality lane, not a residual for this execution-locality lane.
