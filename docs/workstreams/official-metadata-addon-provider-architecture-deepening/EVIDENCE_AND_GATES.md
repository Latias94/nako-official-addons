# Official Metadata Addon Provider Architecture Deepening - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Gate Plan

| Gate | Command | When |
| --- | --- | --- |
| Workstream docs | `python -m json.tool docs/workstreams/official-metadata-addon-provider-architecture-deepening/WORKSTREAM.json > $null` | OMAPAD-010 |
| Provider config/registry/manifest | `cargo nextest run -p nako-metadata-scraper provider registry config addon_manifest --no-fail-fast` | OMAPAD-020 |
| Provider assembly/routes | `cargo nextest run -p nako-metadata-scraper provider health_endpoint diagnostics --no-fail-fast` | OMAPAD-030 |
| Search policy | `cargo nextest run -p nako-metadata-scraper tmdb bangumi relevance partial degraded --no-fail-fast` | OMAPAD-040 |
| Provider outcomes | `cargo nextest run -p nako-metadata-scraper provider_note redaction ranking tmdb bangumi douban --no-fail-fast` | OMAPAD-050 |
| Rendered-page support | `cargo nextest run -p nako-metadata-scraper browser_worker douban --no-fail-fast` | OMAPAD-060 |
| Package gate | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | OMAPAD-070 |
| Format | `cargo fmt --all -- --check` | Before each commit and closeout |
| Diff hygiene | `git diff --check` | Before each commit and closeout |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | OMAPAD-010 planning | Opened this workstream from the architecture review and user-approved five-refactor Goal. Validated `WORKSTREAM.json` with `python -m json.tool docs/workstreams/official-metadata-addon-provider-architecture-deepening/WORKSTREAM.json > $null`. | Pass |
| 2026-05-25 | OMAPAD-020 provider descriptor ownership | Moved provider default enablement, enablement env vars, provider config loading, and proxy health facts into provider catalog entries. Ran `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper provider registry config addon_manifest --no-fail-fast`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPAD-030 single provider assembly | Added `ProviderAssembly`, made registry providers/diagnostics derive from assembly, and moved route network policy output to provider diagnostics. Ran `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper provider health_endpoint diagnostics --no-fail-fast`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPAD-040 shared search policy | Added `providers/search_policy.rs` for direct lookup, title-variant search, dedupe, ranking-budget selection, partial-search preservation, and degraded fallback orchestration. TMDB/Bangumi adapters now pass provider-local HTTP/search/enrichment/degraded callbacks into the shared policy. Ran `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper tmdb bangumi relevance partial degraded --no-fail-fast`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPAD-050 typed provider outcomes | Added `engine/outcome.rs` with typed `ProviderOutcome` facts and one redaction-safe provider note renderer. TMDB, Bangumi, Douban, Fixture, BrowserWorker, and shared search policy now emit outcomes instead of provider-local diagnostic prose. Ran `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper provider_note redaction ranking tmdb bangumi douban --no-fail-fast`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPAD-060 rendered-page support seam | Added `providers/rendered_page.rs` as the browser-worker protocol runtime. Douban now depends on rendered-page support instead of owning worker request/response parsing; `browser_worker` remains in the provider catalog as a default-off metadata provider for explicit rendered-page URL extraction. Ran `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper browser_worker douban --no-fail-fast`; `git diff --check`. | Pass |
| 2026-05-25 | OMAPAD-070 integration gate | Ran the full metadata scraper package gate after OMAPAD-020 through OMAPAD-060. `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with 143 tests run, 143 passed, and 2 skipped. `cargo fmt --all -- --check` and `git diff --check` passed. Reviewed `crates/nako-metadata-scraper/README.md` and `addons/metadata-scraper/README.md`; public provider strategy/config docs already cover browser-worker rendered-page extraction and Douban's rendered HTML dependency. | Pass |

## Notes

- Prefer targeted `cargo nextest` filters while working on a slice.
- Broader package and formatting gates are required before closeout.
- Live provider drift checks remain opt-in and out of scope for this lane.
