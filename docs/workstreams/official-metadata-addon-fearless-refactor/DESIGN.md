# Official Metadata Addon Fearless Refactor

Status: Complete
Last updated: 2026-05-23

## Why This Lane Exists

`nako-official-addons` currently ships a small **Addon Sidecar** skeleton for
metadata suggestions. Nako core has already completed the Addon Protocol,
Admin Addon onboarding, token/grant operations, health checks, install guides,
runtime readiness, resource-call diagnostics, and side-effect handoff seams.

The addon repository now needs a fearless refactor before real provider breadth
arrives. The goal is not to preserve the current fixture-first shape; the goal
is to build the correct future-facing architecture while the repository is
small and has no compatibility burden.

## Relevant Authority

- Current addon code:
  - `crates/nako-metadata-scraper/src/config.rs`
  - `crates/nako-metadata-scraper/src/manifest.rs`
  - `crates/nako-metadata-scraper/src/routes.rs`
  - `crates/nako-metadata-scraper/src/engine/mod.rs`
  - `crates/nako-metadata-scraper/src/providers/mod.rs`
  - `crates/nako-metadata-scraper/src/providers/fixture.rs`
- Nako core authority:
  - `../nako/CONTEXT.md`
  - `../nako/docs/GOALS.md`
  - `../nako/docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `../nako/docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `../nako/docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
  - `../nako/docs/workstreams/addon-architecture-deepening/`
  - `../nako/docs/workstreams/admin-addon-operations-mvp/`
  - `../nako/docs/workstreams/admin-web-addon-onboarding/`
  - `../nako/docs/workstreams/admin-web-addon-credential-grant-onboarding/`
  - `../nako/docs/workstreams/addon-runtime-and-distribution/`
- Reference repository policy:
  - `README.md` in this repository.
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/tinyMediaManager/README.md`
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/tinyMediaManager/LICENSE`
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/mdcx/README.md`
  - `F:/SourceCodes/Rust/repo-ref/nako-scraper/mdcx/LICENSE`

## Problem

The current addon is intentionally shallow:

1. The manifest exposes `providers.fixture`, `providers.tmdb`,
   `providers.bangumi`, and `providers.douban`, but runtime configuration only
   reads listen address, base URL, and preferred language.
2. `default_providers()` always returns the fixture provider, so provider
   enablement, diagnostics, and configuration are declared but not enforced.
3. Route handlers own provider fan-out, sorting, payload shaping, and error
   swallowing directly. This makes the HTTP surface know too much about
   metadata scrape orchestration.
4. The provider interface is too small for future real providers: it lacks
   capabilities, availability, safe diagnostics, external identity, image or
   artwork candidate planning, and provider-specific confidence evidence.
5. There is no shared HTTP runtime for provider calls. Real providers would
   otherwise duplicate timeout, retry, rate-limit, proxy, user-agent, and
   secret redaction behavior.
6. The addon cannot yet drive a real end-to-end Nako flow: register disabled,
   start sidecar, health check, grant scopes, enable, call metadata resource,
   and inspect safe diagnostics.
7. Reference projects show mature media-manager capability breadth, but their
   code must not be copied. The repository needs an original Nako design that
   learns from those product surfaces without importing their implementation.

## Target State

When this lane closes:

- The addon has a deep `MetadataScrapeRuntime` module that owns request
  normalization, provider selection, provider fan-out, timeout-aware execution,
  candidate ranking, safe diagnostics, and response shaping.
- Runtime configuration and manifest configuration agree. Enabled providers,
  provider options, secret-reference field declarations, and diagnostics all
  describe the same model.
- Providers implement a richer interface with capability metadata,
  availability checks, request execution, normalized candidates, and safe
  failure classification.
- The fixture provider remains as a test adapter, not the architecture.
- Real provider work starts with one bounded provider adapter only after the
  runtime seam exists. TMDB is the preferred first provider because core Nako
  already has TMDB provider concepts and fixtures.
- The addon can be registered and exercised through Nako core's Admin Addon
  workflow in a repeatable local smoke path.
- Docs and examples explain install, configuration, health, and local smoke
  without implying Nako manages the sidecar process.
- Reference repositories are recorded as product capability inspiration only.
  No source code, schema, fixtures, generated files, comments, or line-by-line
  translations from reference projects enter this repository.

## In Scope

- Fearless refactor of `nako-metadata-scraper` modules.
- Deleting or replacing shallow current modules when deeper modules land.
- Provider registry and provider configuration model.
- Metadata scrape runtime and safe diagnostics.
- Provider HTTP runtime for outbound provider calls.
- Secret reference declaration and environment resolution for sidecar-owned
  provider credentials where needed.
- Fixture provider as an in-memory/test adapter.
- One real provider adapter as the first production proof, likely TMDB.
- Nako core end-to-end local smoke documentation and scripts if needed.
- Docker Compose/systemd example updates for the refactored sidecar.
- Tests using nextest where possible.

## Out Of Scope

- Copying, porting, translating, or deriving implementation code from
  tinyMediaManager, MDCx, MediaElch, Kodi scrapers, Jellyfin plugins, or other
  reference repositories.
- Addon Manager discovery, install, update, marketplace, package signing,
  process supervision, logs, rollback, or Docker socket control.
- Native Plugin ABI or Jellyfin Plugin Compatibility.
- Direct media-library file writes by the addon.
- Direct canonical metadata mutation outside Nako-owned Addon APIs.
- Broad provider matrix in the first proof. Do not implement TMDB, Bangumi,
  Douban, IMDb, TVDb, FanArt, subtitles, trailers, and rename planning in one
  slice.
