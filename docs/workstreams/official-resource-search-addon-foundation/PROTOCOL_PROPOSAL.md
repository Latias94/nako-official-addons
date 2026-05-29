# Deferred Nako Protocol Proposal

This proposal records the host-side changes that should be made in `../nako`
after the official addon-side foundation is reviewed. Do not make these changes
inside the current plugin-side lane.

## Recommended Addon Protocol Additions

Add a first-class resource kind:

- `AddonResource::ResourceSearch`
- Wire name: `resource_search`
- Purpose: read-only external resource discovery.

Add a read/discovery scope:

- `AddonScope::AcquisitionSearchRead`
- Wire name: `acquisition_search_read`
- Purpose: allow an addon to search external sources and return candidates to
  the host. It does not grant download execution, acquisition candidate writes,
  metadata writes, stream URL reads, or catalog reads.

The existing alpha sidecar uses `AddonResource::Automation` only because the
current protocol has no better semantic fit. The migration path should be:

1. Add `resource_search` to Nako protocol and catalog validation.
2. Update `nako-resource-search` to declare `resource_search`.
3. Keep the current automation alpha route for one compatibility window only if
   installed manifests need a transition path.
4. Remove the automation alpha declaration after host support is stable.

## DTO Shape

The host contract should mirror the sidecar's alpha DTOs, with host-owned names:

- `AddonResourceSearchRequest`
  - `query: String`
  - `limit: Option<usize>`
  - `sources: Vec<String>`
  - `link_types: Vec<AddonResourceLinkType>`
  - `refresh: bool`
  - `ext: serde_json::Value`
- `AddonResourceSearchResponse`
  - `schema: String`
  - `query: String`
  - `total: usize`
  - `results: Vec<AddonResourceSearchResult>`
  - `merged_by_type: BTreeMap<AddonResourceLinkType, Vec<AddonMergedResourceLink>>`
  - `provider_executions: Vec<AddonResourceSearchProviderExecution>`
- `AddonResourceSearchResult`
  - stable result id, title, source, optional content, links, tags, images,
    score.
- `AddonResourceLink`
  - raw URL, normalized URL, classified link type, source, optional password,
    optional note.

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

## Host Execution Boundary

Nako should call resource-search addons through the same bounded worker model as
other HTTP addons:

- Admin/API handlers enqueue or schedule work; they do not synchronously call
  arbitrary addon URLs from request handlers.
- The worker validates manifest resource declarations and scope grants.
- The worker validates resource envelopes and response envelopes.
- The worker records provider execution diagnostics without storing secrets.

## Acquisition Intake Handoff

Search results are suggestions, not trusted acquisitions.

Recommended flow:

1. Addon returns typed resource search results.
2. Host displays or otherwise evaluates candidates under Nako policy.
3. After an explicit host/user decision, Nako creates acquisition intake
   candidates using the existing host model.
4. Optional addon-side candidate submission can use
   `ADDON_RUNTIME_ACQUISITION_INTAKE_CANDIDATES_PATH` only if Nako later adds a
   separate write scope such as `AcquisitionCandidateWrite`.

The read scope must not imply candidate write permission.

## Link Checking And Download Hooks

Do not fold link checking or downloading into the base search resource.

Recommended follow-on contracts:

- `resource_link_check` task or resource for availability checks.
- Provider-specific timeout and proxy policy.
- Optional downloader hook task with explicit operator configuration.
- No default support for invoking local BitTorrent, ed2k, or cloud-drive
  download clients without a dedicated permission and audit trail.

## Security Rules

- Deny by default. Searching requires `acquisition_search_read`.
- No raw downloader command execution from search.
- No host-trusted stream URLs from resource search responses.
- Password fields are data returned to the operator, not secrets for addon
  authentication.
- Diagnostics may report provider ids, counts, and safe status codes, but not
  bearer tokens, cookies, resolved secret values, or local downloader endpoints.

## Catalog And Official Addon Updates

After the protocol lands, update `nako-official-addon-catalog` in `../nako`:

- Add `resource_search` official catalog constants.
- Add install descriptors for binary and container runtime modes.
- Add schema ids matching the host DTOs.
- Prefer `nako.official.resource-search` as the stable addon id.
