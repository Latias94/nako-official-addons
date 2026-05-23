# Official Metadata Addon Fearless Refactor — TODO

Status: Complete
Last updated: 2026-05-23

Task IDs use the `OMAFR` prefix.

## M0 — Scope And Evidence Freeze

- [x] OMAFR-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-fearless-refactor]
  Goal: Open the workstream with problem, target state, non-goals, reference
  policy, and validation gates.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: `docs/workstreams/official-metadata-addon-fearless-refactor/DESIGN.md`.
  Handoff: Continue with OMAFR-020 runtime/configuration module plan and tests.

## M1 — Configuration And Manifest Truth

- [x] OMAFR-020 [owner=codex] [deps=OMAFR-010] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,crates/nako-metadata-scraper/src/routes.rs,addons/metadata-scraper]
  Goal: Replace the shallow config shape with one authoritative sidecar
  configuration model that drives manifest provider declarations, enabled
  providers, Secret Reference fields, and diagnostics.
  Validation: `cargo nextest run -p nako-metadata-scraper config manifest --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Manifest must not advertise provider settings that runtime ignores.
  Evidence: focused config/manifest tests and updated example manifest/docs.
  Result: DONE 2026-05-23. `Config` now owns supported provider defaults,
  manifest configuration schema is generated from runtime config, routes filter
  enabled providers, and health/diagnostics report supported/enabled/disabled
  providers without exposing secrets.
  Handoff: Continue with OMAFR-030 provider registry.

## M2 — Provider Registry And Capability Diagnostics

- [x] OMAFR-030 [owner=codex] [deps=OMAFR-020] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/engine]
  Goal: Replace `default_providers()` with a provider registry module that owns
  provider construction, provider ordering, capabilities, availability, and
  redaction-safe diagnostics.
  Validation: `cargo nextest run -p nako-metadata-scraper provider --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Fixture must become one adapter behind the registry, not the default
  architecture.
  Evidence: provider registry tests for enabled/disabled/unavailable providers.
  Result: DONE 2026-05-23. `ProviderRegistry` now owns the provider catalog,
  construction, enablement filtering, capability descriptors, ready/disabled/
  unavailable status, and health diagnostics. `default_providers()` was
  deleted.
  Handoff: Continue with OMAFR-040 metadata scrape runtime.

## M3 — Metadata Scrape Runtime

- [x] OMAFR-040 [owner=codex] [deps=OMAFR-030] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/routes.rs]
  Goal: Introduce `MetadataScrapeRuntime` so routes only adapt HTTP envelopes
  while runtime owns request normalization, provider fan-out, ranking,
  artifacts, safe failure handling, and response shaping.
  Validation: `cargo nextest run -p nako-metadata-scraper metadata --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Route handlers must not know provider ordering, candidate ranking, or
  provider failure vocabulary.
  Evidence: route and runtime tests prove identical fixture behavior through
  the new seam.
  Result: DONE 2026-05-23. `MetadataScrapeRuntime` now owns request
  normalization, provider fan-out, candidate sorting, response/artifact
  shaping, and provider failure swallowing. `routes.rs` only adapts HTTP
  envelopes and uses the runtime result.
  Handoff: Continue with OMAFR-050 outbound provider HTTP runtime.

## M4 — Provider HTTP Runtime

- [x] OMAFR-050 [owner=codex] [deps=OMAFR-040] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/Cargo.toml]
  Goal: Add a sidecar-owned provider HTTP runtime for real provider adapters:
  timeout, retry, user-agent, optional proxy, response-size budget, and
  redaction-safe error classification.
  Validation: `cargo nextest run -p nako-metadata-scraper http_runtime --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: No provider adapter may construct an ad hoc HTTP client with its own
  timeout/retry/secret logging behavior.
  Evidence: fake transport tests for retryable, non-retryable, timeout, and
  redaction cases.
  Result: DONE 2026-05-23. `ProviderHttpRuntime` now owns provider HTTP
  timeout, bounded retry, User-Agent, optional proxy client construction,
  response-size limits, JSON parsing, and retryable/non-retryable error
  classification through a fake-transport-testable boundary.
  Handoff: Continue with OMAFR-060 first real provider proof.

## M5 — First Real Provider Proof

