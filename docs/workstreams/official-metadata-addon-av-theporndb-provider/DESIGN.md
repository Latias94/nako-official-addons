# Official Metadata Addon AV ThePornDB Provider

Status: Active
Last updated: 2026-05-26

## Problem

The AV scraper now covers several Japanese official and community sources, but it still lacks a token-backed API provider for western and cross-site scene metadata. MDCx treats ThePornDB as a high-value western source and uses both API search and hash lookup. Nako needs the same class of provider without copying MDCx implementation details or leaking secrets through diagnostics.

## Reference Boundary

`repo-ref/mdcx` remains GPLv3/reference-only. This lane uses it only for behavior-level strategy: ThePornDB is token-backed, supports scene/movie API flows, supports hash lookup, and is valuable for western provider presets. Selectors, source structure, comments, fixtures, and concrete implementation are not copied.

The public ThePornDB OpenAPI spec is used as the primary API contract for endpoint names, authentication shape, request parameters, and response field names.

## Target Shape

- Add a disabled-by-default `theporndb` provider using `ProviderHttpRuntime`.
- Require `NAKO_METADATA_SCRAPER_THEPORNDB_API_TOKEN` before building an enabled provider.
- Support AV/title lookup through `/scenes` with `parse`, `sku`, or `q` style parameters.
- Support direct scene lookup through external IDs and public/API URLs when a slug is supplied.
- Map title, overview, release date, runtime, rating, performers, directors, site/studio, tags, poster/background/trailer, and external IDs into the shared candidate/fact model.
- Expose redaction-safe diagnostics for token/proxy configuration only.
- Add ThePornDB to field policy and western/community presets where it improves coverage without breaking manual defaults.
- Document token, proxy, and hash-roadmap behavior.

## Scope

- `crates/nako-metadata-scraper/src/providers/theporndb.rs`
- `crates/nako-metadata-scraper/src/providers/mod.rs`
- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/manifest.rs`
- `crates/nako-metadata-scraper/tests/live_provider_drift.rs`
- `crates/nako-metadata-scraper/README.md`
- `addons/metadata-scraper/README.md`
- `addons/metadata-scraper/manifest.example.json`
- Workstream docs under this directory.

## Non-Goals

- No local file hashing in this slice. The current scrape request model does not provide stable file bytes or hashes to providers.
- No raw live ThePornDB payloads in committed fixtures.
- No browser-worker integration. ThePornDB is an API provider.
- No actor database side effects or image downloading.

## Architecture Direction

The provider should mirror the Prestige shape: thin config/catalog integration, an HTTP-runtime-backed provider, serde response structs, and a focused mapper from provider JSON into `ProviderMetadataCandidate`. Direct IDs are stronger than inferred AV/title search. If no direct slug exists, use AV query facts first and fallback to title text.

The hash API should be documented as a follow-up until the core query model has a redaction-safe hash input contract. This avoids a half-implemented local file dependency in provider code.
