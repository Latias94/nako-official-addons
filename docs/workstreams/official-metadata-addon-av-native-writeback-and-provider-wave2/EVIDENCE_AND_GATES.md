# Evidence And Gates

Status: Closed
Last updated: 2026-05-26

## Required Gates

| Gate | Command | Required before |
| --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2/WORKSTREAM.json` | OMAV2-010 completion |
| Workstream hygiene | `git diff --check` | OMAV2-010 completion and closeout |
| Nako protocol/writeback | `cargo nextest run -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon --no-fail-fast` | OMAV2-020 completion |
| Nako server writeback | `cargo nextest run -p nako-server addon_side_effect_metadata_write --no-fail-fast` | OMAV2-020 completion |
| Nako focused format | `cargo fmt -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon -p nako-server -- --check` | OMAV2-020 completion and closeout |
| Addon AV materialization | `cargo nextest run -p nako-metadata-scraper av field_policy resolver writeback javdb dmm fc2 javbus --no-fail-fast` | OMAV2-030 completion |
| Bulk maturity | `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast` | OMAV2-040 completion |
| Provider wave 2 | `cargo nextest run -p nako-metadata-scraper config registry manifest av javlibrary --no-fail-fast` | OMAV2-050 completion |
| Official addon package format | `cargo fmt -p nako-metadata-scraper -- --check` | closeout |
| Browser worker regression | `npm --prefix addons/browser-worker test` | closeout |
| Official addon package validation | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | closeout |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-26 | OMAV2-010 | `python -m json.tool docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2/WORKSTREAM.json`; `git diff --check` | Pass |
| 2026-05-26 | OMAV2-020 | `cargo nextest run -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon --no-fail-fast`; `cargo nextest run -p nako-server addon_side_effect_metadata_write --no-fail-fast`; `cargo fmt -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon -p nako-server -- --check` | Pass: 30 protocol/client/reference tests; 4 server metadata_write tests; Nako commit `a0ad9a8` |
| 2026-05-26 | OMAV2-030 | `cargo nextest run -p nako-metadata-scraper av field_policy resolver writeback javdb dmm fc2 javbus --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2/WORKSTREAM.json`; `git diff --check` | Pass: 36 related tests |
| 2026-05-26 | OMAV2-040 | `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check` | Pass: 11 bulk tests; provider suppression/cooldown state remains explicit in `resume_state` |
| 2026-05-26 | OMAV2-050 | `cargo nextest run -p nako-metadata-scraper config registry manifest av javlibrary --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check` | Pass: 54 filtered tests; JavLibrary and MGStage are disabled-by-default rendered AV providers |
| 2026-05-26 | OMAV2-060 | `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `npm --prefix addons/browser-worker test`; `cargo nextest run -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon --no-fail-fast` in `../nako`; `cargo nextest run -p nako-server addon_side_effect_metadata_write --no-fail-fast` in `../nako`; `cargo fmt -p nako-addon-protocol -p nako-addon-client -p nako-reference-addon -p nako-server -- --check` in `../nako`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-native-writeback-and-provider-wave2/WORKSTREAM.json`; `git diff --check` | Pass: 192 metadata-scraper tests, 4 browser-worker tests, 30 Nako protocol/client/reference tests, 4 Nako server writeback tests, focused Nako and addon formatting, valid workstream JSON, and clean diff whitespace |
