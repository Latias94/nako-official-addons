# Official Metadata Addon Field Policy Locality Deepening

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

The execution-locality lane removed Bulk execution, rendered-page support, and render drift
provider facts from central routing code. One adjacent residual remains: the default AV field
provider preference policy still lives in `ProviderRegistry` as central provider order arrays.

That shape is shallow because adding or rebalancing a provider requires editing a central table that
knows which provider should win title, outline, artwork, trailer, actor, score, and other AV fields.
The registry should compose provider-owned facts, not own per-provider policy decisions.

## Relevant Authority

- Previous lane:
  - `docs/workstreams/official-metadata-addon-execution-locality-deepening`
- Existing docs:
  - `../nako/CONTEXT.md`
  - `README.md`
  - `addons/metadata-scraper/README.md`
- ADRs:
  - `../nako/docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
  - `../nako/docs/adr/0042-external-casting-protocol-adapters.md`

## Problem

`ProviderRegistry` currently contains `DEFAULT_FIELD_PROVIDER_PREFERENCES` plus many
`DEFAULT_*_PROVIDER_ORDER` constants. Those constants encode provider-specific priority facts in a
central module. This increases edit fan-out and makes the registry less of a composition layer.

## Target State

- Provider modules declare their own default field preference descriptors.
- `ProviderRegistry` composes those descriptors into `ProviderFieldPolicy`.
- Domain field groups remain centralized only as field vocabulary helpers, not provider order lists.
- The existing default preset behavior remains test-equivalent.
- Obsolete central provider order arrays are deleted.

## In Scope

- `crates/nako-metadata-scraper/src/providers/registry.rs`
- Provider catalog entry modules that participate in default AV field policy
- `crates/nako-metadata-scraper/src/engine/query.rs` only if policy construction needs a more typed
  input shape
- Workstream docs under `docs/workstreams/official-metadata-addon-field-policy-locality-deepening`

## Out Of Scope

- Changing request-visible `provider_field_policy` payload behavior.
- Changing the quality-score field policy preset.
- Changing fusion conflict resolution.
- Rebalancing the provider preference order unless required by the new representation.
- Notification, Chromecast, or release publishing work.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Default provider order can be represented as provider-owned descriptor facts without behavior change. | High | Existing central table is static and deterministic. | Keep behavior tests as the contract and preserve exact order. |
| Field alias groups are domain vocabulary, not provider-specific facts. | High | The aliases map output field names to canonical metadata concepts. | Keep aliases centralized but avoid central provider order arrays. |
| Targeted registry/runtime tests are sufficient during iteration. | High | Policy construction is inside one crate and already covered by registry/runtime tests. | Broaden to package gate before closeout. |

## Architecture Direction

Use a provider-owned descriptor model:

- `ProviderDefaultFieldPreference` describes a field group plus an order owned by one provider.
- `ProviderCatalogEntry` exposes a static slice of those descriptors.
- `ProviderRegistry` folds catalog entries into field -> provider order maps.
- `ProviderFieldPolicy` remains the runtime contract consumed by fusion and request parsing.

This keeps provider capability and preference facts near the provider module, while retaining one
registry composition point.

## Closeout Condition

This lane can close when:

- central `DEFAULT_*_PROVIDER_ORDER` and `DEFAULT_FIELD_PROVIDER_PREFERENCES` arrays are removed,
- provider modules own default field preference descriptors,
- tests prove the default preset preserves existing provider order,
- package gates pass,
- and workstream docs record the shipped behavior.

Closeout status: complete on 2026-05-27. Default AV field provider order facts are now declared by
provider catalog descriptors and folded by `ProviderRegistry`. Request-visible field policy parsing,
fusion behavior, and the quality-score preset were intentionally preserved.
