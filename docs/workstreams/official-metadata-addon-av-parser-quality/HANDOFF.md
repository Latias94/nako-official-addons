# Official Metadata Addon AV Parser Quality - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

Wave 3 is complete and exposed a parser-quality gap: the official uncensored
provider uses row-level structured label extraction to prevent field bleeding,
while older AV providers mostly use repeated full-text label scanners.

## Active Task

- Task ID: APQ-030
- Owner: codex
- Files: `crates/nako-metadata-scraper/src/providers/javlibrary.rs`, `crates/nako-metadata-scraper/src/providers/javdb`, `crates/nako-metadata-scraper/src/providers/fc2`
- Validation: `cargo nextest run -p nako-metadata-scraper javlibrary javdb fc2 rendered_av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
- Status: READY
- Review: Do not force all providers into one shape if their page models differ.

## Blockers

- None.

## Next Recommended Action

- Execute APQ-030: audit JavLibrary, JavDB, and FC2 for shared parser reuse and
  migrate only where tests prove cleaner behavior.
