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

The architecture review identified six deepening candidates. This workstream
will solve all six unless a task reveals that a candidate should be split into a
separate durable lane.

## Active Task

- Task ID: OMSAD-050
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers/registry.rs`, `crates/nako-metadata-scraper/src/providers/mod.rs`, `crates/nako-metadata-scraper/src/providers/*`, `crates/nako-metadata-scraper/src/engine/query.rs`, `crates/nako-metadata-scraper/src/engine/resolver.rs`, `addons/metadata-scraper/README.md`, `crates/nako-metadata-scraper/README.md`
- Validation: `cargo nextest run -p nako-metadata-scraper config registry manifest field_policy resolver av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm request-provided `provider_field_policy` still overrides
  defaults and docs describe descriptor-derived defaults.
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

- Execute OMSAD-050 by moving default AV field-quality/profile facts into
  provider descriptors and removing provider identity lists from engine query
  policy construction.
