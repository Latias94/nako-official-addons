# Official Metadata Addon AV Field Policy Presets - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

Closed. The addon now supports configurable AV field policy presets:
`default`, `quality_scores`, and `none`.

## Active Task

- Task ID: OMAFP-020
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/config.rs`, `crates/nako-metadata-scraper/src/providers/registry.rs`
- Validation: `cargo nextest run -p nako-metadata-scraper config registry field_policy --no-fail-fast`
- Status: DONE
- Review: Self-reviewed with targeted and package gates.
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Keep AV provider enablement presets separate from AV field policy presets.
- Adapt reference field source orders to supported providers instead of copying implementation.
- Leave JavBus cookie/session proof outside this lane.
- Default `av_field_policy_preset` is `default`; `quality_scores` preserves the descriptor-derived behavior.
- Request payload `provider_field_policy` still replaces the configured default per scrape.

## Blockers

- None.

## Follow-Ons

- Add more providers if parity needs fields from unsupported sources such as Jav321,
  AvSex, FreeJavBT, or 7MMTV.
- JavBus live proof still requires a real operator cookie/session because the site currently reaches
  age/region verification through the proxy.

## Next Recommended Action

- Continue provider breadth or live scraping work; this field policy lane is closed.
