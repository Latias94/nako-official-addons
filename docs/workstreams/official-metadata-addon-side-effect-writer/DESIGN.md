# Official Metadata Addon Side Effect Writer - Design

Status: Active
Last updated: 2026-05-23

## Problem

The official metadata Addon can return provider suggestions, but it does not
yet prove the stronger Nako Addon model where an Addon Sidecar uses an Addon
Token to submit protected Addon Side Effects through Nako-owned runtime routes.

Nako core already has Addon Token, Library-Scoped Addon Grant,
`/addon/v1/side-effects`, `metadata_write`, and `artwork_write` apply paths.
The official Addon should exercise those seams without turning ordinary
metadata suggestion calls into implicit media-library mutation.

## Target State

- The Addon has a small outbound Nako runtime client for `/addon/v1/access-check`
  and `/addon/v1/side-effects`.
- Side effects are disabled by default and require explicit runtime
  configuration plus an explicit request payload.
- Metadata suggestions can be converted into a bounded `metadata_write`
  side-effect payload for an operator-provided target.
- Provider image facts are modeled as typed artwork candidates and can be
  converted into bounded `artwork_write` side-effect payloads.
- Bulk Metadata Scrape remains a design/task follow-on until Nako owns the
  Addon Task execution seam; the Addon must not run hidden background work.

## Scope

- Add side-effect runtime configuration and redaction-safe diagnostics.
- Add a testable outbound Nako runtime client with fake transport tests.
- Extend metadata runtime request handling with explicit side-effect submission
  options.
- Add typed artwork candidate facts behind provider adapters and response
  payloads.
- Update smoke/docs to describe manual token/grant setup and explicit writes.
- Evaluate the Bulk Metadata Scrape / Addon Task path and split or defer it.

## Non-Goals

- Automatic writes during ordinary `/metadata` calls.
- Depending on private `../nako` server/core crates from this public Addon
  workspace.
- Addon Manager lifecycle automation.
- Nako-hosted Addon Task execution before the host seam is implemented.
- Direct filesystem, database, or storage access from the Addon.
- Douban/crawler provider implementation.

## Architecture Direction

Keep the Addon install artifact as one sidecar. Add a deep `NakoRuntimeClient`
module whose interface hides bearer token placement, endpoint shape, response
redaction, timeout, and safe failure mapping.

`MetadataScrapeRuntime` remains the place where request normalization, provider
fan-out, ranking, response shaping, and optional side-effect submission are
coordinated. Providers expose facts, not final write behavior.

The side-effect request contract is JSON-shaped to avoid coupling this Addon to
private Nako crates. Public protocol structs from `nako-addon-protocol` may be
used when they are already published, such as `AddonMetadataPatch` and
`AddonArtworkWritePayload`.

## Related Evidence

- `../nako/docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- `../nako/docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
- `../nako/docs/workstreams/addon-protected-writes/`
- `../nako/docs/workstreams/addon-managed-artwork-artifacts/`
- `../nako/docs/workstreams/managed-artwork-ingest-selection/`
- `../nako/docs/workstreams/official-addon-e2e-alpha2/`
