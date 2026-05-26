# Official Metadata Addon AV Parser Quality - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

Wave 3 is complete and exposed a parser-quality gap: the official uncensored
provider uses row-level structured label extraction to prevent field bleeding,
while older AV providers mostly use repeated full-text label scanners.

## Active Task

- Task ID: APQ-020
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers/dmm`, `crates/nako-metadata-scraper/src/providers/mgstage.rs`, `crates/nako-metadata-scraper/src/providers/javbus.rs`
- Validation: `cargo nextest run -p nako-metadata-scraper dmm mgstage javbus rendered_av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Keep migrations behavior-preserving except for tested field-boundary fixes.

## Blockers

- None.

## Next Recommended Action

- Execute APQ-020 with TDD: migrate DMM, MGStage, and JavBus only where
  row-level label parsing proves cleaner behavior.
