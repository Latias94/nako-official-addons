# Official Metadata Addon Provider Extension Decentralization - Handoff

Status: Complete
Last updated: 2026-05-25

## Current State

The user approved a follow-on fearless refactor after the provider architecture
deepening lane closed. This lane focuses on remaining provider extension costs:

1. Provider config decentralization.
2. Provider-owned external ID aliases.
3. Explicit browser-rendered support semantics for Douban and browser_worker.
4. Cleanup of stale tests/docs discovered during the work.

## Completed

OMAPED-010 completed:

- Workstream docs were created and agree on the provider extension
  decentralization target state.
- Release publishing and live smoke remain explicitly out of scope.
- JSON, format, and diff hygiene gates passed.

OMAPED-020 completed:

- `ProviderConfig` no longer carries `tmdb`, `bangumi`, `browser_worker`, and
  `douban` optional fields on every row.
- Provider config is now represented by typed `ProviderConfigKind` variants.
- TMDB, Bangumi, browser_worker, and Douban config structs live in their
  provider modules, with central re-exports to preserve existing call sites.
- Manifest defaults, secret references, provider enablement, and env var parsing
  stayed compatible under the targeted gate.

OMAPED-030 completed:

- Added `QueryExternalIdAlias` descriptors to the query parsing Interface.
- Provider catalog entries now contribute `tmdb_id`, `imdb_id`, `bangumi_id`,
  and browser-worker rendered-page URL aliases.
- `MetadataScrapeRuntime` receives aliases from `ProviderRegistry`, so
  `engine::query` does not import provider implementation modules.
- Existing explicit external ID payloads and top-level aliases still parse, and
  known numeric providers still reject non-positive numeric IDs.

OMAPED-040 completed:

- Added shared `RenderedPageSupportConfig` in the rendered-page support Module.
- `browser_worker` and Douban provider configs now hold rendered-page support
  config explicitly instead of duplicating base URL and timeout fields.
- Douban remains represented as a browser-rendered provider.
- `browser_worker` remains a real default-off metadata provider for explicit
  rendered-page URL extraction.
- Existing browser-worker env var names and defaults are preserved.

OMAPED-050 completed:

- Full `nako-metadata-scraper` package gate passed after the provider config,
  external ID alias, and rendered-page support refactors.
- `crates/nako-metadata-scraper/README.md` and
  `addons/metadata-scraper/README.md` now mention explicit `external_ids` and
  top-level aliases, including `browser_worker_url`.

## Next Task

No next task in this lane.

Closeout result:

- Review found no blocking workstream compliance or code-quality findings.
- Final package, format, JSON, and diff hygiene gates passed.
- The misleading `browser_worker_id` top-level alias was removed during
  closeout review; `browser_worker_url` remains the documented alias.
- No architecture follow-on was split; release publishing and live provider
  smoke remain out of scope.

## Risks

- Moving config shapes can accidentally change env var defaults or manifest
  secret reference ordering.
- External ID alias extraction must not couple `engine::query` directly to
  provider implementation modules.
- Douban's rendered-page support should become clearer without demoting the
  explicit `browser_worker` provider semantics.

## Validation Memory

OMAPED-010 passed with `python -m json.tool docs/workstreams/official-metadata-addon-provider-extension-decentralization/WORKSTREAM.json`, `cargo fmt --all -- --check`, and `git diff --check`.
OMAPED-020 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper config manifest provider registry --no-fail-fast`, and `git diff --check`.
OMAPED-030 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper external_id tmdb bangumi browser_worker --no-fail-fast`, and `git diff --check`.
OMAPED-040 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper browser_worker douban rendered --no-fail-fast`, and `git diff --check`.
OMAPED-050 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper --no-fail-fast`, and `git diff --check`.
OMAPED-060 passed with `cargo fmt --all -- --check`, `python -m json.tool docs/workstreams/official-metadata-addon-provider-extension-decentralization/WORKSTREAM.json`, `cargo nextest run -p nako-metadata-scraper --no-fail-fast`, and `git diff --check`.
