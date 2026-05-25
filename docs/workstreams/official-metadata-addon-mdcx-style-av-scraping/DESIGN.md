# Official Metadata Addon MDCx-Style AV Scraping

Status: Active
Last updated: 2026-05-26

## Problem

The metadata scraper can already run bounded batch metadata tasks, but its query model is still movie/anime-oriented. AV libraries are commonly file-name and number driven, so generic title search loses too much signal before providers run. The sidecar also lacks AV-specific diagnostics that explain how each bulk item was interpreted.

MDCx shows the mature shape of this domain: clean file names before search, normalize AV numbers, route numbers by family, split search and detail fetches, let one provider fail without blocking a batch, and keep enough per-field/source evidence to debug batch runs.

## Reference Boundary

`repo-ref/mdcx` is GPLv3 plus project-specific distribution terms. It is used only as behavioral reference. This workstream must not copy MDCx source, comments, fixtures, selector implementations, or project structure. Any code added here must be original Rust code that follows this repository's local provider and runtime abstractions.

## Target Shape

- Add an AV query facts layer that extracts a normalized AV number from `av_number`, `number`, `file_name`, `filename`, `path`, `title`, or `name`.
- Classify extracted numbers into stable routing families: `censored`, `uncensored`, `fc2`, `amateur`, `western`, `domestic`, or `unknown`.
- Echo redaction-safe AV query facts in normal scrape output and bulk task item output, without echoing full local file paths.
- Add a disabled-by-default browser-worker-backed JavDB provider baseline using the existing rendered-page runtime.
- Make JavDB search by normalized AV number, parse search/detail pages separately, and emit external IDs for `javdb`, `javdb_url`, and `av_number`.
- Keep Nako-owned scheduling/retry/cancel semantics by extending the existing `bulk-metadata-scrape` task instead of adding a second batch executor.

## Scope

- `crates/nako-metadata-scraper/src/engine`: AV number recognition, query facts, response shaping, bulk item summaries.
- `crates/nako-metadata-scraper/src/providers`: JavDB provider catalog entry and rendered HTML implementation.
- `crates/nako-metadata-scraper/src/config.rs`: JavDB configuration, environment variables, diagnostics.
- `crates/nako-metadata-scraper/README.md`: AV request fields, provider enablement, and bulk output behavior.
- Workstream evidence and handoff docs.

## Non-Goals

- No local file renaming, moving, NFO writing, image downloading, or actor photo database.
- No full MDCx-style multi-source field reducer in the first slice.
- No live network gate for JavDB; tests use synthetic rendered HTML fixtures.
- No cookie/session handling until the browser worker has an explicit contract for it.

## Assumptions

| Assumption | Confidence | Notes |
| --- | --- | --- |
| Browser worker render support is the right integration layer for JavDB-style pages. | Medium | Douban already proves the local abstraction. |
| Existing bulk task should stay the only batch executor. | High | Nako owns scheduling, retry, cancel, and progress. |
| A single JavDB baseline plus AV query facts is a valid first vertical slice. | High | Multi-provider routing can build on the same facts later. |
| Bulk diagnostics must not echo full paths. | High | AV file paths can contain sensitive local naming. |

## Design Notes

AV recognition belongs in `engine::query`, not in one provider. Providers should consume normalized facts and return external IDs. This keeps future providers such as FC2, JavBus-like, or DMM-like sources from each implementing incompatible file-name parsing.

JavDB is introduced disabled by default. Enabling it should require:

- `NAKO_METADATA_SCRAPER_PROVIDER_JAVDB_ENABLED=true`
- a reachable browser worker via `NAKO_METADATA_SCRAPER_BROWSER_WORKER_BASE_URL`

The first bulk upgrade is diagnostic: every processed item gets an optional `av` summary copied from the scrape response query. It is intentionally small and stable so core Nako task history can index it without provider-specific coupling.

