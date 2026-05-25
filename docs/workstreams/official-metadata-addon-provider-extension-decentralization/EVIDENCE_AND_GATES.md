# Official Metadata Addon Provider Extension Decentralization - Evidence And Gates

Status: Complete
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
| 2026-05-25 | OMAPED-020 provider config decentralization | Replaced `ProviderConfig`'s provider-specific optional-field matrix with typed `ProviderConfigKind` variants and moved TMDB, Bangumi, browser_worker, and Douban config structs into their provider modules with central re-exports for compatibility. Ran `cargo nextest run -p nako-metadata-scraper config manifest provider registry --no-fail-fast` with 94 tests run, 94 passed, and 51 skipped; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPED-030 provider-owned external ID aliases | Added `QueryExternalIdAlias` descriptors, made provider catalog entries contribute top-level alias and numeric validation facts, and passed aliases from `ProviderRegistry` into `MetadataScrapeRuntime` without coupling query parsing to provider implementation modules. Ran `cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker --no-fail-fast` with 76 tests run, 76 passed, and 70 skipped; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPED-040 rendered-page support semantics | Added shared `RenderedPageSupportConfig` and made browser_worker plus Douban hold rendered-page support config explicitly while preserving existing browser-worker env vars. Douban remains a browser-rendered provider and `browser_worker` remains the default-off metadata provider for explicit rendered-page URL extraction. Ran `cargo nextest run -p nako-metadata-scraper browser_worker douban rendered --no-fail-fast` with 4 tests run, 4 passed, and 142 skipped; `cargo fmt --all -- --check`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPED-050 cleanup and integration | Ran the full metadata scraper package gate after OMAPED-020 through OMAPED-040. `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with 144 tests run, 144 passed, and 2 skipped. Updated metadata scraper README docs to mention explicit `external_ids` and top-level aliases `tmdb_id`, `imdb_id`, `bangumi_id`, and `browser_worker_url`. `cargo fmt --all -- --check`, `python -m json.tool docs/workstreams/official-metadata-addon-provider-extension-decentralization/WORKSTREAM.json`, and `git diff --check` passed. | Pass |
| 2026-05-25 | OMAPED-060 closeout | Review found no blocking workstream compliance or code-quality findings. Removed the misleading `browser_worker_id` top-level alias during closeout review because browser worker external values are URLs, not IDs. Final gates passed: `cargo fmt --all -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-provider-extension-decentralization/WORKSTREAM.json`; `git diff --check`; `cargo nextest run -p nako-metadata-scraper --no-fail-fast` with 144 tests run, 144 passed, and 2 skipped. | Pass |

## Notes

- Keep public payloads, env vars, manifest defaults, and default-off provider
  behaviour compatible unless a later task explicitly records otherwise.
- Live provider drift checks remain opt-in and out of scope for this lane.
- Closeout did not split follow-ons because provider config, external ID alias,
  rendered-page support, docs, and integration targets are complete.
