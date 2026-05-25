# Official Metadata Addon Provider Extension Decentralization - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Gate Plan

| Gate | Command | When |
| --- | --- | --- |
| Workstream docs | `python -m json.tool docs/workstreams/official-metadata-addon-provider-extension-decentralization/WORKSTREAM.json` | OMAPED-010 |
| Provider config/manifest | `cargo nextest run -p nako-metadata-scraper config manifest provider registry --no-fail-fast` | OMAPED-020 |
| External ID aliases | `cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker --no-fail-fast` | OMAPED-030 |
| Rendered-page support | `cargo nextest run -p nako-metadata-scraper browser_worker douban rendered --no-fail-fast` | OMAPED-040 |
| Package gate | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | OMAPED-050 and closeout |
| Format | `cargo fmt --all -- --check` | Before each commit and closeout |
| Diff hygiene | `git diff --check` | Before each commit and closeout |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | OMAPED-010 planning | Opened this workstream from the user-approved follow-on provider extension refactor Goal. Validated `WORKSTREAM.json` with `python -m json.tool docs/workstreams/official-metadata-addon-provider-extension-decentralization/WORKSTREAM.json`; `cargo fmt --all -- --check`; `git diff --check`. | Pass |

## Notes

- Keep public payloads, env vars, manifest defaults, and default-off provider
  behaviour compatible unless a later task explicitly records otherwise.
- Live provider drift checks remain opt-in and out of scope for this lane.
