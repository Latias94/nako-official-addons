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
| 2026-05-26 | OMAV3-020 | `cargo nextest run -p nako-metadata-scraper rendered_av provider_fixture av --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `git diff --check` | Pass: shared rendered AV fixture covers DMM, FC2, JavDB, JavBus, JavLibrary, and MGStage tests |
| 2026-05-26 | OMAV3-030 | `cargo nextest run -p nako-metadata-scraper provider_guard bulk runtime provider_execution --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `git diff --check` | Pass: request/config-visible provider budgets, bounded bulk cache/cooldown state, and redaction-safe proxy/session diagnostics |
| 2026-05-26 | OMAV3-040 | `cargo nextest run -p nako-metadata-scraper prestige --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper prestige config registry manifest av --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper proxy routes --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave3/WORKSTREAM.json`; `git diff --check` | Pass: Prestige official JSON API provider is disabled by default, supports censored AV search/direct lookup, emits declared external IDs, updates field policy/schema/docs, and reports proxy policy without URL leakage |
| 2026-05-26 | OMAV3-050 | `cargo nextest run -p nako-metadata-scraper fc2ppvdb --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper fc2 av config registry manifest --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave3/WORKSTREAM.json`; `git diff --check` | Pass: FC2PPVDB selected over FC2Hub/FC2Club for deterministic long-tail FC2 article fallback, disabled by default, route-gated to FC2, and emits declared FC2PPVDB/AV external IDs |
