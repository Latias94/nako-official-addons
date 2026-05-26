# Official Metadata Addon AV Native Writeback And Provider Wave 2

Status: Closed
Last updated: 2026-05-26

## Why This Lane Exists

The metadata scraper now has structured MDCx-style AV facts, browser-worker
proxy/session/wait controls, and a first rendered AV fallback provider. The
remaining architecture gap is that the Nako addon metadata write protocol still
persists only a narrow scalar/tag patch. That makes actors, directors, studios,
series, external IDs, thumbnails, and extra artwork visible in scrape responses
but not first-class persisted library data.

This lane intentionally allows breaking protocol changes. The target is the
clean canonical model, not compatibility with the older minimal
`AddonMetadataPatch` surface.

## Relevant Authority

- Current addon work:
  - `docs/workstreams/official-metadata-addon-av-mdcx-parity/`
  - `crates/nako-metadata-scraper/src/engine/av.rs`
  - `crates/nako-metadata-scraper/src/engine/bulk.rs`
  - `crates/nako-metadata-scraper/src/providers/javbus.rs`
- Nako protocol and persistence:
  - `../nako/crates/nako-addon-protocol/src/lib.rs`
  - `../nako/crates/nako-server/src/app/addons/metadata_write.rs`
  - `../nako/crates/nako-core/src/media/item.rs`
  - `../nako/crates/nako-catalog/src/lib.rs`
- Reference-only upstream:
  - `repo-ref/mdcx/mdcx/config/models.py`
  - `repo-ref/mdcx/mdcx/models/types.py`
  - `repo-ref/mdcx/mdcx/crawlers/__init__.py`
  - `repo-ref/mdcx/mdcx/crawlers/javlibrary.py`
  - `repo-ref/mdcx/mdcx/crawlers/javbus.py`

## Problem

AV provider quality is constrained by the narrow writeback contract. Adding more
providers without native persistence would duplicate richer data into response
only structures and leave batch writeback incomplete. The bulk task also remains
a bounded sequential batch wrapper with duplicate reuse, not a mature scrape
workflow with provider failure suppression and operator-visible retry classes.

## Target State

When this lane closes:

- Nako addon metadata writeback accepts a canonical graph-shaped patch for
  ratings, images, credits, collections, studios, and external IDs.
- Nako metadata writeback uses full catalog graph projection for changed
  canonical metadata, so credits/studios/collections/images/search stay
  consistent.
- The official metadata scraper materializes selected AV facts into the native
  writeback patch, while keeping response evidence explainable.
- Bulk scrape output has mature provider accounting: retry classes, temporary
  provider suppression, cooldown hints, and stable resume state.
- Provider wave 2 adds high-value MDCx-inspired AV sources behind
  disabled-by-default config and synthetic rendered-HTML tests.
- Docs describe the broken protocol boundary and new operational defaults.

## In Scope

- Breaking `nako-addon-protocol` metadata write payload shape.
- Updating Nako server metadata write adapter and focused tests.
- Updating official addon candidate/writeback mapping to fill canonical credits,
  studios, collections, external IDs, and image refs from AV facts.
- Extending bulk scrape diagnostics and resumable policy state.
- Adding provider wave 2 slices after the shared writeback and bulk foundation.
- Updating README, workstream docs, manifests/config docs, and validation gates.

## Out Of Scope

- Maintaining old minimal metadata writeback payload compatibility.
- Copying MDCx source, comments, selector tables, regex tables, or fixtures.
- Live scraping adult websites in CI.
- NFO writer, actor photo download, watermarking, or file renaming.
- UI redesign for reviewing AV metadata.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Nako `CanonicalMetadata` already has the graph fields needed for this phase. | High | `../nako/crates/nako-core/src/media/item.rs` includes ratings, images, credits, collections, studios, and external IDs. | Add core fields before protocol work. |
| Full catalog projection after metadata write is cleaner than field-specific graph selection. | High | `nako-catalog::plan_item_catalog_projection` already hydrates all graph types from canonical metadata. | Add a complete graph selection type, but prefer avoiding partial graph complexity. |
| Breaking the protocol is acceptable for this branch. | High | User explicitly requested no backwards compatibility and clean architecture. | If release compatibility becomes necessary, split a migration branch later. |
| Provider wave 2 should reuse browser-worker rendered-page mechanics rather than add per-provider browser stacks. | High | Browser-worker now owns proxy/session/wait controls. | Add provider-specific options only for routing and parsing. |

## Architecture Direction

Nako should treat addon metadata writeback as an external canonical metadata
proposal. The protocol crate remains independent from `nako-core`, but its
metadata write payload should mirror canonical metadata intentionally and map
one-to-one in the server adapter.

The server adapter should avoid partial catalog graph update logic. It should
merge the incoming canonical patch with local locks, then run full catalog
projection from the updated `MediaItem`. This is simpler, easier to reason
about, and matches the no-compatibility constraint.

The addon should keep `AvMetadataFacts` as provider evidence, but selected AV
facts should also be materialized into the `AddonMetadataPatch`: actors and
directors become credits, studio/maker/publisher/label become studios/tags,
series becomes collections, `thumb_url` and `extrafanart_urls` become images,
and provider identifiers become external IDs.

Bulk maturity should stay inside the existing `bulk-metadata-scrape` task. Nako
owns task scheduling; the sidecar owns scrape planning, provider diagnostics,
and resume-safe suppression hints.

## Closeout Condition

This lane can close when:

- Nako and addon protocol/writeback tests pass,
- official addon tests pass for AV materialization, bulk maturity, and provider
  wave 2,
- docs describe the new broken contract,
- each affected repo has only intended changes staged/committed,
- and remaining AV parity work is explicit follow-up scope.

## Closeout Summary

Closed on 2026-05-26.

This lane shipped the breaking native metadata writeback contract in Nako,
materialized selected AV facts into canonical graph writeback fields in the
official metadata scraper, added explicit provider retry/suppression accounting
for bulk scrape, and added disabled-by-default JavLibrary and MGStage rendered
providers. The browser-worker remains the single browser/proxy/session/wait
boundary; provider parsers are independently implemented from MDCx references.

Remaining AV work is follow-up scope, mainly more provider waves, review UI,
NFO/rename, and actor-image workflows.
