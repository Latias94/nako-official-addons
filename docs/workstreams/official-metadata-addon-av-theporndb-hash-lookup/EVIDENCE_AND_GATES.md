# Evidence And Gates

Status: Complete
Last updated: 2026-05-26

## Required Gates

- Workstream JSON: `python -m json.tool docs/workstreams/official-metadata-addon-av-theporndb-hash-lookup/WORKSTREAM.json`
- Targeted tests: `cargo nextest run -p nako-metadata-scraper theporndb registry_exposes_provider_external_id_capabilities registry_derives_legacy_external_id_aliases_from_capabilities --no-fail-fast`
- Package gate: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- Format gate: `cargo fmt -p nako-metadata-scraper -- --check`
- Diff hygiene: `git diff --check`

## Evidence Log

- PASS 2026-05-26: `python -m json.tool docs/workstreams/official-metadata-addon-av-theporndb-hash-lookup/WORKSTREAM.json`
- PASS 2026-05-26: `cargo nextest run -p nako-metadata-scraper theporndb registry_exposes_provider_external_id_capabilities registry_derives_legacy_external_id_aliases_from_capabilities registry_reports_redaction_safe_provider_diagnostics --no-fail-fast` (10 passed)
- PASS 2026-05-26: `cargo nextest run -p nako-metadata-scraper --no-fail-fast` (237 passed, 3 skipped)
- PASS 2026-05-26: `cargo fmt -p nako-metadata-scraper -- --check`
- PASS 2026-05-26: `git diff --check`
