# Official Metadata Addon AV Parser Quality - Handoff

Status: Complete
Last updated: 2026-05-26

## Current State

Wave 3 is complete and exposed a parser-quality gap: the official uncensored
provider uses row-level structured label extraction to prevent field bleeding,
while older AV providers mostly use repeated full-text label scanners.

## Closeout

- Task ID: APQ-040
- Status: DONE
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed
  with 222 tests; `cargo fmt -p nako-metadata-scraper -- --check`,
  `python -m json.tool docs/workstreams/official-metadata-addon-av-parser-quality/WORKSTREAM.json`,
  and `git diff --check` passed.
- Review: No parser-quality decision exists only in journal notes. All current
  AV provider families share row-level label parsing where relevant.

## Blockers

- None.

## Follow-Ups

- Future Wave 4 providers should use `rendered_av::structured_or_labeled_value`
  with provider-local row selectors from the start.
- Manual live drift tooling remains useful, but CI must not store adult-site
  payloads.
