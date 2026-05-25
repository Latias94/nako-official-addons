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

## Next Task

Start OMAPED-020.

Recommended next implementation focus:

- begin replacing the provider config optional-field matrix with a typed
  provider config Interface.

## Risks

- Moving config shapes can accidentally change env var defaults or manifest
  secret reference ordering.
- External ID alias extraction must not couple `engine::query` directly to
  provider implementation modules.
- Douban's rendered-page support should become clearer without demoting the
  explicit `browser_worker` provider semantics.

## Validation Memory

OMAPED-010 passed with `python -m json.tool docs/workstreams/official-metadata-addon-provider-extension-decentralization/WORKSTREAM.json`, `cargo fmt --all -- --check`, and `git diff --check`.
