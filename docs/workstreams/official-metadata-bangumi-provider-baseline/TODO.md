# Official Metadata Bangumi Provider Baseline - TODO

Status: Complete
Last updated: 2026-05-23

Task IDs use the `OMBGM` prefix.

## M0 - Scope And Baseline

- [x] OMBGM-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-bangumi-provider-baseline]
  Goal: Open the workstream with official API facts, target state,
  implementation slices, and verification gates.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: Workstream docs.
  Result: DONE 2026-05-23.
  Handoff: Continue with OMBGM-020 provider surface.

## M1 - Provider Surface

- [x] OMBGM-020 [owner=codex] [deps=OMBGM-010] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,crates/nako-metadata-scraper/src/providers]
  Goal: Add Bangumi to runtime-supported provider IDs, environment config,
  manifest provider schema, secret-reference fields, registry diagnostics, and
  provider catalog.
  Validation: `cargo nextest run -p nako-metadata-scraper config manifest registry --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Bangumi must be disabled by default and must not require a token for
  public API readiness, but an optional token must be redaction-safe.
  Evidence: config/manifest/registry tests.
  Result: DONE 2026-05-23. Bangumi is now a runtime-supported provider ID,
  disabled by default, listed in manifest provider schema, represented in safe
  diagnostics, and registry-buildable without a token for public read APIs.
  Handoff: Continue with OMBGM-030 Bangumi API adapter.

## M2 - Bangumi API Adapter

- [x] OMBGM-030 [owner=codex] [deps=OMBGM-020] [scope=crates/nako-metadata-scraper/src/providers/bangumi.rs]
  Goal: Implement bounded Bangumi subject search plus subject detail enrichment
  through `ProviderHttpRuntime`.
  Validation: `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast`; no live Bangumi network in default gates; `git diff --check`.
  Review: The adapter must send configured User-Agent through runtime config,
  optionally send bearer auth, map subject title/date/summary/tags/images into
  patch/facts, and never expose raw provider bodies or tokens.
  Evidence: fake-transport tests proving request shape, response mapping, and
  redaction-safe facts.
  Result: DONE 2026-05-23. Bangumi now performs bounded subject search and
  subject detail enrichment through `ProviderHttpRuntime`; fake transport tests
  prove User-Agent config, optional bearer auth, search body/query shape, detail
  URL, localized title mapping, release date/year, tags, image metadata, and
  provider-neutral facts.
  Handoff: Continue with OMBGM-040 docs/examples.

## M3 - Docs And Examples

- [x] OMBGM-040 [owner=codex] [deps=OMBGM-030] [scope=README.md,addons/metadata-scraper]
  Goal: Update operator docs and examples for Bangumi defaults, environment
  variables, User-Agent, and optional token behavior.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `git diff --check`.
  Review: Docs must not claim Douban, crawler, Playwright, or episode-level
  support in this lane.
  Evidence: README/addon README/example env updates.
  Result: DONE 2026-05-23. Root README, addon README, Dockerfile, compose, and
  systemd examples now describe Bangumi defaults, User-Agent, optional token,
  current mapped fields, and Douban/Playwright deferral.
  Handoff: Continue with OMBGM-050 closeout.

## M4 - Closeout

- [x] OMBGM-050 [owner=planner] [deps=OMBGM-040] [scope=docs/workstreams/official-metadata-bangumi-provider-baseline]
  Goal: Verify final evidence, close the lane, and split follow-ons for Douban
  crawler runtime, episode metadata, images/artwork, and live-provider QA.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Use verify-rust-workstream before marking the lane complete.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Result: DONE 2026-05-23. Final JSON, fmt, workspace nextest, and diff-check
  gates passed. Workstream is closed with Douban/crawler runtime, live provider
  QA, episode metadata, and artwork materialization split as follow-ons.
  Handoff: Open a dedicated Douban/crawler runtime design lane when ready.
