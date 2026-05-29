# Official Resource Search PanSou-Compatible Provider

## Problem

The resource search foundation currently proves the sidecar contract with a
fixture provider. The next useful step is to connect that contract to an
external multi-source search aggregator without copying third-party source code
or committing Nako to a downloader policy.

PanSou exposes a compact `/api/search` JSON contract that already models
keyword search, plugin selection, cloud-drive type filtering, raw results, and
merged links. The official addon should be able to consume a PanSou-compatible
HTTP service as an optional provider.

## Target State

- `nako-resource-search` has a disabled-by-default
  `pansou_compatible` provider.
- Operators enable it only by setting an explicit provider flag and base URL.
- The provider posts to `/api/search` and maps PanSou results into the
  sidecar's internal resource search DTOs.
- Tests cover request shaping, response mapping, token redaction, and provider
  enablement without requiring a live PanSou service.

## Scope

- Extend resource-search config with PanSou-compatible provider settings.
- Add a reqwest-backed provider adapter.
- Map PanSou `results` and fallback `merged_by_type` payloads into
  `ResourceSearchResult` and `ResourceLink`.
- Update manifest configuration schema, README, and evidence.

## Non-goals

- Running PanSou inside this addon.
- Copying PanSou source code.
- Live PanSou CI.
- Link availability checks.
- Downloader or BitTorrent invocation.
- Nako core protocol changes.

## Decisions

- Keep the provider disabled by default. A missing or blank base URL never
  triggers network calls.
- Use bearer authorization only when `NAKO_RESOURCE_SEARCH_PANSOU_TOKEN` is set,
  and never print the token in debug, health, diagnostics, or test output.
- Prefer `res=results` so mapping preserves title/content/source provenance.
  If a compatible service returns only `merged_by_type`, synthesize grouped
  search results with safe titles.
