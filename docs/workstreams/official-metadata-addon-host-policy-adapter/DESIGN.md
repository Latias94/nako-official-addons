# Official Metadata Addon Host Policy Adapter - Design

Status: Complete
Last updated: 2026-05-26

## Problem

Nako now owns metadata application policy through its host-side
`MetadataApplication` Module. The official metadata scraper must remain a
sidecar adapter: it can submit canonical metadata-shaped facts, but it must not
choose host merge mode, lock behavior, local/NFO authority, or catalog
projection.

## Finding

The existing sidecar boundary is already mostly correct:

- `/metadata` responses are suggestions unless the request explicitly contains
  `writeback`.
- `writeback` accepts only `library_id`, `target`, and `idempotency_key`.
- metadata writeback submits the selected candidate patch as facts.
- native AV enrichment only materializes provider facts into
  `AddonMetadataPatch`; it does not encode host authority.

This lane therefore pins the boundary with tests and documentation instead of
adding a new abstraction.

## Boundary

The sidecar may:

- normalize provider facts;
- rank/select candidates;
- construct canonical metadata patches;
- request a Nako side effect with provenance and idempotency.

The sidecar must not:

- accept `refresh_mode`, `application_mode`, merge mode, lock overrides, or NFO
  authority fields in `writeback`;
- decide whether an existing field should be overwritten;
- project catalog graph/search state.

## Non-Goals

- No host policy implementation in this repository.
- No changes to provider scraping breadth.
- No changes to Addon installation or distribution.
