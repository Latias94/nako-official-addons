# Official Metadata Addon Provider Extension Decentralization

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

The previous provider architecture lane made provider descriptors, assembly,
search policy, rendered-page support, and typed outcomes deeper. The next
provider will likely arrive soon, so the remaining central extension costs are
worth paying down before the shallow seams harden again.

## Relevant Authority

- ADRs:
  - `../nako/docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `../nako/docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `../nako/docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
- Existing docs:
  - `addons/metadata-scraper/README.md`
  - `crates/nako-metadata-scraper/README.md`
- Preceding lane:
  - `docs/workstreams/official-metadata-addon-provider-architecture-deepening/`

## Problem

Provider extension is still not deep enough. A new provider still pushes
provider-specific knowledge through central config structs, top-level external
ID alias parsing, rendered-page support terminology, and stale tests that make
the current behaviour harder to trust.

## Target State

- Provider config has no invalid optional-field matrix where each provider row
  can carry every other provider's config shape.
- Provider-local config structs live near their provider adapters, while the
  central runtime only knows a small provider config interface.
- Top-level external ID aliases are declared by provider-owned descriptors or a
  provider extension seam instead of being hard-coded in the query parser.
- Browser-rendered support is modelled as shared support infrastructure used by
  Douban and the `browser_worker` metadata provider, with names that make the
  support dependency explicit.
- Known stale test names and docs are cleaned up while preserving public
  payloads, config environment variables, manifest shape, and default provider
  enablement.

## In Scope

- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/manifest.rs`
- `crates/nako-metadata-scraper/src/engine/query.rs`
- `crates/nako-metadata-scraper/src/engine/runtime.rs`
- `crates/nako-metadata-scraper/src/providers/registry.rs`
- `crates/nako-metadata-scraper/src/providers/*`
- metadata scraper docs only when behaviour or naming changes

## Out Of Scope

- Adding a real new provider in this lane.
- Changing the Nako HTTP Addon model.
- Changing public request/response payload shape.
- Renaming existing environment variables.
- Release publishing or live provider smoke gates.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Provider config's optional-field matrix is now the largest remaining extension cost. | High | `ProviderConfig` carries `tmdb`, `bangumi`, `browser_worker`, and `douban` optional fields for every provider row. | New providers still require central invalid-state boilerplate. |
| External ID aliases should be provider-owned before adding more providers. | High | `engine/query.rs` hard-codes `tmdb_id`, `imdb_id`, and `bangumi_id`. | New providers require query-parser edits instead of descriptor edits. |
| Douban is a browser-rendered provider, not a standalone HTTP API provider. | High | Douban uses browser worker rendered HTML for search and detail pages. | Support naming remains confusing and future rendered providers copy the same glue. |
| Public compatibility should be preserved. | High | Existing users depend on env vars, manifest defaults, and provider note payloads. | Any intentional break needs a separate ADR or migration lane. |

## Architecture Direction

Deepen provider extension by reducing central invalid states and moving
provider-specific parsing facts behind provider-owned descriptors.

The central runtime may keep compile-time knowledge that a provider exists, but
it should not carry every provider's detailed config shape in a single shallow
struct. Query parsing should receive provider-owned external ID descriptors
rather than knowing provider aliases directly. Rendered-page support should be a
deep support Module with reusable config and operation semantics; Douban should
remain explicit about depending on the browser worker as a rendered-page
support Adapter.

## Closeout Condition

This lane can close when:

- provider config no longer exposes the optional-field matrix as the main
  extension Interface;
- top-level external ID aliases are provider-owned or descriptor-driven;
- rendered-page support naming and config are shared by Douban and
  `browser_worker` without changing public env vars;
- stale tests/docs discovered during the refactor are cleaned up;
- targeted metadata scraper tests, package tests, formatting, JSON, and diff
  hygiene pass.

Closeout status: Complete on 2026-05-25. Provider config decentralization,
provider-owned external ID aliases, rendered-page support semantics, README
cleanup, package verification, and final closeout gates all passed with no
follow-on split.
