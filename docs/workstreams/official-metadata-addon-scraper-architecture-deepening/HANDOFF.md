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
OMSAD-060 is complete: resolver owns cluster identity and merge evidence,
`fusion` owns field selection/evidence, ranking owns score ordering, and native
writeback projection is exposed through `native_writeback`.
OMSAD-070 is complete: metadata and artwork writeback now share
`side_effect::run_side_effect_writeback`; metadata/artwork adapters own target
validation, candidate/payload preparation, provenance, submission, and result
shape.

The architecture review identified six deepening candidates. This workstream
will solve all six unless a task reveals that a candidate should be split into a
separate durable lane.

## Active Task

- Task ID: OMSAD-080
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/engine`, `crates/nako-metadata-scraper/src/providers`, `addons/metadata-scraper/README.md`, `crates/nako-metadata-scraper/README.md`, `docs/workstreams/official-metadata-addon-scraper-architecture-deepening`
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `npm --prefix addons/browser-worker test`; `python -m json.tool docs/workstreams/official-metadata-addon-scraper-architecture-deepening/WORKSTREAM.json`; `git diff --check`
- Status: READY
- Review: Confirm no architecture-review candidate remains unresolved unless
  explicitly split.
- Evidence: OMSAD-070 passed `cargo nextest run -p nako-metadata-scraper side_effect_state_machine --no-fail-fast`, `cargo nextest run -p nako-metadata-scraper writeback artwork runtime --no-fail-fast`, and `cargo fmt -p nako-metadata-scraper -- --check`.

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

- Execute OMSAD-080 by running full gates, checking docs/readmes against the
  shipped architecture, and closing or splitting any remaining follow-up work.
