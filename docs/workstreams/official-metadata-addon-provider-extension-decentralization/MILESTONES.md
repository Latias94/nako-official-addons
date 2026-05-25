# Official Metadata Addon Provider Extension Decentralization - Milestones

Status: Complete
Last updated: 2026-05-25

## M0 - Scope Frozen

- Workstream docs exist and agree.
- Tasks target provider extension friction, not release/smoke work.
- Compatibility non-goals are explicit.

## M1 - Provider Config Decentralized

- Provider config rows cannot carry unrelated provider config shapes.
- Provider-local config structs live near provider adapters or behind a small
  provider-owned Interface.
- Manifest defaults and secret references remain compatible.

## M2 - External ID Aliases Provider-Owned

- Top-level alias extraction is descriptor-driven.
- Existing `tmdb_id`, `imdb_id`, `bangumi_id`, and explicit external ID payloads
  keep working.
- Query parsing does not gain provider implementation coupling.

## M3 - Rendered Support Explicit

- Douban's browser-rendered dependency is named and configured as support
  infrastructure.
- `browser_worker` keeps metadata provider identity only for explicit rendered
  page URL extraction.
- Another browser-rendered provider can reuse the support Module without
  copying worker HTTP contract code.

## M4 - Integrated Gates

- Targeted metadata scraper tests pass for config, manifest, query parsing,
  provider registry, browser worker, and Douban.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passes.
- `cargo fmt --all -- --check` passes.
- `git diff --check` passes.

## M5 - Closeout Ready

- EVIDENCE_AND_GATES.md has fresh command evidence.
- TODO.md task handoff states are final.
- HANDOFF.md records remaining risks and follow-ons.
- WORKSTREAM.json is updated to `complete` or a follow-on is split.

Status: Complete. OMAPED-060 closed the lane with final package, format, JSON,
and diff hygiene evidence; no architecture follow-on was split.
