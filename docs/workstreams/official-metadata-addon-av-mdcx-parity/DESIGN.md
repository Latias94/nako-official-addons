# Official Metadata Addon AV MDCx Parity

Status: Complete
Last updated: 2026-05-26

## Why This Lane Exists

The metadata scraper now has AV routing, JavDB, DMM, FC2, field-source policy,
and resumable batch accounting, but it still treats many MDCx-grade AV fields as
plain tags. The browser-worker path also lacks an explicit proxy/session/wait
contract, which makes several mature AV providers unreliable in real networks.

## Relevant Authority

- Existing docs:
  - `addons/metadata-scraper/README.md`
  - `addons/browser-worker/README.md`
  - `crates/nako-metadata-scraper/README.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-mdcx-style-av-scraping/`
  - `docs/workstreams/official-metadata-addon-av-provider-policy/`
  - `docs/workstreams/official-metadata-browser-worker/`
- Reference-only upstream:
  - `repo-ref/mdcx/mdcx/config/models.py`
  - `repo-ref/mdcx/mdcx/models/types.py`
  - `repo-ref/mdcx/mdcx/crawlers/__init__.py`
  - `repo-ref/mdcx/mdcx/crawlers/javbus.py`
  - `repo-ref/mdcx/mdcx/crawlers/javlibrary.py`

## Problem

AV providers can scrape useful fields such as actors, director, studio, series,
publisher, wanted count, thumb, trailer, and extra fanart, but the addon cannot
represent them as structured provider facts or select them with field policy.
The rendered-page worker also cannot route traffic through a configured proxy or
express basic wait/session intent.

## Target State

When this lane closes:

- AV candidates expose structured MDCx-inspired fields alongside the existing
  portable `AddonMetadataPatch`.
- Provider field policy can select those AV fields across merged provider
  clusters.
- Browser-worker `/extract` and `/render` accept a redaction-safe proxy/session
  and wait contract, with env-based proxy configuration.
- At least one high-value MDCx-inspired fallback provider is added
  disabled-by-default with synthetic rendered-HTML tests.
- Docs make the current protocol boundary explicit: structured AV facts are
  returned in scrape responses now; Nako-side persistence of credits/studios and
  collections is a separate protocol/server follow-up.

## In Scope

- Add an internal structured AV fact model to `nako-metadata-scraper`.
- Map existing JavDB, DMM, and FC2 facts into that model.
- Include AV fields in candidate response JSON and source evidence.
- Add browser-worker proxy/session/wait request parsing and health diagnostics.
- Add one provider-expansion slice after the shared field/proxy foundation.
- Update manifests, README docs, workstream evidence, and focused tests.

## Out Of Scope

- Copying MDCx source, comments, selector tables, or regex tables.
- Persisting all AV fields into Nako canonical metadata in this lane.
- Full MDCx provider parity in one batch.
- Live scraping against adult sites during CI validation.
- NFO writer, actor photo download, watermarks, or file renaming.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The addon can safely expose provider-specific structured AV facts without changing `AddonMetadataPatch`. | High | `metadata_response` serializes addon-owned candidate payloads and artifacts. | A Nako protocol change must be split before provider work can continue. |
| Existing Nako canonical metadata already has credits, collections, studios, images, and external IDs, but addon writeback does not expose them yet. | High | `../nako/crates/nako-core/src/media/item.rs`; `AddonMetadataPatch` is narrower. | This lane must stay response-side only until the server write path is extended. |
| Browser-worker proxy support belongs in the Node worker, not in each Rust AV provider. | High | Rust providers call `/render`; Playwright owns target-site traffic. | Provider configs would duplicate browser concerns and still fail for page traffic. |
| JavBus/JavLibrary-style providers are valuable first fallback candidates after DMM/JavDB/FC2. | Medium | MDCx default route groups and crawler registry prioritize broad AV sources. | Reorder provider expansion based on real library route needs. |

## Architecture Direction

The scraper should keep `AddonMetadataPatch` as the portable writeback surface
and add a separate structured AV fact surface for fields that the current Nako
write protocol cannot persist yet. This mirrors the current separation between
patches, artwork candidates, evidence, and provider execution diagnostics.

Browser proxy/session/wait behavior should be centralized in
`addons/browser-worker`. Rust provider configs should keep using
`RenderedPageRuntime`; the render request can carry small, redaction-safe
options while the worker owns Crawlee and Playwright mechanics.

New AV providers should be disabled by default, route-aware, fixture-tested, and
added only after shared field/proxy infrastructure exists. MDCx remains
reference-only under GPL guardrails: strategy and behavior can be studied, but
source and selectors must be independently written.

## Closeout Condition

This lane can close when:

- the target state is implemented,
- evidence gates pass,
- docs reflect shipped behavior and remaining protocol limits,
- commits contain only intended changes,
- and any remaining MDCx parity work is split into explicit follow-ups.

## Closeout Notes

Closed on 2026-05-26. The lane shipped response-side structured AV facts,
provider-policy selection for those facts, browser-worker proxy/session/wait
controls, and the first disabled-by-default rendered AV fallback provider
(`javbus`). Nako canonical writeback expansion and broader provider parity are
tracked as follow-up candidates rather than hidden work in this lane.
