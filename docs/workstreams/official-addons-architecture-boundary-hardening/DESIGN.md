# Official Addons Architecture Boundary Hardening - Design

Status: Active
Last updated: 2026-05-29

## Problem

Recent local review found that the official addon ecosystem has moved past the
first proof-of-contract phase. The remaining risks are not missing isolated
features; they are boundary drift between `nako-official-addons` and `../nako`,
large application modules that mix independent addon workflows, and provider
operational policy that is still too global for the number of live/rendered
providers now supported.

Concrete findings:

1. `nako-official-addon-catalog` already defines official manifests for
   resource search, subtitle, and DLNA, but those sidecars still hand-roll their
   manifest constants and builders locally. This duplicates official addon
   facts across the catalog crate, runtime manifests, checked-in examples, and
   server install-guide snippets.
2. `../nako/crates/nako-server/src/app/addons.rs` now owns registration,
   grants, token issuance, health/readiness, resource-search sessions, subtitle
   sessions/import, install-guide generation, and official catalog snippets in
   one service module. The file already has submodules for side effects and
   runtime internals, but user-facing workflow boundaries still sit in the
   parent file.
3. `ProviderHttpRuntime` centralizes timeout, retry, backoff, proxy, and body
   size limits, but it does not yet model provider operation policy such as
   `Retry-After`, short-lived safe caching, or throttle buckets.
4. `nako-notification-bridge` keeps routes, provider fan-out orchestration,
   diagnostics HTML, and a large route test module together. This is lower risk
   than the cross-repo issues, but it is now a clear locality cleanup.
5. The mature provider model research docs still describe resolver and external
   ID capability work as future P0 even though those seams now exist in code.

## Target State

- Official addon manifests and install descriptors use the catalog crate as the
  single source of truth where the catalog already has enough parameters.
- Sidecars own only runtime configuration fragments, provider discovery, and
  route behavior; shared official manifest facts stay in
  `nako-official-addon-catalog`.
- Nako server addon workflows are split by product boundary instead of growing
  inside one parent application service file.
- Provider HTTP operation policy becomes explicit and provider-local without
  introducing a persistent cache or a hidden refresh scheduler.
- Notification bridge routing keeps HTTP entry points thin, with diagnostics
  rendering and provider-send orchestration moved behind local modules.
- Stale research/workstream docs are updated so future agents do not redo
  already-completed resolver/capability work.

## Scope

Primary repository:

- `crates/nako-resource-search/src/manifest.rs`
- `crates/nako-subtitle-provider/src/manifest.rs`
- `crates/nako-dlna-renderer/src/manifest.rs`
- `crates/nako-metadata-scraper/src/providers/http_runtime.rs`
- metadata provider client/config call sites touched by HTTP operation policy
- `crates/nako-notification-bridge/src/routes.rs`
- new notification bridge route support modules as needed
- stale docs under `docs/workstreams/official-metadata-addon-mature-provider-model-research/`

Cross-repo scope in `../nako`:

- `crates/nako-official-addon-catalog/src/lib.rs` if builder parameters need to
  be expanded
- `crates/nako-server/src/app/addons.rs`
- new addon app service submodules under `crates/nako-server/src/app/addons/`
- focused addon service tests if module boundaries require test updates

## Non-Goals

- Do not implement External Acquisition Runner in this lane. That is a product
  feature with runner credentials, idempotency, cancellation, progress, and
  audit semantics; it should become a separate workstream after the active
  Admin acquisition intake lane stabilizes.
- Do not move addon process supervision into Nako.
- Do not add a full Jellyfin-style host metadata provider manager to the
  metadata sidecar.
- Do not change public Addon Protocol wire shapes unless a bounded task proves
  it is necessary and records the compatibility impact.
- Do not edit active `../nako/web` workstream files.

## Architecture Direction

Manifest ownership should be one-directional: the public catalog crate owns
official addon facts that Nako core can resolve, while each sidecar adapts those
facts to runtime configuration and serves the declared routes. If a sidecar
needs dynamic provider schema fragments, the catalog builder should accept that
dynamic input rather than forcing the sidecar to duplicate the whole manifest.

The Nako server addon service should deepen around product workflows:
registration/grants/tokens, runtime health/readiness, resource search, subtitle
import, and install catalog/guide generation. Keep repository traits and public
Admin API DTOs stable while moving implementation locality.

The metadata provider HTTP runtime should accept explicit operation policy from
provider clients. Start small: honor `Retry-After`, allow bounded in-memory
safe GET caching by provider operation key, and add throttle buckets only where
provider call sites opt in.

Notification bridge should keep route handlers as adapters. Provider fan-out
and diagnostics rendering should move into modules that can be tested without
Axum request plumbing.

