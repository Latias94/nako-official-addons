# Evidence And Gates

Status: Complete
Last updated: 2026-05-26

## Required Gates

| Gate | Command | Required before |
| --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-av-mdcx-parity/WORKSTREAM.json` | OMAVM-010 completion |
| AV structured facts | `cargo nextest run -p nako-metadata-scraper av field_policy resolver javdb dmm fc2 --no-fail-fast` | OMAVM-020 completion |
| Browser worker contract | `npm --prefix addons/browser-worker test` | OMAVM-030 completion |
| Rendered-page Rust contract | `cargo nextest run -p nako-metadata-scraper rendered browser_worker javdb dmm fc2 --no-fail-fast` | OMAVM-030 completion |
| Provider expansion | `cargo nextest run -p nako-metadata-scraper config registry manifest av --no-fail-fast` | OMAVM-040 completion |
| Package validation | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | closeout |
| Format | `cargo fmt -p nako-metadata-scraper -- --check` | closeout |
| Diff hygiene | `git diff --check` | closeout |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | OMAVM-010 | `python -m json.tool docs/workstreams/official-metadata-addon-av-mdcx-parity/WORKSTREAM.json` | Pass |
| 2026-05-26 | OMAVM-020 | `cargo nextest run -p nako-metadata-scraper av field_policy resolver javdb dmm fc2 --no-fail-fast`; later `cargo nextest run -p nako-metadata-scraper av field_policy resolver javdb dmm fc2 javbus --no-fail-fast` | Pass: 31 related tests, then 34 related tests |
| 2026-05-26 | OMAVM-030 | `npm --prefix addons/browser-worker test`; `cargo nextest run -p nako-metadata-scraper rendered browser_worker javdb dmm fc2 --no-fail-fast` | Pass: 4 browser-worker tests; 18 Rust rendered-page tests |
| 2026-05-26 | OMAVM-040 | `cargo nextest run -p nako-metadata-scraper config registry manifest av --no-fail-fast` | Pass: 48 tests |
| 2026-05-26 | OMAVM-050 | `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `npm --prefix addons/browser-worker test`; `python -m json.tool docs/workstreams/official-metadata-addon-av-mdcx-parity/WORKSTREAM.json`; `python -m json.tool addons/metadata-scraper/manifest.example.json`; `git diff --check` | Pass: 183 Rust tests passed, 2 skipped; 4 browser-worker tests passed; JSON and diff checks passed |
