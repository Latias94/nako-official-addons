# Official Metadata Addon Provider Architecture Deepening - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

The user approved doing all five architecture candidates with fearless
refactoring for provider/plugin extensibility:

1. Provider descriptor ownership.
2. Shared search-enrichment policy.
3. Browser-worker support Seam.
4. Typed provider outcomes.
5. Single provider assembly.

This workstream is the authoritative execution lane for that Goal.

## Next Task

Start OMAPAD-040.

OMAPAD-020 completed:

- `ProviderCatalogEntry` now carries default enablement, enablement env var,
  provider config loader, and proxy health fact function.
- `Config` builds provider configs from the provider catalog instead of
  hard-coding `ProviderId::ALL` and every provider-specific config branch.
- Targeted provider/config/manifest tests, format check, and diff hygiene
  passed.

Recommended next implementation focus:

- extract the shared TMDB/Bangumi search-enrichment policy without moving raw
  provider payload parsing across the Seam;
- preserve direct ID lookup behavior, title-variant fallback, partial-search
  preservation, relevance budget ordering, and degraded candidate fallback;
- keep public payloads unchanged.

## Risks

- Provider config refactoring can accidentally change manifest defaults or
  secret reference ordering.
- Search policy extraction can overreach and pull raw TMDB/Bangumi payload
  shapes across the Seam. Keep raw parsing provider-local.
- Removing or demoting `browser_worker` provider identity may require docs and
  manifest example updates.
- Typed provider outcomes should preserve public payload compatibility unless a
  schema change is intentionally documented.

## Validation Memory

OMAPAD-010 passed with `python -m json.tool docs/workstreams/official-metadata-addon-provider-architecture-deepening/WORKSTREAM.json > $null`.
OMAPAD-020 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper provider registry config addon_manifest --no-fail-fast`, and `git diff --check`.
OMAPAD-030 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper provider health_endpoint diagnostics --no-fail-fast`, and `git diff --check`.
