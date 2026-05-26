# Official Metadata Addon Scraper Architecture Deepening - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane is active. The previous AV native writeback/provider wave 2 lane is
closed. OMSAD-020 is complete: runtime now builds a typed
`MetadataScrapeOutcome`, response rendering projects from it, and bulk fresh
scrape consumes typed AV facts, provider execution, failure reason, and provider
suppression facts without parsing public response JSON.
OMSAD-030 is complete: rendered providers now declare a typed
`RenderedPageIntent`, shared runtime projection serializes browser-worker
`wait_for`, `proxy_policy`, and `session_key`, and environment defaults can
shape all rendered-page requests without provider-local JSON assembly.
OMSAD-040 is complete: `RenderedAvFlow` owns direct lookup, route gating,
search-to-detail, empty result behavior, and detail rendering for JavBus,
JavLibrary, and MGStage; provider adapters keep URL construction and parsing.
OMSAD-050 is complete: default AV field policy is generated from provider
quality descriptors in the catalog, runtime receives that default explicitly,
and request-provided `provider_field_policy` still overrides it.

The architecture review identified six deepening candidates. This workstream
will solve all six unless a task reveals that a candidate should be split into a
separate durable lane.

## Active Task

- Task ID: OMSAD-060
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/engine/resolver.rs`, `crates/nako-metadata-scraper/src/engine/ranking.rs`, `crates/nako-metadata-scraper/src/engine/artwork.rs`, `crates/nako-metadata-scraper/src/engine/native_writeback.rs`
- Validation: `cargo nextest run -p nako-metadata-scraper resolver ranking artwork writeback av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm resolver owns cluster identity, fusion owns field
  selection/evidence, ranking owns ordering, and native writeback projection is
  not hidden inside ranking.
- Evidence:

## Decisions

- Use fearless refactoring: compatibility shims are not required when a cleaner
  Interface or schema-versioned output is better.
- Start with typed scrape outcome because bulk JSON round-trip is the highest
  leverage shallow Seam.
- Keep provider-specific site quirks local to provider adapters.
- Keep browser-worker as the browser/proxy/session/wait owner.
- Do not implement Nako core refresh policy in this addon lane.

## Blockers

- None.

## Next Recommended Action

- Execute OMSAD-060 by splitting resolver cluster identity, candidate
  field-fusion/evidence, ranking, artwork selection, and native writeback
  projection behind narrower test seams.
