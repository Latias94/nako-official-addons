# Official Metadata Addon Provider Hardening

Status: Complete
Last updated: 2026-05-23

## Why This Lane Exists

The addon already has a shared provider HTTP runtime seam, but network policy,
reachability diagnostics, and provider candidate quality are still not deep
enough for region-blocked or flaky access paths. Operators need TMDB and
Bangumi scraping to stay reachable, explainable, and testable without pushing
browser automation or host-side task orchestration into this addon.

## Relevant Authority

- ADRs:
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- Existing docs:
  - `README.md`
  - `crates/nako-metadata-scraper/README.md`
  - `docs/workstreams/official-metadata-addon-fearless-refactor/DESIGN.md`
  - `docs/workstreams/official-metadata-addon-side-effect-writer/DESIGN.md`
  - `docs/workstreams/official-metadata-browser-worker/HANDOFF.md`
- Reference repositories:
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/CheckTMDB/README.md`
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/tinyMediaManager/README.md`
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/mdcx/README.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-fearless-refactor`
  - `docs/workstreams/official-metadata-addon-side-effect-writer`
  - `docs/workstreams/official-metadata-bangumi-provider-baseline`
  - `docs/workstreams/official-metadata-browser-worker`

## Problem

- `provider_http_runtime` already owns timeout, retry, user-agent, and proxy
  mechanics, but the user-facing configuration and diagnostics are not yet a
  first-class network policy.
- TMDB and Bangumi candidates can still be improved through provider-local
  normalization, ranking, and image selection.
- Region-blocked or DNS-polluted access paths are operationally real, but the
  addon should not copy hosts-file or IP-refresh workarounds into its core.
- Current workstream boundaries already defer browser automation and bulk task
  orchestration elsewhere, so this lane should stay focused on provider-side
  hardening.

## Target State

When this lane closes:

- Provider network policy is surfaced through configuration, docs, and tests.
- Diagnostics can distinguish endpoint, DNS, and proxy failure modes without
  leaking secrets.
- TMDB and Bangumi candidate shaping is deeper and more reliable.
- The addon remains modular: routes stay thin, transport policy stays in the
  HTTP runtime, and provider semantics stay inside provider modules.
- Browser automation, host-side task execution, and bulk scrape orchestration
  remain in their own lanes.

## In Scope

- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/providers/http_runtime.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/routes.rs`
- `crates/nako-metadata-scraper/README.md`
- `addons/metadata-scraper/README.md`
- focused tests and smoke coverage for the above seams

## Out Of Scope

- `addons/browser-worker`
- Douban crawler/browser automation implementation
- Addon Task runtime on the Nako host
- Bulk scrape orchestration inside the addon
- Copying code, schemas, fixtures, generated files, or text from reference
  repositories

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| `provider_http_runtime` is the correct seam for outbound policy. | High | It already owns timeout, retry, user-agent, and proxy behavior. | Network policy would need a new seam. |
| Proxy support belongs in addon configuration, not route handlers. | High | Current config exposes provider options but not a shared network policy. | Routes would keep leaking transport concerns. |
| CheckTMDB is useful as an operator-workaround reference, not as code to copy. | High | Its README is a hosts-file/DNS workaround, which is operationally brittle for addon core. | The lane would need to revisit the boundary. |
| Browser automation stays in a separate lane. | High | Existing workstreams and docs already isolate the browser worker. | This lane would need to absorb unrelated scope. |

## Architecture Direction

Keep transport policy in `provider_http_runtime`.
Keep provider semantics in the provider modules.
Keep diagnostics as a separate seam from scraping.
Prefer one shared network-policy model with provider-specific overrides rather
than duplicating request construction inside TMDB or Bangumi.

## Closeout Condition

This lane can close when:

- network policy is surfaced and documented,
- provider diagnostics are specific enough to be actionable,
- TMDB and Bangumi candidate quality has visibly deepened,
- evidence gates pass,
- and follow-on work is either split or explicitly deferred.

This lane is now closed. Remaining breadth work belongs in follow-on lanes.
