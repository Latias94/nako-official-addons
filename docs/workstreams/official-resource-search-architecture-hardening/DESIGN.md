# Official Resource Search Architecture Hardening

## Problem

`nako-resource-search` now proves the first official search sidecar with two
adapters:

- `fixture`
- `pansou_compatible`

That is enough evidence that the provider seam is real. The next risk is
letting search grow by adding provider-specific fields, source policies, result
fusion, downloader hooks, and Nako protocol assumptions directly to
`engine.rs`, `domain.rs`, and `config.rs`.

Search must support multiple media-facing data sources, but the official addon
must not become a bundle of site-specific scrapers or downloader integrations.
The architecture needs to separate:

- search intent
- provider capability
- source trust
- external HTTP adapter contracts
- result normalization and fusion
- acquisition handoff
- optional link checking
- optional downloader hooks

## Target State

- Resource search has an explicit query intent model.
- Providers declare capabilities and source policy through a registry.
- Official providers and third-party providers have clear ownership rules.
- Fusion/ranking is a deep module with a small interface.
- Domain DTOs do not depend on link-classification implementation modules.
- Provider-specific config and manifest schema fragments live with providers.
- Nako core protocol requirements remain documented until the host lane starts.

## Official Vs Third-Party Boundary

Official addon should own:

- deterministic fixture provider for tests and smoke;
- generic external search adapters with operator-supplied endpoints, such as
  PanSou-compatible HTTP;
- stable link taxonomy and normalization;
- redaction-safe diagnostics and provider execution summaries;
- generic source policy and capability declarations;
- acquisition-candidate handoff contract after Nako core supports it;
- optional link-check task contracts after host policy exists.

Official addon should not own by default:

- site-specific cloud drive scraper implementations;
- Telegram channel indexing or channel-specific rules;
- cookie/login/captcha/browser-bypass providers;
- tracker-specific torrent scrapers;
- downloader client integrations such as qBittorrent, Transmission, aria2, or
  cloud-drive save APIs;
- region-specific or legally ambiguous provider defaults.

Those should be third-party addons or external services called through generic
official adapter contracts.

## Search Types To Support

The official search domain should model these intents:

- `free_text`: operator-entered query text.
- `media_title`: title/year/language/media kind from a Nako media item.
- `external_id`: known identifiers such as IMDb/TMDB/Bangumi where available.
- `exact_link`: classify or inspect a known resource URL.
- `refresh`: same intent, bypassing provider cache when a provider supports it.

Follow-on non-search flows:

- `resource_link_check`: availability/lock/unsupported checks.
- `acquisition_candidate_handoff`: host-approved conversion into Nako
  acquisition intake.
- `downloader_hook`: explicit operator-owned external action, never implied by
  search.

## Architecture Direction

Proposed module shape:

- `domain::query`: request DTOs, internal query intent, and media context.
- `domain::result`: result DTOs, provider execution DTOs, merged output DTOs.
- `links`: taxonomy, URL normalization, and link construction.
- `providers::registry`: provider descriptors, enabled provider assembly, and
  capability diagnostics.
- `providers::<adapter>`: adapter-owned config, request shaping, response
  mapping, and tests.
- `engine::orchestrator`: provider fan-out, source selection, timeout policy,
  and execution accounting.
- `engine::fusion`: deduplication, source provenance, grouping, and ranking.
- `source_policy`: official/third-party/external-service classification and
  default enablement rules.
- `manifest`: composed schema fragments from enabled provider descriptors.

## Deletion Plan

- Delete direct provider construction from `ResourceSearchRuntime::new`.
- Delete provider-specific config fields from top-level `Config` where they can
  move behind provider-owned config.
- Delete `domain` dependency on `links`; link construction should live in
  `links`, not DTO definitions.
- Delete fusion helpers from `engine.rs`; keep orchestration separate from
  ranking and grouping.
- Delete hard-coded provider schema blocks from `manifest.rs`; compose them
  from provider descriptors.

## Boundary Plan

Provider adapter interface should become:

- provider id
- descriptor/capabilities
- redaction-safe configuration status
- `search(query) -> ProviderSearchBatch`

`ProviderSearchBatch` should carry:

- provider id
- result list
- warnings or safe failure facts
- cache/finality hints when a provider supports fast partial search

The runtime should not know how PanSou maps `cloud_types`, how fixture slugs are
made, or how a future external service authenticates. It should only know the
provider descriptor and the search batch contract.

## Risk Plan

- Keep current alpha wire payload stable until the Nako `resource_search`
  protocol lands.
- Keep PanSou-compatible provider disabled by default.
- Do not add live site scraping or downloader calls in this lane.
- Preserve current smoke and manifest parity tests.
- Rollback signal: any change that makes default fixture smoke require network
  access is invalid.

## Nako Core Dependency

Still deferred:

- `AddonResource::ResourceSearch`
- `AddonScope::AcquisitionSearchRead`
- host DTOs for search request/response/link/result
- host worker support for bounded addon resource search calls
- optional write scope for acquisition candidate submission

This lane prepares the addon-side architecture so that host integration can be
small and explicit later.