- [x] OMAFR-060 [owner=codex] [deps=OMAFR-050] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/engine,addons/metadata-scraper]
  Goal: Add one bounded real provider adapter or a fake-HTTP provider proof if
  external credentials are unavailable. Prefer TMDB because Nako core already
  has TMDB domain experience.
  Validation: `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast`; no live-network test unless explicitly gated by env; `git diff --check`.
  Review: Tests must use checked-in original fixtures or synthetic data, not
  copied reference-project fixtures.
  Evidence: provider mapping tests and disabled/missing-secret diagnostics.
  Result: DONE 2026-05-23. Added bounded TMDB movie-search provider proof on
  top of the shared provider HTTP runtime. TMDB is disabled by default, becomes
  available only when a read-access token is configured, and contributes a
  manifest secret-reference field when enabled.
  Handoff: Continue with OMAFR-070 Nako end-to-end smoke.

## M6 — Nako End-To-End Smoke

- [x] OMAFR-070 [owner=codex] [deps=OMAFR-040] [scope=addons/metadata-scraper,docs/workstreams/official-metadata-addon-fearless-refactor,README.md]
  Goal: Document and, where practical, script a local smoke flow against
  `../nako`: start sidecar, register disabled, issue token/grants if needed,
  health check, enable, call metadata resource, and inspect safe diagnostics.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; manual smoke evidence when Nako server is available; `git diff --check`.
  Review: Smoke must not require Nako to install, start, stop, or supervise the
  sidecar process.
  Evidence: smoke commands, expected safe outputs, and any scripts/docs.
  Result: DONE_WITH_CONCERNS 2026-05-23. Added a local PowerShell smoke script
  that verifies `/manifest.json`, `/health`, and `/metadata` directly, and can
  optionally register/reuse the manifest through Nako Admin API, run health
  check, enable, issue an Addon Token, and call the redaction-safe metadata
  resource diagnostic. Direct sidecar smoke passed against a temporary local
  sidecar on `127.0.0.1:19100`. Nako-mediated smoke was documented/scripted
  but not run because no local Nako server/admin token was available and
  `../nako` had unrelated dirty worktree changes.
  Handoff: Continue with OMAFR-080 docs/examples cleanup.

## M7 — Docs, Examples, And Deletion Sweep

- [x] OMAFR-080 [owner=codex] [deps=OMAFR-060,OMAFR-070] [scope=README.md,addons/metadata-scraper,crates/nako-metadata-scraper]
  Goal: Align README, addon README, Dockerfile, compose, systemd, example
  manifest, and code layout with the refactored architecture; delete obsolete
  shallow helpers and stale docs.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo test --doc --workspace` if docs add Rust examples; `git diff --check`.
  Review: No stale provider claims, copied reference artifacts, or dead modules.
  Evidence: docs/examples reflect runtime truth.
  Result: DONE 2026-05-23. Root README, addon README, Dockerfile, compose,
  systemd, and manifest example now agree on fixture-by-default,
  TMDB-disabled-by-default runtime truth. Added a regression test proving
  `addons/metadata-scraper/manifest.example.json` matches the runtime-generated
  manifest for the compose base URL. Removed the misleading systemd
  `env:TMDB_READ_ACCESS_TOKEN` sample and documented `EnvironmentFile` for
  operator-managed secrets.
  Handoff: Continue with OMAFR-090 closeout.

## M8 — Closeout Or Follow-On Split

- [x] OMAFR-090 [owner=planner] [deps=OMAFR-080] [scope=docs/workstreams/official-metadata-addon-fearless-refactor]
  Goal: Verify final evidence, close the lane, and split provider breadth
  follow-ons for Bangumi, Douban, artwork, subtitle, rename planning, or bulk
  scrape if they are not complete.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Use review-workstream and verify-rust-workstream before marking the
  lane complete.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Result: DONE 2026-05-23. Final gates passed. The architecture lane is
  closed, and remaining product/provider breadth is split into follow-ons:
  live Nako Admin-mediated smoke, TMDB breadth, Bangumi/Douban providers,
  artwork/subtitle lanes, rename/NFO planning, and bulk scrape/scoring
  hardening.
  Handoff: Record next executable provider/product lane.
