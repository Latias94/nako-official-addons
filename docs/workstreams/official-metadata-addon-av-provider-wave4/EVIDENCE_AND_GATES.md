# Evidence And Gates

Status: Active
Last updated: 2026-05-26

## Required Gates

- Targeted provider tests: `cargo nextest run -p nako-metadata-scraper airav avsox xcity --no-fail-fast`
- Targeted integration tests: `cargo nextest run -p nako-metadata-scraper manifest av_provider_preset field_policy --no-fail-fast`
- Package gate: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- Format gate: `cargo fmt -p nako-metadata-scraper -- --check`
- Docs/data gate: `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave4/WORKSTREAM.json`
- Diff hygiene: `git diff --check`

## Evidence Log

- PASS on 2026-05-26: `python -m json.tool docs/workstreams/official-metadata-addon-av-provider-wave4/WORKSTREAM.json`
- PASS on 2026-05-26: `cargo nextest run -p nako-metadata-scraper airav avsox xcity --no-fail-fast`
- PASS on 2026-05-26: `cargo nextest run -p nako-metadata-scraper av_drift config manifest airav avsox xcity registry --no-fail-fast` with 36 tests.
- PASS on 2026-05-26: `cargo fmt -p nako-metadata-scraper -- --check`
- PASS on 2026-05-26: `git diff --check`
- PASS on 2026-05-26: `cargo nextest run -p nako-metadata-scraper --no-fail-fast` with 230 passed, 3 skipped.
