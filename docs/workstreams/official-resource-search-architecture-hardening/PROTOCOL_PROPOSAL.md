# Deferred Nako Resource Search Protocol Proposal

Status: addon-side architecture hardened on 2026-05-28. Nako core changes are
deferred to a separate `../nako` host lane.

## Current Alpha State

`nako-resource-search` still declares an `automation` resource at
`/resource-search` because the current Addon Protocol has no first-class
resource search kind.

The addon-side model now has stable boundaries:

- `ResourceSearchIntent` is internal and normalizes free-text, media-title,
  external-id, and exact-link searches.
- Providers declare descriptors, capabilities, source policy, and manifest
  schema fragments.
- Provider registry owns enablement and redaction-safe diagnostics.
- Fusion owns filtering, ranking, deduplication, grouping, and provenance.
- Providers return `ProviderSearchBatch` with safe warnings and finality hints.
- The official addon owns generic adapters and taxonomy, not live site scrapers
  or downloader clients.

## Required Nako Protocol Additions

Add a first-class read-only addon resource:

- Rust enum: `AddonResource::ResourceSearch`
- Wire name: `resource_search`
- Purpose: bounded external resource discovery for acquisition candidates.

Add a read scope:

- Rust enum: `AddonScope::AcquisitionSearchRead`
- Wire name: `acquisition_search_read`
- Purpose: allow an addon to search external sources and return resource
  candidates.

This scope must not imply:

- download execution;
- cloud-drive save actions;
- acquisition candidate writes;
- metadata writes;
- stream URL reads;
- catalog reads.

Any future addon-initiated candidate submission should use a separate write
scope such as `AddonScope::AcquisitionCandidateWrite`.

## Host DTOs

The stable host contract should make search intent explicit instead of relying
on opaque `ext` conventions.

Recommended request:

- `AddonResourceSearchRequest`
  - `schema: String`
  - `intent: AddonResourceSearchIntent`
  - `query: String`
  - `limit: Option<usize>`
  - `sources: Vec<String>`
  - `link_types: Vec<AddonResourceLinkType>`
  - `refresh: bool`
  - `context: serde_json::Value`

Recommended intent enum:

- `FreeText { text: String }`
- `MediaTitle { title: String, year: Option<i32>, media_kind: Option<String> }`
- `ExternalId { kind: String, value: String }`
- `ExactLink { url: String }`

Recommended response:

- `AddonResourceSearchResponse`
  - `schema: String`
  - `query: String`
  - `intent: AddonResourceSearchIntent`
  - `total: usize`
  - `results: Vec<AddonResourceSearchResult>`
  - `merged_by_type: BTreeMap<AddonResourceLinkType, Vec<AddonMergedResourceLink>>`
  - `provider_executions: Vec<AddonResourceSearchProviderExecution>`

Recommended provider execution DTO:

- `provider_id: String`
- `status: AddonProviderExecutionStatus`
- `result_count: usize`
- `finality: AddonProviderSearchFinality`
- `safe_message: Option<String>`

Recommended status values:

- `ok`
- `error`
- `skipped`

Recommended finality values:

- `complete`
- `partial`
- `unknown`

The alpha sidecar can map host intent into its current `query` plus `ext`
payload during the transition window.

## Result And Link DTOs

Recommended result:

- `id: String`
- `title: String`
- `source: String`
- `content: Option<String>`
- `links: Vec<AddonResourceLink>`
- `tags: Vec<String>`
- `images: Vec<String>`
- `score: u16`

Recommended link:

- `url: String`
- `normalized_url: String`
- `link_type: AddonResourceLinkType`
- `source: String`
- `password: Option<String>`
- `note: Option<String>`

Recommended merged link:

- `url: String`
- `normalized_url: String`
- `link_type: AddonResourceLinkType`
- `password: Option<String>`
- `note: Option<String>`
- `sources: Vec<String>`

Initial link type taxonomy:

- `aliyun`
- `baidu`
- `quark`
- `tianyi`
- `uc`
- `mobile`
- `115`
- `pikpak`
- `xunlei`
- `123`
- `magnet`
- `ed2k`
- `web`
- `other`

## Provider Metadata

Nako should preserve provider ids in request source filters and response
diagnostics. Provider descriptors should remain addon-owned, but Nako should be
able to display redaction-safe facts from health or diagnostics payloads:

- provider id;
- display name;
- source policy: `official`, `external_service`, `third_party`;
- default enablement;
- active/configured state;
- capability names;
- safe status code.

Nako should not require official addons to bundle site-specific scraper code.
Generic external-service adapters, such as PanSou-compatible HTTP, remain valid
when explicitly configured by the operator.

## Host Execution Boundary

Nako should call resource-search addons through the bounded addon worker model:

- API handlers enqueue or schedule work instead of calling arbitrary addon URLs
  directly.
- The worker validates manifest resource declarations and scope grants.
- The worker validates request and response envelopes.
- Timeouts, retry count, and maximum result limits are host-owned.
- Provider execution diagnostics are stored only as redaction-safe facts.

Default local smoke must remain no-network. A fixture-only official sidecar is
healthy; live external services activate only through explicit configuration.

## Acquisition Handoff

Resource search results are candidates, not acquisitions.

Recommended flow:

1. Nako calls a read-only `resource_search` addon.
2. Nako displays or evaluates candidates under host policy.
3. A user or host rule explicitly selects a candidate.
4. Nako converts selected links into acquisition intake candidates using host
   policy and host-owned audit trails.

The search response should never directly trigger downloader execution.

## Link Checking And Downloader Hooks

Do not fold link checking or downloading into base resource search.

Recommended follow-on contracts:

- `resource_link_check` task or resource for availability, password-needed, and
  unsupported-link checks.
- Explicit downloader hook task for qBittorrent, Transmission, aria2, ed2k, or
  cloud-drive save actions.
- Separate scopes and audit events for every write or external action.

## Migration Plan

1. Add `resource_search` and `acquisition_search_read` to `../nako`
   protocol enums, serde wire names, catalog validation, and SDK DTOs.
2. Add host worker support for bounded resource-search addon calls.
3. Add tests for request/response validation, scope denial, redaction-safe
   diagnostics, and timeout behavior.
4. Update `nako-resource-search` manifest to declare `resource_search` instead
   of temporary `automation`.
5. Keep the automation alpha route only for one compatibility window if needed.
6. Remove the automation alpha declaration after host support is stable.

## Out Of Scope

- Site-specific scraper implementations in the official addon.
- Downloader command execution from search.
- Cloud-drive save API calls from search.
- Treating returned passwords as addon authentication secrets.
- Host-trusted playback or stream URLs from search responses.
