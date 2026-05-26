# Task Ledger

Prefix: OMSAD

## Active

- [ ] OMSAD-050 [owner=codex] [deps=OMSAD-040] [scope=crates/nako-metadata-scraper/src/providers/registry.rs,crates/nako-metadata-scraper/src/providers/mod.rs,crates/nako-metadata-scraper/src/providers/*,crates/nako-metadata-scraper/src/engine/query.rs,crates/nako-metadata-scraper/src/engine/resolver.rs,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Move AV provider field-quality/default policy facts toward provider descriptors and stop hard-coding default provider identity lists in engine query code.
  Validation: `cargo nextest run -p nako-metadata-scraper config registry manifest field_policy resolver av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirm request-provided `provider_field_policy` still overrides defaults and docs describe descriptor-derived defaults.
  Evidence:
  Handoff:

## Pending

- [ ] OMSAD-060 [owner=codex] [deps=OMSAD-050] [scope=crates/nako-metadata-scraper/src/engine/resolver.rs,crates/nako-metadata-scraper/src/engine/ranking.rs,crates/nako-metadata-scraper/src/engine/artwork.rs,crates/nako-metadata-scraper/src/engine/native_writeback.rs]
  Goal: Split entity resolution from candidate fusion/ranking/native writeback projection so each Interface is testable at one Seam.
  Validation: `cargo nextest run -p nako-metadata-scraper resolver ranking artwork writeback av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirm resolver owns cluster identity, fusion owns field selection/evidence, ranking owns ordering, and native writeback projection is not hidden inside ranking.
  Evidence:
  Handoff:

- [ ] OMSAD-070 [owner=codex] [deps=OMSAD-020,OMSAD-060] [scope=crates/nako-metadata-scraper/src/engine/writeback.rs,crates/nako-metadata-scraper/src/engine/artwork.rs,crates/nako-metadata-scraper/src/engine/runtime.rs]
  Goal: Consolidate metadata/artwork side-effect writeback into one state machine Module with type-specific adapters for payload and provenance.
  Validation: `cargo nextest run -p nako-metadata-scraper writeback artwork runtime --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirm disabled runtime, invalid target, access denied, access failure, submit failure, and success statuses are tested through the shared Interface.
  Evidence:
  Handoff:

- [ ] OMSAD-080 [owner=codex] [deps=OMSAD-070] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/providers,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md,docs/workstreams/official-metadata-addon-scraper-architecture-deepening]
  Goal: Run full gates, update docs, review module boundaries, and close or split follow-ups.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `npm --prefix addons/browser-worker test`; `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json`; `git diff --check`
  Review: Confirm no architecture-review candidate remains unresolved unless explicitly split.
  Evidence:
  Handoff:

## Completed

- [x] OMSAD-010 [owner=codex] [deps=-] [scope=docs/workstreams/official-metadata-addon-scraper-architecture-deepening]
  Goal: Open the durable architecture deepening lane with task ledger, target state, gates, and no-compatibility policy.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json`; `git diff --check`
  Review: Confirmed all six architecture-review candidates are represented as independently verifiable slices.
  Evidence: PASS on 2026-05-26.
  Handoff: DONE. OMSAD-020 completed.

- [x] OMSAD-020 [owner=codex] [deps=OMSAD-010] [scope=crates/nako-metadata-scraper/src/engine/runtime.rs,crates/nako-metadata-scraper/src/engine/response.rs,crates/nako-metadata-scraper/src/engine/bulk.rs,crates/nako-metadata-scraper/src/engine/orchestration.rs,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Introduce a typed scrape outcome Seam so bulk and response rendering consume internal facts without public JSON round-trip.
  Validation: `cargo nextest run -p nako-metadata-scraper bulk runtime metadata_endpoint --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirmed public response rendering is a projection from `MetadataScrapeOutcome`; bulk fresh scrape consumes typed provider execution, AV facts, failure reason, and suppression facts without parsing `AddonResourceResponse` JSON.
  Evidence: PASS on 2026-05-26; 41 focused tests passed, full package gate passed with 193 tests, formatting and diff hygiene passed.
  Handoff: DONE. OMSAD-030 completed.

- [x] OMSAD-030 [owner=codex] [deps=OMSAD-020] [scope=crates/nako-metadata-scraper/src/providers/rendered_page.rs,crates/nako-metadata-scraper/src/providers/browser_worker.rs,crates/nako-metadata-scraper/src/providers/douban.rs,crates/nako-metadata-scraper/src/providers/douban/client.rs,crates/nako-metadata-scraper/src/providers/javdb.rs,crates/nako-metadata-scraper/src/providers/javdb/client.rs,crates/nako-metadata-scraper/src/providers/dmm.rs,crates/nako-metadata-scraper/src/providers/dmm/client.rs,crates/nako-metadata-scraper/src/providers/fc2.rs,crates/nako-metadata-scraper/src/providers/fc2/client.rs,crates/nako-metadata-scraper/src/providers/javbus.rs,crates/nako-metadata-scraper/src/providers/javlibrary.rs,crates/nako-metadata-scraper/src/providers/mgstage.rs,addons/browser-worker/test,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Add Render Intent as the deep Interface for browser-worker calls, including wait/proxy/session fields where supported.
  Validation: `cargo nextest run -p nako-metadata-scraper rendered_page browser_worker douban javbus javlibrary mgstage --no-fail-fast`; `npm --prefix addons/browser-worker test`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirmed providers declare browser-worker calls through `RenderedPageIntent` and `RenderedPageSupportConfig::intent`; wait/proxy/session fields serialize through one shared request projection without provider-local payload assembly.
  Evidence: PASS on 2026-05-26; render intent red/green tracer passed, 19 focused Rust tests passed, 4 browser-worker tests passed.
  Handoff: DONE. OMSAD-040 completed.

- [x] OMSAD-040 [owner=codex] [deps=OMSAD-030] [scope=crates/nako-metadata-scraper/src/providers/rendered_av.rs,crates/nako-metadata-scraper/src/providers/javbus.rs,crates/nako-metadata-scraper/src/providers/javlibrary.rs,crates/nako-metadata-scraper/src/providers/mgstage.rs]
  Goal: Deepen rendered AV provider flow so direct lookup, AV route gating, search-to-detail, and empty/failure policy live in one reusable Module.
  Validation: `cargo nextest run -p nako-metadata-scraper javbus javlibrary mgstage av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Confirmed `RenderedAvFlow` owns direct URL/ID lookup, AV route gating, search-to-detail, first-result empty behavior, and detail rendering; provider adapters retain URL construction, parser, and mapper quirks.
  Evidence: PASS on 2026-05-26; rendered AV flow red/green tracer passed and 33 focused Rust tests passed.
  Handoff: DONE. OMSAD-050 is active.

## Follow-Up Candidates

- Nako core refresh policy, locked fields, local metadata, and local artwork
  priority.
- Additional AV providers after rendered AV flow is deep enough.
- Persistent provider cache/rate-limit policy if real-site usage proves the need.
