# Evidence And Gates

Status: Active
Last updated: 2026-05-26

## Required Gates

| Gate | Command | Required before |
| --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave3/WORKSTREAM.json` | OMAV3-010 and closeout |
| Workstream hygiene | `git diff --check` | every task completion |
| Provider fixture harness | `cargo nextest run -p nako-metadata-scraper rendered_av provider_fixture av --no-fail-fast` | OMAV3-020 |
| Provider protection | `cargo nextest run -p nako-metadata-scraper provider_guard bulk runtime provider_execution --no-fail-fast` | OMAV3-030 |
| Prestige provider | `cargo nextest run -p nako-metadata-scraper prestige config registry manifest av --no-fail-fast` | OMAV3-040 |
| FC2 long-tail provider | `cargo nextest run -p nako-metadata-scraper fc2 av config registry manifest --no-fail-fast` | OMAV3-050 |
| Uncensored provider trio | `cargo nextest run -p nako-metadata-scraper caribbean 1pondo 10musume av config registry manifest --no-fail-fast` | OMAV3-060 |
| Package validation | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | closeout |
| Browser-worker validation | `npm --prefix addons/browser-worker test` | closeout if browser-worker changed |
| Formatting | `cargo fmt -p nako-metadata-scraper -- --check` | every Rust task completion |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | OMAV3-010 | `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave3/WORKSTREAM.json`; `git diff --check` | Pass: workstream opened |
