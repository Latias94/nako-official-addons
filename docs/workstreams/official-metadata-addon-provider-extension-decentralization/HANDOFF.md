# Official Metadata Addon Provider Extension Decentralization - Handoff

Status: Active
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

## Next Task

Start OMAPED-030.

Recommended next implementation focus:

- move top-level external ID alias declarations and known numeric validation
  into provider-owned descriptors or a provider extension seam.

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
