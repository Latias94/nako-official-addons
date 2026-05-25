# Handoff

Status: Active
Current task: OMAV-050
Last updated: 2026-05-26

## Context

The user asked to reference `repo-ref/mdcx` for mature AV scraping and batch strategy. The implementation should be fearless where the local architecture benefits, but must keep Nako's existing task/runtime model.

## Decisions

- Use MDCx only as behavioral reference because of GPLv3 and project-specific terms.
- Put AV number recognition in the query layer so all current and future AV providers share the same facts.
- Add JavDB as the first browser-worker-backed AV provider, disabled by default.
- Extend `bulk-metadata-scrape` output with AV summaries instead of adding a separate batch scheduler.

## Completed In This Slice

- `engine::av` extracts and normalizes AV numbers from explicit IDs, AV fields, file names, paths, titles, and names.
- Metadata responses include optional redaction-safe `query.av`.
- Bulk task item output includes optional per-item `av` summary.
- Bulk task reuses duplicate AV-number scrape results inside one bounded batch when items do not request side effects.
- Bulk task reports `reused_from_index`, summary counters, and `safe_failure_reason: no_candidates`.
- JavDB provider is available as `javdb`, disabled by default, and uses browser-worker rendered search/detail pages.
- FC2 provider is available as `fc2`, disabled by default, and uses browser-worker rendered direct article pages for FC2 numbers.
- Provider registry, manifest schema, README, example manifest, and diagnostics know about JavDB.

## Validation

- `cargo nextest run -p nako-metadata-scraper av --no-fail-fast`: passed.
- `cargo nextest run -p nako-metadata-scraper javdb --no-fail-fast`: passed.
- `cargo nextest run -p nako-metadata-scraper fc2 --no-fail-fast`: 3 passed.
- `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast`: 8 passed.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: 167 passed, 2 skipped.
- `rustfmt --edition 2024 --check <modified nako-metadata-scraper rust files>`: passed.
- `git diff --check`: passed.

## Next Steps

1. Commit this vertical slice.
2. Decide whether to open OMAV-090 for broader multi-provider AV routing and field provenance.
3. Decide whether to open OMAV-080 for richer cross-batch failure accounting and resume semantics.