- Admin Web UI changes unless a real smoke gap blocks this addon.
- Publishing `nako-addon-protocol` or this addon package.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The Addon Sidecar model is fixed for this phase. | High | Nako ADR 0003, 0015, and 0020 are accepted. | Reopen core ADRs before changing addon shape. |
| This repository has no deployed compatibility burden. | High | Only a skeleton addon exists and git is clean. | If users exist, add migration notes but still keep current protocol compatibility. |
| Provider configuration must become one shared model. | High | Manifest declares provider options that runtime ignores. | Provider breadth will spread config drift across route handlers. |
| A scrape runtime seam should precede real providers. | High | Route handler currently owns orchestration and response shaping. | First real provider will harden shallow handler logic. |
| tinyMediaManager can be used for capability inspiration. | Medium | Apache-2.0 license and high-level README capability list. | Keep use limited to product-level capability mapping unless legal review approves deeper use. |
| MDCx must be treated as inspiration only. | High | GPLv3 plus extra non-commercial terms in README. | Do not inspect or reuse implementation details. |
| TMDB is the best first real provider. | Medium | Core Nako already has TMDB concepts and provider experience. | If credential or API limits block it, use a contract-tested fake HTTP provider first. |

## Architecture Direction

Use Nako language:

- This repository provides one installable **Addon Sidecar**:
  `nako-metadata-scraper`.
- Internal provider modules are implementation detail. Users should not install
  one addon per provider unless trust, license, or deployment requirements
  differ.
- **Metadata Scrape** returns candidate metadata and evidence. Canonical writes
  happen only through Nako-owned acceptance or protected-write APIs.
- **Addon Hosted Page** diagnostics stay untrusted and redaction-safe.
- **Secret References** describe required secrets in the manifest. Resolved
  provider credentials must never appear in logs, health responses, diagnostics,
  or response payloads.

Deep modules to introduce:

1. `configuration`
   - Owns env parsing, provider enablement, provider runtime settings, and
     secret reference resolution.
   - Produces one immutable runtime config for the sidecar.
2. `provider_registry`
   - Owns provider construction and capability/availability diagnostics.
   - Keeps fixture/test adapters and real providers behind the same interface.
3. `provider_http_runtime`
   - Owns outbound HTTP timeout, retry, user-agent, proxy, rate-limit hooks,
     response-size limits, and safe error classification.
4. `metadata_scrape_runtime`
   - Owns payload normalization, provider ordering, fan-out policy, candidate
     normalization, ranking, response payload, and artifact shaping.
5. `diagnostics`
   - Owns health and hosted diagnostic facts so routes stay thin.

Deletion targets:

- Delete `default_providers()` once the registry owns provider construction.
- Keep `routes.rs` as an HTTP adapter only.
- Replace ad hoc JSON payload parsing in `engine::MetadataQuery::from_payload`
  with typed request normalization behind the scrape runtime.
- Remove provider declarations from the manifest if a provider cannot actually
  be enabled by configuration.

Reference capability map, inspiration only:

- tinyMediaManager high-level features suggest future lanes: metadata
  scrapers, artwork downloaders, trailers, subtitles, manual metadata editing,
  rename planning, NFO compatibility, media technical facts, collections, and
  TV show import workflows.
- MDCx high-level layout suggests separating command/UI/server/crawler/config
  concerns, but its GPLv3 plus extra terms mean no implementation use.
- The first Nako addon lane should use this map to avoid blocking future
  artwork, subtitle, rename planning, and bulk scrape workflows, not to
  implement them now.

## Closeout Condition

This lane can close when:

- configuration, provider registry, provider HTTP runtime, metadata scrape
  runtime, and diagnostics seams are implemented or explicitly split;
- fixture provider tests prove the new runtime shape without network calls;
- one real provider proof or a documented fake-provider substitute proves the
  outbound runtime shape;
- Nako core end-to-end smoke documentation exercises registration, health,
  grants, enablement, and metadata resource calls;
- example manifests, Docker Compose, and systemd snippets match the shipped
  configuration;
- `cargo fmt --all -- --check`, targeted nextest gates, workspace tests, and
  `git diff --check` pass;
- remaining provider breadth is split into named follow-ons.

## Closeout Summary

Closed on 2026-05-23.

This lane replaced the fixture-first skeleton with a future-facing Addon
Sidecar architecture:

- runtime configuration and manifest generation share one provider model;
- `ProviderRegistry` owns provider construction, capability descriptors,
  availability, enablement, and redaction-safe diagnostics;
- `MetadataScrapeRuntime` owns request normalization, provider fan-out,
  candidate sorting, artifact shaping, and safe failure swallowing;
- `ProviderHttpRuntime` owns outbound HTTP policy for real provider adapters;
- TMDB movie search exists as the first bounded real provider proof, disabled
  by default and unavailable until an operator supplies a read-access token;
- local sidecar smoke is scripted and verified;
- docs/examples are aligned with runtime truth and the checked-in example
  manifest is regression-tested against runtime generation.

The only remaining external validation is the Nako Admin-mediated smoke path,
which requires a running local Nako server and administrator token. It is
scripted through `addons/metadata-scraper/smoke.local.ps1` and split out as a
follow-on evidence item rather than kept as a blocker for this architecture
lane.
