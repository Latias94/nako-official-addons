# Evidence And Gates

Status: Complete
Last updated: 2026-05-26

## Required Gates

| Gate | Command | Required before |
| --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-av-parser-quality/WORKSTREAM.json` | APQ-010 and closeout |
| Workstream hygiene | `git diff --check` | every task completion |
| Structured labels | `cargo nextest run -p nako-metadata-scraper rendered_av official_uncensored fc2ppvdb --no-fail-fast` | APQ-010 |
| Provider migration | `cargo nextest run -p nako-metadata-scraper dmm mgstage javbus javlibrary javdb fc2 rendered_av --no-fail-fast` | APQ-020/APQ-030 |
| Package validation | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | closeout |
| Formatting | `cargo fmt -p nako-metadata-scraper -- --check` | every Rust task completion |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | APQ-000 | `python -m json.tool docs/workstreams/official-metadata-addon-av-parser-quality/WORKSTREAM.json`; `git diff --check` | Pass: workstream opened from user-approved parser-quality goal |
| 2026-05-26 | APQ-010 | `cargo nextest run -p nako-metadata-scraper rendered_av official_uncensored fc2ppvdb --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-parser-quality/WORKSTREAM.json`; `git diff --check` | Pass: shared structured label helper prevents row-boundary bleed and is used by official uncensored plus FC2PPVDB with provider-local row selectors |
| 2026-05-26 | APQ-020 | `cargo nextest run -p nako-metadata-scraper dmm mgstage javbus rendered_av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check` | Pass: DMM, MGStage, and JavBus use shared structured label fallback with provider-local row selectors and focused facts assertions |
| 2026-05-26 | APQ-030 | `cargo nextest run -p nako-metadata-scraper javlibrary javdb fc2 rendered_av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check` | Pass: JavLibrary, JavDB, and FC2 use shared structured label fallback for row-like metadata while keeping provider-specific text/link/image helpers local |
| 2026-05-26 | APQ-040 | `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-parser-quality/WORKSTREAM.json`; `git diff --check` | Pass: closeout verified 222 metadata-scraper tests, formatting, JSON, and diff hygiene; parser-quality behavior is documented |
