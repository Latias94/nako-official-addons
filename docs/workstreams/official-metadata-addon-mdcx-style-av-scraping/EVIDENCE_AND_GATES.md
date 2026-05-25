# Evidence And Gates

Status: Active
Last updated: 2026-05-26

## Required Gates

| Gate | Command | Status | Evidence |
| --- | --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-mdcx-style-av-scraping/WORKSTREAM.json` | Passed | JSON parsed successfully. |
| Example manifest JSON | `python -m json.tool addons/metadata-scraper/manifest.example.json` | Passed | JSON parsed successfully. |
| Targeted AV query tests | `cargo nextest run -p nako-metadata-scraper av --no-fail-fast` | Passed | 12 passed. |
| Targeted JavDB tests | `cargo nextest run -p nako-metadata-scraper javdb --no-fail-fast` | Passed | 3 passed. |
| Targeted bulk tests | `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast` | Passed | 8 passed. |
| Targeted config tests | `cargo nextest run -p nako-metadata-scraper config --no-fail-fast` | Passed | 13 passed. |
| Targeted registry tests | `cargo nextest run -p nako-metadata-scraper registry --no-fail-fast` | Passed | 11 passed. |
| Targeted routes tests | `cargo nextest run -p nako-metadata-scraper routes --no-fail-fast` | Passed | 7 passed. |
| Package tests | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Passed | 165 passed, 2 skipped. |
| Formatting | `rustfmt --edition 2024 --check <modified nako-metadata-scraper rust files>` | Passed | Modified Rust files are formatted. |
| Full workspace formatting | `cargo fmt --all -- --check` | Not used as gate | It scans the adjacent `../nako` path dependency and reports pre-existing formatting differences outside this repo's edited files. |
| Diff hygiene | `git diff --check` | Passed | No whitespace errors. |

## Evidence Log

- 2026-05-26: Reviewed MDCx number recognition, crawler routing, JavDB search/detail behavior, and batch scraper behavior as reference-only input.
- 2026-05-26: Confirmed this repo already has `bulk-metadata-scrape`; implementation will extend it rather than add a parallel batch executor.
- 2026-05-26: Added AV query facts, disabled-by-default JavDB rendered provider, and bulk item AV summaries.
- 2026-05-26: Verified package tests and modified-file formatting. Full workspace `cargo fmt --all -- --check` is not a clean gate because path dependency `../nako` has unrelated pre-existing formatting drift.
- 2026-05-26: Added bounded-batch duplicate AV-number reuse, `reused_from_index`, summary counters, and `safe_failure_reason: no_candidates`.
