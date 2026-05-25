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

## Completed

OMAPAD-020 completed:

- `ProviderCatalogEntry` now carries default enablement, enablement env var,
  provider config loader, and proxy health fact function.
- `Config` builds provider configs from the provider catalog instead of
  hard-coding `ProviderId::ALL` and every provider-specific config branch.
- Targeted provider/config/manifest tests, format check, and diff hygiene
  passed.

OMAPAD-030 completed:

- `ProviderRegistry` exposes one `ProviderAssembly` for ready adapters,
  diagnostics, and provider-owned network policy facts.
- Routes consume provider diagnostics instead of reading provider-specific
  proxy facts from `Config`.

OMAPAD-040 completed:

- `providers/search_policy.rs` owns the common direct-lookup, title-variant
  search, dedupe, ranking-budget, partial-search, and degraded-fallback
  orchestration.
- TMDB and Bangumi pass provider-local callbacks for HTTP search, raw response
  parsing, result IDs, enrichment, degraded candidate mapping, and provider
  notes.
- The shared policy is closure-driven so raw TMDB/Bangumi result types are not
  promoted into a public shared trait contract.

## Next Task

Start OMAPAD-050.

Recommended next implementation focus:

- replace provider-local `provider_note` prose with internal typed outcome
  facts;
- keep the public payload shape compatible unless an intentional schema change
  is documented;
- preserve redaction-safe rendering and current TMDB/Bangumi/Douban diagnostic
  text semantics while moving note composition behind one Module.

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
OMAPAD-040 passed with `cargo fmt --all -- --check`, `cargo nextest run -p nako-metadata-scraper tmdb bangumi relevance partial degraded --no-fail-fast`, and `git diff --check`.
