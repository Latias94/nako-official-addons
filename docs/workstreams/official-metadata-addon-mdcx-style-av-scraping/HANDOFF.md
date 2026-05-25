# Handoff

Status: Complete
Current task: None
Last updated: 2026-05-26

## Context

The user asked to reference `repo-ref/mdcx` for mature AV scraping and batch strategy. The implementation should be fearless where the local architecture benefits, but must keep Nako's existing task/runtime model.

## Decisions

- Use MDCx only as behavioral reference because of GPLv3 and project-specific terms.
- Put AV number recognition in the query layer so all current and future AV providers share the same facts.
- Add JavDB as the first browser-worker-backed AV provider, disabled by default.
- Extend `bulk-metadata-scrape` output with AV summaries instead of adding a separate batch scheduler.
- Let providers declare AV route support; the runtime filters provider calls by normalized AV route instead of hard-coding site-specific branches in batch code.

## Completed In This Slice

- `engine::av` extracts and normalizes AV numbers from explicit IDs, AV fields, file names, paths, titles, and names.
- Metadata responses include optional redaction-safe `query.av`.
- Bulk task item output includes optional per-item `av` summary.
- Bulk task reuses duplicate AV-number scrape results inside one bounded batch when items do not request side effects.
- Bulk task reports `reused_from_index`, summary counters, and `safe_failure_reason: no_candidates`.
- JavDB provider is available as `javdb`, disabled by default, and uses browser-worker rendered search/detail pages.
- FC2 provider is available as `fc2`, disabled by default, and uses browser-worker rendered direct article pages for FC2 numbers.
- Provider registry, manifest schema, README, example manifest, and diagnostics know about JavDB and FC2.
- AV route-aware orchestration keeps FC2 numbers on the FC2 path and non-FC2 AV numbers on the JavDB path.
- Candidate evidence includes redaction-safe `field_sources`, `provider_sources`, and `merge_reasons` for merged provider facts.

## Validation

- `cargo nextest run -p nako-metadata-scraper av --no-fail-fast`: passed.
- `cargo nextest run -p nako-metadata-scraper engine --no-fail-fast`: 66 passed.
- `cargo nextest run -p nako-metadata-scraper javdb --no-fail-fast`: 5 passed.
- `cargo nextest run -p nako-metadata-scraper fc2 --no-fail-fast`: 6 passed.
- `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast`: 8 passed.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: 171 passed, 2 skipped.
- `rustfmt --edition 2024 --check <modified nako-metadata-scraper rust files>`: passed.
- `git diff --check`: passed.

## Next Steps

1. Commit this vertical slice.
2. Decide whether to open OMAV-080 for richer cross-batch failure accounting and resume semantics.
