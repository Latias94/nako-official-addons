# Official Metadata Addon AV Parser Quality - Handoff

Status: Active
Last updated: 2026-05-26

## Current State

Wave 3 is complete and exposed a parser-quality gap: the official uncensored
provider uses row-level structured label extraction to prevent field bleeding,
while older AV providers mostly use repeated full-text label scanners.

## Active Task

- Task ID: APQ-040
- Owner: codex
- Files: `crates/nako-metadata-scraper/README.md`, `addons/metadata-scraper/README.md`, `docs/workstreams/official-metadata-addon-av-parser-quality`
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `python -m json.tool docs/workstreams/official-metadata-addon-av-parser-quality/WORKSTREAM.json`; `git diff --check`
- Status: READY
- Review: Confirm no parser-quality decision exists only in journal notes.

## Blockers

- None.

## Next Recommended Action

- Execute APQ-040: run package validation, document parser-quality behavior,
  and close or split any remaining parser work.
