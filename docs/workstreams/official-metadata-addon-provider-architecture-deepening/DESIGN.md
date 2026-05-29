# Official Metadata Addon Provider Architecture Deepening

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

The metadata scraper has grown from a fixture proof into a multi-provider Addon
Sidecar with TMDB, Bangumi, Douban, browser-worker-backed rendered extraction,
bulk tasks, explicit side effects, and redaction-safe diagnostics.

The previous fearless-refactor and provider hardening lanes established the
right broad shape: one installable metadata Addon, provider adapters behind a
registry, shared HTTP runtime policy, runtime-owned candidate shaping, and
provider-owned parser/mapper/enrichment modules. The next risk is provider
scale. Adding another provider still requires too much central knowledge across
config, registry, manifest, query parsing, diagnostics, and duplicated
search-enrichment policy.

This lane deepens the provider/plugin architecture before more provider breadth
hardens the remaining shallow Modules.

## Relevant Authority

- ADRs:
  - `../nako/docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `../nako/docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `../nako/docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- Existing docs:
  - `addons/metadata-scraper/README.md`
  - `crates/nako-metadata-scraper/README.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-fearless-refactor/`
  - `docs/workstreams/official-addon-contract-smoke-provider-hardening/`
  - `docs/workstreams/official-addons-cross-repo-fearless-refactor/`
  - `docs/workstreams/official-metadata-addon-provider-relevance-budget/`
  - `docs/workstreams/official-metadata-addon-provider-partial-search-diagnostics/`
  - `docs/workstreams/official-metadata-browser-worker/`

## Problem

Provider extension remains too shallow: provider Modules own parser/mapper code,
but central Modules still know provider config shape, provider defaults,
external ID aliases, diagnostics, boot status, browser-worker support semantics,
and repeated search-enrichment policy.

## Target State

- Provider descriptor ownership is deep enough that a new provider can declare
  its config, defaults, secret references, capabilities, build/availability
  rules, and manifest contribution close to its adapter.
- Provider assembly happens once and produces ready adapters, disabled or
  unavailable diagnostics, and health-safe provider facts from one source of
  truth.
- TMDB and Bangumi share a deep search-enrichment policy Module for common
  provider flow: direct ID attempts, title variants, dedupe, budget selection,
  partial-search preservation, and degraded fallback.
- Provider outcome diagnostics are typed facts rendered through one
  redaction-safe Module instead of provider-local free-text strings.
- Browser-worker integration is a support Seam for rendered-page providers, not
  a confused mix of metadata provider identity and infrastructure adapter.
- Public payloads, manifest shape, default provider enablement, and current
  default-off real-provider behavior remain compatible unless explicitly
  documented.

## In Scope

- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/providers/mod.rs`
- `crates/nako-metadata-scraper/src/providers/registry.rs`
- `crates/nako-metadata-scraper/src/providers/browser_worker.rs`
- `crates/nako-metadata-scraper/src/providers/douban.rs`
- `crates/nako-metadata-scraper/src/providers/douban/*`
- `crates/nako-metadata-scraper/src/providers/tmdb/*`
- `crates/nako-metadata-scraper/src/providers/bangumi/*`
- `crates/nako-metadata-scraper/src/engine/query.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/routes.rs`
- metadata scraper docs and smoke examples when behavior or config changes

## Out Of Scope

- Adding a new metadata provider only to prove count.
- Splitting each provider into a separate user-installed Addon.
- Changing the Nako HTTP Addon model or weakening manifest/grant authority.
- Live provider smoke gates that require real TMDB/Bangumi credentials.
- Reworking unrelated notification-bridge provider architecture.
- Release publishing or Docker live smoke.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Provider config and descriptor facts should live closer to provider adapters. | High | `config.rs` still hard-codes every provider while provider modules own `catalog_entry()`. | New providers keep spreading changes across central Modules. |
| TMDB and Bangumi now justify a shared search-enrichment policy Seam. | High | Both adapters implement direct ID lookup, title variants, dedupe, ranking budget, partial failure preservation, and degraded fallback. | Keep policy provider-local and only extract smaller typed helpers. |
| Browser worker should primarily be a rendered-page support Seam. | Medium | Douban already uses the worker as infrastructure while `browser_worker` is also exposed as a metadata provider. | Keep standalone `browser_worker` provider but make the support dependency explicit. |
| Typed provider outcomes can preserve public payload compatibility. | Medium | `provider_note` is already a rendered diagnostic field in ranking evidence. | Defer public payload changes and only introduce internal outcome facts first. |
| Provider assembly can be made single-pass without changing runtime behavior. | High | `registry.diagnostics()` and `registry.providers()` currently compute related facts separately. | Split the assembly task or keep a compatibility adapter temporarily. |

## Architecture Direction

Deepen provider architecture by moving provider-specific knowledge behind
provider-owned Modules and by turning duplicated provider orchestration into
shared policy where two adapters already prove the Seam is real.

The central registry should compose provider descriptors; it should not know
each provider's config field layout, environment parsing, or support dependency
details. Routes should stay HTTP adapters. Metadata runtime should own scrape
flow, but provider-local adapters should not each reimplement the same
search-enrichment failure policy.

Browser-rendered extraction should be modeled as an internal support dependency
for provider adapters. If the standalone `browser_worker` provider remains, it
must have a clear metadata-provider role; otherwise it should be removed from
the provider catalog and retained as a rendered-page runtime.

Provider diagnostics should use typed provider outcomes and render
redaction-safe text at one Seam. Provider adapters should emit facts, not public
diagnostic prose.

## Closeout Condition

This lane can close when:

- all five refactor slices are implemented or explicitly split with a documented
  reason;
- targeted metadata scraper tests pass for provider registry/config, search
  pipeline, browser worker/Douban rendered-page behavior, diagnostics, and
  routes;
- workspace formatting and diff hygiene pass;
- docs reflect any changed provider strategy or config behavior;
- and follow-on work is either split or explicitly deferred.

Closeout status: Complete on 2026-05-25. All five refactor slices are
implemented, public payload/config compatibility is preserved, package and
format gates pass, and no architecture follow-on was split.
