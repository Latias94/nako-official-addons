# Official Metadata Bangumi Provider Baseline

Status: Complete
Last updated: 2026-05-23

## Problem

The metadata scraper now has the provider registry, HTTP runtime, and
provider-neutral ranking model needed for real provider breadth. The next
valuable provider is Bangumi because it covers animation and ACG metadata that
TMDB does not model well. Bangumi is currently not a runtime-supported provider,
so the manifest must not advertise it and users cannot enable it.

## Target State

- Bangumi is a first-class provider ID in runtime config, manifest schema,
  diagnostics, and provider registry.
- Bangumi is disabled by default.
- Bangumi uses the shared `ProviderHttpRuntime`.
- The provider implements a bounded subject search plus detail enrichment
  baseline using official Bangumi API v0 endpoints.
- Provider output is normalized facts and metadata patches. The engine remains
  the sole owner of final confidence scoring and deterministic sorting.
- Default tests use synthetic fake HTTP transport only. No live Bangumi network
  calls are required for package or workspace gates.
- User-Agent is explicit and configurable because Bangumi asks non-browser API
  clients to identify developer/app/version.

## Official API Facts

Primary sources:

- https://github.com/bangumi/api
- https://raw.githubusercontent.com/bangumi/api/master/open-api/v0.yaml
- https://raw.githubusercontent.com/bangumi/api/master/docs-raw/user%20agent.md

Facts used by this workstream:

- `POST /v0/search/subjects` searches subjects with a JSON body containing
  `keyword`, optional `sort`, and optional `filter`.
- Subject search accepts `limit` and `offset` query parameters.
- `GET /v0/subjects/{subject_id}` fetches subject detail.
- Subject type `2` is anime; type `6` is real/live action.
- The subject schema exposes `id`, `type`, `name`, `name_cn`, `summary`,
  `date`, `platform`, `images`, `eps`, `total_episodes`, `rating`,
  `meta_tags`, and `tags`.
- Authentication is optional for these read paths, but may affect sensitive or
  NSFW visibility.

## Scope

In scope:

- Runtime config and environment variables for Bangumi.
- Manifest provider schema and secret-reference field when Bangumi token is
  enabled/configured.
- Provider registry support and diagnostics.
- Bangumi provider adapter with fake-transport tests.
- README/example updates after implementation.

Out of scope:

- Douban. Douban may require a crawler/browser automation runtime and should be
  designed separately.
- Playwright integration.
- Live Bangumi network tests in default gates.
- Episode-level scraping, people/cast enrichment, image downloading, and NFO
  writing.

## Architecture Direction

Bangumi should look like TMDB from the runtime's perspective:

1. Config resolves provider defaults and optional secrets.
2. Registry decides whether the provider is ready, disabled, or unavailable.
3. The provider calls official HTTP APIs through `ProviderHttpRuntime`.
4. The provider maps remote data into `ProviderMetadataCandidate` facts.
5. `MetadataScrapeRuntime` ranks and shapes responses.

Bangumi-specific detail belongs inside `providers::bangumi`. Shared retry,
timeout, response-size, JSON parsing, and redaction behavior stays in
`ProviderHttpRuntime`.

## Risks

- Bangumi search API is documented as experimental. Tests should lock our local
  mapping behavior, not rely on undocumented ordering.
- User-Agent policy matters. A generic default request-library UA is not
  acceptable.
- Some fields can be empty strings. Mapping must treat blank strings as absent.
- `infobox` can be structurally flexible. The first baseline should not parse it
  into hard contracts.
