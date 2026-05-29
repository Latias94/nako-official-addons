# Official Metadata Addon Fearless Refactor — Handoff

Status: Complete
Last updated: 2026-05-23

## Current State

The workstream is closed. OMAFR-010 through OMAFR-090 are complete.

Current facts:

- `nako-official-addons` is on `main` and was clean when the lane opened.
- The repository currently has one crate, `nako-metadata-scraper`.
- Runtime configuration owns listen address, base URL, preferred language, and
  provider enablement defaults.
- The Addon Manifest configuration schema is generated from runtime-supported
  providers. It currently exposes `providers.fixture` only.
- Routes filter constructed providers by runtime enablement.
- Health and diagnostics report supported, enabled, and disabled provider IDs
  without exposing secrets.
- `providers::registry::ProviderRegistry` owns provider construction, ordering,
  capability descriptors, availability status, and redaction-safe diagnostics.
- The fixture provider is now an adapter behind the registry.
- `engine::MetadataScrapeRuntime` owns request normalization, provider
  fan-out, ranking, payload shaping, and provider failure swallowing.
- `routes.rs` is now an HTTP envelope adapter over the runtime.
- `providers::http_runtime::ProviderHttpRuntime` owns outbound provider HTTP
  timeout, bounded retry, User-Agent, optional proxy construction,
  response-size budget, JSON parsing, and retryability classification.
- `providers::tmdb::TmdbMetadataProvider` is the first real provider proof. It
  supports bounded TMDB movie search through the shared HTTP runtime, is
  disabled by default, and is unavailable until a read-access token is
  configured.
- `addons/metadata-scraper/smoke.local.ps1` verifies direct sidecar manifest,
  health, and metadata calls, and can optionally drive the Nako Admin
  registration/health/enable/resource diagnostic flow without printing tokens
  or provider secrets.
- `addons/metadata-scraper/manifest.example.json` is now covered by a runtime
  equality test so it cannot drift from `addon_manifest` unnoticed.
- Nako core has already completed Addon registration, Admin onboarding,
  tokens/grants, health checks, install guide, runtime readiness, routing
  plans, and artifact/intake handoff.

Reference repository findings:

- tinyMediaManager is Apache-2.0 and useful as a high-level capability map:
  metadata scrapers, artwork downloaders, trailers, subtitles, NFO, renaming,
  technical facts, collections, and TV show import.
- MDCx is GPLv3 with extra non-commercial terms in its README. Treat it as
  inspiration only. Do not copy implementation, tests, fixtures, schemas, or
  selectors.

## Active Task

- Task ID: none
- Owner: planner
- Status: CLOSED
- Evidence: OMAFR-020 through OMAFR-090 gates passed; see
  EVIDENCE_AND_GATES.md.

Previous task:

- Task ID: OMAFR-090
- Status: DONE
- Files:
  - `docs/workstreams/official-metadata-addon-fearless-refactor`
- Evidence:
  - `cargo fmt --all -- --check` passed.
  - Smoke script parse check passed.
  - `python -m json.tool` passed for `WORKSTREAM.json`.
  - `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with
    27 tests.
  - `cargo nextest run --workspace --no-fail-fast` passed with 27 tests.
  - `git diff --check` passed with only a Cargo.lock line-ending warning.

Previous implementation task:

- Task ID: OMAFR-080
- Status: DONE
- Files:
  - `README.md`
  - `addons/metadata-scraper`
  - `crates/nako-metadata-scraper/src/manifest.rs`
- Evidence:
  - README and examples now describe fixture default, optional TMDB proof, and
    operator-managed TMDB secret configuration consistently.
  - Added `checked_in_example_manifest_matches_runtime_manifest`.
  - `cargo nextest run -p nako-metadata-scraper manifest --no-fail-fast`
    passed.

Previous concern:

- Task ID: OMAFR-070
- Status: DONE_WITH_CONCERNS
- Files:
  - `addons/metadata-scraper`
  - `docs/workstreams/official-metadata-addon-fearless-refactor`
  - `README.md`
- Evidence:
  - PowerShell parse check passed for `smoke.local.ps1`.
  - `cargo build -p nako-metadata-scraper` passed.
  - Direct sidecar smoke passed against a temporary sidecar on
    `127.0.0.1:19100`.
  - Nako-mediated smoke is scripted but was not run because no local Nako
    server/admin token was available and `../nako` had unrelated dirty worktree
    changes.

## Decisions Since Last Update

- Open a local workstream in this repository rather than editing `../nako`
  workstreams.
- Make configuration and manifest truth the first executable slice.
- Build a deep runtime/provider seam before adding real provider breadth.
- Keep one installable addon and multiple internal provider adapters.
- Use tinyMediaManager and MDCx only for product capability inspiration.
- OMAFR-020 intentionally removed stale TMDB/Bangumi/Douban manifest settings
  until real adapters exist, because manifest configuration must not advertise
  provider settings the runtime ignores.
- OMAFR-030 deleted `default_providers()` and moved construction/diagnostics
  into `ProviderRegistry`.
- OMAFR-040 moved metadata request normalization and provider fan-out into
  `MetadataScrapeRuntime`.
- OMAFR-050 added `ProviderHttpRuntime` and `ReqwestProviderHttpTransport`.
  Tests use fake transport only; no live provider/network dependency was
  introduced into default test gates.
- OMAFR-060 added TMDB movie-search provider proof with synthetic tests only.
  No live TMDB call is part of default validation.
- OMAFR-070 added local smoke automation. The default path does not mutate
  Nako; Admin-mediated registration/enable/resource diagnostic requires
  explicit `-RegisterInNako` and an admin token or `-NoAdminAuth` for a local
  unauthenticated dev server.
- OMAFR-080 aligned docs/examples with runtime truth and added a test to keep
  the checked-in example manifest from drifting.

## Blockers

- None for this closed architecture lane.
- Residual external evidence item: Nako-mediated smoke still needs a live
  local Nako server plus admin auth. Direct sidecar smoke is verified.

## Next Recommended Action

- Open the next workstream for the highest-value follow-on:
  1. live Nako Admin-mediated smoke evidence;
  2. TMDB provider breadth beyond movie search;
  3. Bangumi/Douban provider adapters;
  4. artwork/subtitle provider lanes;
  5. rename planning and NFO-compatible sidecar workflows;
  6. bulk scrape, provider scoring, and ranking hardening.
