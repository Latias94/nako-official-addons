# Evidence And Gates

Status: Complete
Last updated: 2026-05-26

## Required Gates

| Gate | Command | Status | Evidence |
| --- | --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-policy/WORKSTREAM.json` | Passed | JSON parsed successfully. |
| Documentation hygiene | `git diff --check` | Passed | No whitespace errors after README updates. |
| Field policy tests | `cargo nextest run -p nako-metadata-scraper field_policy resolver ranking --no-fail-fast` | Passed | 20 passed after adding default AV policy and artwork source selection. |
| Provider targeted tests | `cargo nextest run -p nako-metadata-scraper dmm --no-fail-fast` | Passed | 3 passed. |
| Registry/config tests | `cargo nextest run -p nako-metadata-scraper config registry manifest --no-fail-fast` | Passed | 27 passed after adding DMM diagnostics expectations. |
| Package tests | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Passed | 180 passed, 2 skipped after default AV policy follow-up. |
| Formatting | `cargo fmt -p nako-metadata-scraper -- --check` | Passed | Formatting check passed after applying `cargo fmt -p nako-metadata-scraper`. |
| Manifest JSON | `python -m json.tool addons/metadata-scraper/manifest.example.json` | Passed | Example manifest parsed successfully. |
| Diff hygiene | `git diff --check` | Passed | No whitespace errors. |

## Evidence Log

- 2026-05-26: Opened lane for AV contract docs, provider-field policy, and more
  AV provider coverage. Confirmed current `addons/browser-worker` exposes a
  Crawlee/Playwright-backed `/render` contract and should own browser execution.
- 2026-05-26: `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-policy/WORKSTREAM.json` passed.
- 2026-05-26: Updated metadata-scraper and browser-worker READMEs for AV direct IDs, provider execution, bulk resume/failure summaries, and Crawlee ownership. `git diff --check` passed.
- 2026-05-26: Added request-level `provider_field_policy` for field selection inside merged provider clusters. `cargo nextest run -p nako-metadata-scraper field_policy resolver ranking --no-fail-fast` passed with 19 tests before the default AV policy follow-up.
- 2026-05-26: Added default AV provider-field policy and poster/backdrop source selection. `cargo nextest run -p nako-metadata-scraper provider_field_policy --no-fail-fast` passed with 2 tests.
- 2026-05-26: Re-ran field policy/resolver/ranking filter after default policy follow-up. `cargo nextest run -p nako-metadata-scraper field_policy resolver ranking --no-fail-fast` passed with 20 tests.
- 2026-05-26: Added DMM as a disabled-by-default official censored-release AV provider tracer. It supports `dmm_id` and `dmm_url` direct lookup, normalized AV-number search, rendered detail parsing, provider diagnostics, and manifest/config schema integration. `cargo nextest run -p nako-metadata-scraper dmm --no-fail-fast` passed with 3 tests.
- 2026-05-26: Re-ran registry/config/manifest integration filter after adding DMM to health diagnostics expectations. `cargo nextest run -p nako-metadata-scraper config registry manifest --no-fail-fast` passed with 27 tests.
- 2026-05-26: Full package gate passed after default AV policy follow-up. `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with 180 tests and 2 skipped.
- 2026-05-26: Closeout hygiene passed: `cargo fmt -p nako-metadata-scraper -- --check`, `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-policy/WORKSTREAM.json`, `python -m json.tool addons/metadata-scraper/manifest.example.json`, and `git diff --check`.
