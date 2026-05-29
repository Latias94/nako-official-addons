# Official Addons Architecture Boundary Hardening - Handoff

Status: Complete
Last updated: 2026-05-29

## Current State

The lane is closed. OAABH-010 froze the scope after local architecture review.
OAABH-020 is complete: resource-search, subtitle-provider, and DLNA renderer
runtime manifests now use `nako-official-addon-catalog` builders for shared
official facts instead of duplicating manifest construction locally. OAABH-030
is complete: the large Nako addon app service has been split into local modules
for catalog/install guide, surfaces/readiness, routing, diagnostics,
resource-search, and subtitles. OAABH-040 is complete: metadata provider HTTP
operations now carry explicit retry-after, safe-cache, and throttle intent,
with TMDB detail enrichment as the first wired provider call site. OAABH-050 is
complete: notification bridge provider fan-out and diagnostics HTML rendering
now live outside route handlers.
OAABH-060 is complete: the mature provider model research docs now distinguish
historical P0 recommendations from completed resolver, external ID capability,
provider field-policy, and HTTP operation-policy baseline architecture.

## Closed Task

- Task ID: OAABH-070
- Owner: planner
- Files: `docs/workstreams/official-addons-architecture-boundary-hardening`
- Validation: final focused gates and workstream evidence review
- Status: DONE

## Decisions Since Last Update

- Keep External Acquisition Runner out of this lane; open it separately after
  Admin acquisition intake stabilizes.
- Use catalog builders where they already have enough parameters.
- If a catalog builder cannot express runtime sidecar configuration, extend the
  builder instead of copying manifest construction back into the sidecar.
- Keep resource-search provider schema fragments sidecar-owned because their
  defaults come from runtime provider configuration.
- Do not touch the dirty `../nako/web` files or screenshot artifacts while
  working on the server boundary task.
- Keep provider operation policy provider-local and explicit. Retry-after and
  cacheability must be operation facts, not hidden scheduler state.
- Preserve the new provider policy shape: policy describes provider-owned
  operation facts; it does not introduce an implicit cache or global scheduler.
- Preserve notification bridge route payloads. `provider_send` owns fan-out and
  attempt-history recording; `diagnostics` owns HTML page rendering.
- Mature provider research docs should now be treated as historical research
  plus status notes. Do not re-open resolver/external-ID work from that doc.

## Blockers

- None known.

## Next Recommended Action

- Commit or review this lane when ready. Do not include unrelated `../nako/web`
  and `../nako/docs/workstreams/web-playlist-management-ui-mutations/TODO.md`
  changes unless the user explicitly asks to include them.
