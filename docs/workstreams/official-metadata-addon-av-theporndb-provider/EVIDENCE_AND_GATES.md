# Evidence And Gates

Status: Complete
Last updated: 2026-05-26

## Required Gates

- Workstream JSON: `python -m json.tool docs/workstreams/official-metadata-addon-av-theporndb-provider/WORKSTREAM.json`
- Targeted provider tests: `cargo nextest run -p nako-metadata-scraper theporndb --no-fail-fast`
- Targeted integration tests: `cargo nextest run -p nako-metadata-scraper config registry manifest av_provider_preset field_policy av_drift --no-fail-fast`
- Package gate: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- Format gate: `cargo fmt -p nako-metadata-scraper -- --check`
- Diff hygiene: `git diff --check`

## Evidence Log

- PASS 2026-05-26: `python -m json.tool docs/workstreams/official-metadata-addon-av-theporndb-provider/WORKSTREAM.json`
- PASS 2026-05-26: `python -m json.tool addons/metadata-scraper/manifest.example.json`
- PASS 2026-05-26: `cargo nextest run -p nako-metadata-scraper theporndb --no-fail-fast`
- PASS 2026-05-26: `cargo nextest run -p nako-metadata-scraper theporndb config registry manifest av_provider_preset field_policy av_drift health_endpoint --no-fail-fast` (41 passed)
- PASS 2026-05-26: `cargo nextest run -p nako-metadata-scraper --no-fail-fast` (235 passed, 3 skipped)
- PASS 2026-05-26: `cargo fmt -p nako-metadata-scraper -- --check`
- PASS 2026-05-26: `git diff --check`
