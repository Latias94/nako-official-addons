# Official Metadata Addon AV Provider Wave 3 - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

The lane is newly opened. The previous scraper architecture lane is closed and
provides the foundation for this work: typed scrape outcomes, render intent,
rendered AV flow, provider quality descriptors, resolver/fusion split, and
shared side-effect writeback. This lane should add provider breadth without
copying provider test or operational policy decisions.

## Active Task

- Task ID: OMAV3-020
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers/rendered_av.rs`, `crates/nako-metadata-scraper/src/providers/*`, `crates/nako-metadata-scraper/src/engine`
- Validation: `cargo nextest run -p nako-metadata-scraper rendered_av provider_fixture av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Confirm existing JavBus/JavLibrary/MGStage tests use the shared
  harness or justify provider-local tests.
- Evidence: OMAV3-010 passed JSON and diff hygiene gates.

## Decisions

- Start with a shared rendered AV fixture harness before adding more providers.
- Keep all new providers disabled by default.
- Keep MDCx as reference-only; do not copy code, selectors, fixtures, regex
  tables, comments, or structure.
- Treat provider protection as explicit policy/state, not hidden scheduler
  memory.

## Blockers

- None.

## Next Recommended Action

- Execute OMAV3-020 with TDD: introduce the shared rendered AV provider fixture
  harness and migrate at least one existing provider test onto it.
