# Evidence And Gates

Status: Complete
Last updated: 2026-05-26

## Required Gates

| Gate | Command | Status | Evidence |
| --- | --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-mdcx-style-av-scraping/WORKSTREAM.json` | Passed | JSON parsed successfully. |
| Example manifest JSON | `python -m json.tool addons/metadata-scraper/manifest.example.json` | Passed | JSON parsed successfully. |
| Targeted AV query tests | `cargo nextest run -p nako-metadata-scraper av --no-fail-fast` | Passed | 12 passed. |
| Targeted JavDB tests | `cargo nextest run -p nako-metadata-scraper javdb --no-fail-fast` | Passed | 6 passed. |
| Targeted FC2 tests | `cargo nextest run -p nako-metadata-scraper fc2 --no-fail-fast` | Passed | 7 passed. |
| Targeted engine tests | `cargo nextest run -p nako-metadata-scraper engine --no-fail-fast` | Passed | 66 passed. |
| Targeted bulk tests | `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast` | Passed | 10 passed. |
| Targeted config tests | `cargo nextest run -p nako-metadata-scraper config --no-fail-fast` | Passed | 13 passed. |
| Targeted registry tests | `cargo nextest run -p nako-metadata-scraper registry --no-fail-fast` | Passed | 11 passed. |
| Targeted routes tests | `cargo nextest run -p nako-metadata-scraper routes --no-fail-fast` | Passed | 7 passed. |
| Package tests | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Passed | 175 passed, 2 skipped. |
| Formatting | `rustfmt --edition 2024 --check <modified nako-metadata-scraper rust files>` | Passed | Modified Rust files are formatted. |
| Full workspace formatting | `cargo fmt --all -- --check` | Not used as gate | It scans the adjacent `../nako` path dependency and reports pre-existing formatting differences outside this repo's edited files. |
| Diff hygiene | `git diff --check` | Passed | No whitespace errors. |
| Direct AV provider lookup | `cargo nextest run -p nako-metadata-scraper javdb --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper fc2 --no-fail-fast` | Passed | JavDB 6 passed; FC2 7 passed. |
| Provider execution summary | `cargo nextest run -p nako-metadata-scraper engine --no-fail-fast` | Passed | 66 passed. |
| Resumable batch accounting | `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast` | Passed | 10 passed. |

## Evidence Log

- 2026-05-26: Reviewed MDCx number recognition, crawler routing, JavDB search/detail behavior, and batch scraper behavior as reference-only input.
- 2026-05-26: Confirmed this repo already has `bulk-metadata-scrape`; implementation will extend it rather than add a parallel batch executor.
- 2026-05-26: Added AV query facts, disabled-by-default JavDB rendered provider, and bulk item AV summaries.
- 2026-05-26: Verified package tests and modified-file formatting. Full workspace `cargo fmt --all -- --check` is not a clean gate because path dependency `../nako` has unrelated pre-existing formatting drift.
- 2026-05-26: Added bounded-batch duplicate AV-number reuse, `reused_from_index`, summary counters, and `safe_failure_reason: no_candidates`.
- 2026-05-26: Added route-specific FC2 direct article provider using shared AV facts and browser-worker rendered HTML.
- 2026-05-26: Added provider-declared AV route support and redaction-safe field/provider-source evidence for merged provider facts.
- 2026-05-26: Closeout package gate passed with 171 passed, 2 skipped; modified-file format, JSON validation, and diff hygiene gates passed.
- 2026-05-26: Reopened the workstream for AV maturity follow-up covering direct provider lookup, provider execution summaries, and resumable batch accounting.
- 2026-05-26: Added explicit `javdb` and `fc2` provider direct lookup paths that prefer supplied provider IDs before inferred AV-number search.
- 2026-05-26: Added response-level provider execution summaries for selected, route-skipped, returned, empty, and failed providers with redaction-safe failure categories.
- 2026-05-26: Added bulk `resume_state`, richer failure-reason counters, provider-level batch summaries, and cross-batch reuse tests.
- 2026-05-26: Closeout package gate passed with 175 passed, 2 skipped; workstream JSON and diff hygiene gates passed.
