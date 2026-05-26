# Official Metadata Addon Scraper Architecture Deepening

Status: Closed
Last updated: 2026-05-26

## Why This Lane Exists

The metadata scraper now has a mature provider registry, cross-provider
resolver, native metadata writeback, browser-worker rendered extraction, AV
provider wave 2, and bulk provider suppression/cooldown accounting. That is the
right product shape, but the latest architecture review found new shallow
Modules created by scale:

- bulk scrape uses the public response JSON as an internal Interface;
- rendered AV providers repeat direct lookup, route gating, render, search,
  detail, empty, and parse policy;
- Rust rendered-page calls cannot express browser-worker wait/proxy/session
  intent even though the worker supports it;
- AV default field policy hard-codes provider quality inside engine query code;
- resolver, candidate fusion, ranking, artwork, and native writeback projection
  are coupled through broad data shapes;
- metadata and artwork writeback repeat the same side-effect state machine.

This lane intentionally allows breaking changes. The target is a cleaner
future-facing module architecture, not compatibility with the current internal
payload seams.

## Relevant Authority

- Architecture review generated on 2026-05-26:
  - `C:/Users/Frankorz/AppData/Local/Temp/architecture-review-20260526-131914.html`
- Current implementation:
  - `crates/nako-metadata-scraper/src/engine/runtime.rs`
  - `crates/nako-metadata-scraper/src/engine/bulk.rs`
  - `crates/nako-metadata-scraper/src/engine/orchestration.rs`
  - `crates/nako-metadata-scraper/src/engine/resolver.rs`
  - `crates/nako-metadata-scraper/src/engine/ranking.rs`
  - `crates/nako-metadata-scraper/src/engine/native_writeback.rs`
  - `crates/nako-metadata-scraper/src/engine/writeback.rs`
  - `crates/nako-metadata-scraper/src/engine/artwork.rs`
  - `crates/nako-metadata-scraper/src/providers/rendered_page.rs`
  - `crates/nako-metadata-scraper/src/providers/rendered_av.rs`
  - `crates/nako-metadata-scraper/src/providers/javbus.rs`
  - `crates/nako-metadata-scraper/src/providers/javlibrary.rs`
  - `crates/nako-metadata-scraper/src/providers/mgstage.rs`
  - `addons/browser-worker/src/app.mjs`
  - `addons/browser-worker/src/extract.mjs`
- Prior workstreams:
  - `docs/workstreams/official-metadata-addon-provider-architecture-deepening/`
  - `docs/workstreams/official-metadata-addon-provider-fact-resolver/`
  - `docs/workstreams/official-metadata-browser-worker/`
  - `docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2/`

## Domain Vocabulary

- Scrape Outcome: internal typed result of one metadata scrape before public
  response rendering.
- Render Intent: provider-owned browser render request facts, including URL,
  wait, proxy, session, extraction mode, and redaction-safe labels.
- Rendered AV Flow: shared AV scrape flow for rendered providers, including
  direct lookup, route support, search-to-detail, empty result policy, parse
  outcomes, and candidate construction hooks.
- Provider Quality Profile: provider-owned metadata about field strengths,
  source preferences, and default ranking contributions.
- Candidate Fusion: transformation from resolved provider fact clusters into
  final ranked metadata candidates.
- Side Effect Writeback: shared state machine for optional Nako runtime writes.

## Problem

The scraper's public runtime Interface is still compact, but internal Modules
have become shallow. Callers and tests need to know too much about JSON payload
shape, provider ordering, scrape flow ordering, writeback status rules, and
which stage owns field projection.

The deletion test confirms the issue: deleting `bulk.rs` JSON parsing, rendered
provider flow snippets, or field policy provider lists would not remove
complexity. It would reappear across callers. These are earning a deeper Module.

## Target State

When this lane closes:

- one scrape produces a typed `ScrapeOutcome` consumed by bulk and response
  rendering without round-tripping through public JSON;
- rendered browser calls carry a first-class `RenderIntent` from provider
  Modules to browser-worker payloads;
- rendered AV providers share a deep flow Module while keeping site-specific
  URL, route, parser, and mapper logic local;
