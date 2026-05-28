# Official Resource Search First-Class Protocol

Status: Active
Last updated: 2026-05-28

## Why This Lane Exists

The official `nako-resource-search` sidecar was intentionally shipped on a temporary
`automation` resource while Nako host support for resource search did not exist.
Nako now has first-class `resource_search` and `acquisition_search_read`
contracts, so the addon-side compatibility layer has become accidental
complexity.

## Relevant Authority

- Related host work:
  - `../nako/docs/workstreams/addon-resource-search-product-flow`
- Related completed addon workstreams:
  - `docs/workstreams/official-resource-search-addon-foundation`
  - `docs/workstreams/official-resource-search-architecture-hardening`
  - `docs/workstreams/official-resource-search-pansou-compatible-provider`
- Prior proposal:
  - `docs/workstreams/official-resource-search-architecture-hardening/PROTOCOL_PROPOSAL.md`

## Problem

The addon still declares an `automation` resource, uses addon-local alpha
request/response schemas, and exposes diagnostics that describe
`resource_search` as future work. This keeps the host and official addon on
different public contracts and forces tests and docs to preserve obsolete
protocol vocabulary.

## Target State

- The manifest declares `AddonResource::ResourceSearch` at `/resource-search`.
- The resource requires `AddonScope::AcquisitionSearchRead`.
- Request and response payloads use `nako-addon-protocol` first-class
  `AddonResourceSearchRequest` and `AddonResourceSearchResponse` schemas.
- Route validation rejects non-`resource_search` envelopes.
- Internal provider/query/fusion boundaries stay addon-owned.
- Checked-in and runtime container manifests remain byte-for-byte equivalent
  after serialization.
- Docs and smoke tests describe the shipped first-class protocol, not alpha
  automation compatibility.

## In Scope

- `crates/nako-resource-search` manifest, route protocol, tests, and docs.
- `addons/resource-search/manifest.example.json` and `smoke.local.ps1`.
- Workstream and follow-on design notes for link checking, downloader/external
  runner hooks, cloud-drive transfer, and password/code secret references.

## Out Of Scope

- Admin UI.
- Downloader execution.
- Cloud-drive save/transfer execution.
- Link availability checking implementation.
- Site-specific scrapers beyond existing generic providers.
- Any search-triggered acquisition write. Host selection remains explicit.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Nako host protocol names are stable for this lane. | High | `../nako/crates/nako-addon-protocol/src/lib.rs` defines `resource_search`, `acquisition_search_read`, and v1 schemas. | The addon manifest/tests would need a follow-up protocol rename. |
| The official addon can drop the temporary automation declaration immediately. | Medium | User requested fearless refactor and Nako host support exists. | A released host pinned to the old addon protocol would need a compatibility branch, not this mainline lane. |
| Internal provider domain should not be replaced wholesale by protocol DTOs. | High | Providers and config still need addon-owned source policy, parsing, fusion, and PanSou mapping. | Over-merging the domain into protocol DTOs would couple external wire format to provider internals. |

## Architecture Direction

The protocol boundary moves to `routes::resource_protocol`. That module is the
adapter from host-owned first-class DTOs into the addon-owned search domain and
back out again. The rest of the engine continues to operate on internal domain
types so provider code does not depend on host transport concerns.

This gives one explicit dependency direction:

`host protocol DTO -> route adapter -> addon domain -> providers/fusion -> route adapter -> host protocol DTO`

The manifest becomes a direct declaration of that contract. No resource-search
code should mention `automation_run` or alpha schemas after this lane closes.

## Follow-On Boundary Notes

Search remains read-only and only returns candidate information. Follow-on
contracts must stay separate:

- Link checking: a read-only availability/password-needed probe with its own
  scope and timeout policy.
- Downloader/external runner: an explicit audited command/action contract, not
  part of search.
- Cloud-drive transfer: a write/action contract owned by acquisition policy.
- Password/code references: host-owned secret/reference handling for selected
  candidates, not provider authentication secrets.

## Closeout Condition

This lane can close when:

- first-class protocol migration is implemented,
- stale alpha automation vocabulary is removed from code, docs, and smoke tests,
- targeted and package gates pass,
- follow-on boundaries are documented,
- and the commit contains only this lane's changes.
