# Official Addon Contract Smoke Provider Hardening - Design

Status: Complete
Last updated: 2026-05-24

## Problem

The previous cross-repo refactor closed three immediate risks, but left three
follow-on seams that should be hardened before more official addon behavior is
added:

1. The metadata scraper still mirrors the Addon Task envelope locally even
   though `../nako/crates/nako-addon-protocol` already exports
   `AddonTaskRequest` and `AddonTaskResponse`.
2. The official smoke scripts can exercise host-dispatched task execution, but
   the repeatable live harness for starting or targeting a local Nako server,
   sidecar, admin token, routing plans, task run, and result assertion is still
   not a reliable release gate.
3. Provider registry, provider diagnostics, manifest configuration schema, and
   provider construction still share too much knowledge in central modules.
   TMDB/Bangumi/Douban adapters are now modular, but provider descriptors are
   not yet provider-owned.

## Existing Decision

Do not reopen protocol release/versioning as a new problem. `../nako` already
has ADR 0033:

- `../nako/docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`

That ADR separates Addon Version, Addon Protocol Version, and Rust crate
package version. This lane should reference that decision, not duplicate it.

## Target State

- The metadata scraper imports Addon Task envelope types from the public Nako
  addon protocol crate instead of maintaining local mirrors.
- The official smoke flow has a repeatable live mode or documented harness that
  proves registration, grants, metadata resource calls, protected-write gates
  where configured, and host-dispatched `bulk-metadata-scrape` task execution.
- Provider capability/config/diagnostic descriptors move toward provider-owned
  declarations, while `ProviderRegistry` remains the composition point.

## Scope

- `crates/nako-metadata-scraper/src/engine/bulk.rs`
- `crates/nako-metadata-scraper/src/routes.rs`
- `crates/nako-metadata-scraper/src/providers/registry.rs`
- `crates/nako-metadata-scraper/src/providers/*`
- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/manifest.rs`
- `addons/metadata-scraper/smoke.local.ps1`
- `addons/metadata-scraper/compose.example.yml`
- `README.md`
- `addons/metadata-scraper/README.md`
- `../nako/scripts/official-addon-e2e-smoke.ps1`
- `../nako/crates/nako-addon-protocol` only if the already-exported task
  envelope needs small additive helper tests or derives

## Non-Goals

- Re-deciding Addon Protocol versioning or crate release policy already covered
  by ADR 0033.
- Splitting the official metadata addon into many user-installed sidecars.
- Adding a new official subtitle, NFO, artwork, AI, or diagnostics addon in
  this lane.
- Moving browser automation into the Rust sidecar.
- Making Nako supervise addon processes as product behavior.
- Copying implementation code from `F:/SourceCodes/Rust/repo-ref/nako-scraper`.

## Installation Experience Constraint

Official addon boundaries must respect the current sidecar installation model.
Implementation modules can be deep and provider-owned, but the operator should
not be forced to install a pile of tiny sidecars just because the codebase has
clean internal boundaries.

Future official addon families need a separate product decision: one sidecar
with multiple declared capabilities, a bundled suite of sidecars, or separate
installable addons. That decision should weigh operator friction, dependency
weight, crash isolation, permissions, update cadence, and Admin UI clarity.
This workstream must not pre-commit to that split.

## Architecture Direction

The task envelope work is a contract cleanup: `nako-addon-protocol` owns the
wire shape; the sidecar owns task planning and result payloads.

The smoke work should turn existing scripts into a release-quality gate without
implying hidden process supervision. It may provide local helper orchestration,
but product semantics stay explicit: sidecars are external HTTP services.

The provider descriptor work should make provider additions boring. Each
provider should be able to declare its id, capabilities, config requirements,
availability checks, and manifest schema contribution close to its adapter.
The central registry should compose descriptors instead of knowing provider
details.

## Closeout Result

The lane completed the selected 2/3/4 follow-ons:

- metadata scraper task endpoints now use public `nako-addon-protocol`
  `AddonTaskRequest` and `AddonTaskResponse` types;
- smoke scripts now fail early when Nako-owned paths are requested without
  `-RegisterInNako`, can assert writeback outcomes, validate the task
  declaration, and support E2E `-PreflightOnly`;
- provider modules now own their catalog entries, capabilities, construction
  hooks, and secret reference declarations, while `ProviderRegistry` composes
  those descriptors and manifest generation consumes the registry surface.

Full Docker/server live smoke was not executed in this session. It remains a
release-time proof, now backed by a stronger preflight and no silent skip path.