- provider descriptors contribute quality/field profile facts so engine policy
  executes provider priorities without hard-coding every AV provider identity;
- resolver identifies entities, candidate fusion owns field selection and
  evidence assembly, ranking ranks candidates, and native writeback projection
  sits after final candidate assembly;
- metadata and artwork writes use one side-effect writeback state machine with
  type-specific adapters;
- tests prove each new Interface directly, not by reaching through unrelated
  Modules.

## In Scope

- Breaking internal engine/provider Interfaces.
- Breaking addon task output schema when a cleaner typed structure requires it,
  with explicit schema version update.
- Updating Rust rendered-page contract to include wait/proxy/session intent.
- Updating browser-worker request validation only as needed to keep the contract
  aligned.
- Moving duplicated AV provider flow into shared Modules.
- Moving provider field quality knowledge closer to provider descriptors.
- Refactoring resolver/ranking/native-writeback/writeback Modules for locality.
- Updating README, manifest examples, workstream docs, and tests.

## Out Of Scope

- Adding new providers as a feature goal.
- Replacing browser-worker with a different automation stack.
- Nako core refresh modes, locked fields, local NFO, local artwork priority, or
  library scheduling.
- Full Jellyfin-style host provider manager inside the addon.
- Live scraping gates against adult websites in CI.
- Release packaging or Docker publish.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Bulk's JSON round-trip is the highest-leverage cleanup. | High | `bulk.rs` extracts provider reports and AV facts from `self.scrape()` response JSON. | Start with rendered AV flow if typed outcome proves too invasive. |
| Browser-worker already supports enough render options to justify `RenderIntent`. | High | `addons/browser-worker/src/app.mjs` and `extract.mjs` validate wait/proxy/session options. | Keep URL-only Rust calls and postpone worker alignment. |
| Rendered AV providers now prove a real shared Seam. | High | JavBus, JavLibrary, and MGStage repeat direct lookup/render/search/detail flow. | Keep helper-only `rendered_av.rs` and only deepen per duplicated behavior. |
| Provider quality belongs near provider descriptors. | Medium | `ProviderFieldPolicy::default_av()` hard-codes provider identity and order in `query.rs`. | Keep engine policy but generate its default table from descriptors. |
| Resolver/fusion split can be staged after typed outcome. | Medium | Existing resolver already clusters provider facts and produces candidates. | Preserve current resolver while moving projection/ranking first. |
| Side-effect writeback consolidation should come after outcome/fusion work. | Medium | Metadata/artwork writes share state machine but differ in selection/provenance. | Split if it becomes a separate durable lane. |

## Architecture Direction

Deepen Modules where the Interface currently leaks implementation details. The
public HTTP routes stay thin adapters. Provider adapters stay site-local. The
engine should own scrape flow and typed outcomes. Response rendering should be a
projection from typed outcome to public payload, not an internal protocol.

The first refactor should introduce typed scrape outcome because it removes the
dirtiest Seam and gives later slices a stable internal result to consume. After
that, rendered provider work should make browser rendering more expressive
without putting anti-bot policy in central engine code. Resolver and field
policy work should then reduce provider identity leakage and make candidate
assembly explicitly testable.

## Closeout Condition

This lane can close when:

- all OMSAD tasks are complete or explicitly split into a follow-up workstream;
- the full `nako-metadata-scraper` package gate passes;
- browser-worker tests pass if its contract changes;
- docs describe any broken task/output contract or config behavior;
- workstream JSON and diff hygiene pass;
- commits are split by coherent refactor slice.

## Closeout Summary

Closed on 2026-05-26. All six architecture-review candidates shipped:

- typed scrape outcome for response and bulk projections;
- typed render intent for browser-worker calls;
- shared rendered AV flow for JavBus, JavLibrary, and MGStage;
- provider descriptor-derived AV field quality defaults;
- resolver/fusion/ranking/native writeback projection split;
- shared side-effect writeback state machine for metadata and artwork writes.

Final gates passed: full `nako-metadata-scraper` package validation,
browser-worker tests, formatting, workstream JSON, and diff hygiene. Additional
provider breadth, Nako core refresh/locked-field policy, local metadata/artwork
priority, and persistent provider cache/rate-limit policy should use new
workstreams.
