# Official Metadata Addon Production Baseline — TODO

Status: Complete
Last updated: 2026-05-23

Task IDs use the `OMAPB` prefix.

## M0 — Scope And Baseline

- [x] OMAPB-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-production-baseline]
  Goal: Open the workstream with target state, task ledger, validation gates,
  and follow-on boundaries for live smoke, ranking/evidence, and TMDB baseline.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md agree.
  Evidence: Workstream docs.
  Handoff: Continue with OMAPB-020 live Nako smoke preflight.

## M1 — Live Nako Admin-Mediated Smoke

- [x] OMAPB-020 [owner=codex] [deps=OMAPB-010] [scope=addons/metadata-scraper,docs/workstreams/official-metadata-addon-production-baseline]
  Goal: Harden and run the local smoke path against a live Nako server when
  available: register/reuse disabled, health-check, enable, resource
  diagnostic, and redaction-safe output inspection.
  Validation: `pwsh -File addons/metadata-scraper/smoke.local.ps1 -SidecarBaseUrl <url>`; live Nako Admin smoke command when server/admin token are available; `git diff --check`.
  Review: The script must not start/stop Nako or print admin tokens, Addon raw
  tokens, provider tokens, raw diagnostic payloads, or raw Addon responses.
  Evidence: direct smoke output and live Nako smoke output or concrete
  external-blocker reason.
  Result: DONE_WITH_CONCERNS 2026-05-23. Direct sidecar smoke passed against a
  temporary sidecar on `127.0.0.1:19101`. Live Nako Admin-mediated smoke was
  not run because `127.0.0.1:3000` refused connections and `NAKO_ADMIN_TOKEN`
  was unset. The script remains the executable path for live evidence once
  those external conditions are met.
  Handoff: Continue with OMAPB-030 ranking/evidence model.

## M2 — Provider-Neutral Ranking And Evidence

- [x] OMAPB-030 [owner=codex] [deps=OMAPB-010] [scope=crates/nako-metadata-scraper/src/engine,crates/nako-metadata-scraper/src/providers]
  Goal: Replace provider-local confidence scoring with a provider-neutral
  ranking/evidence model owned by the engine.
  Validation: `cargo nextest run -p nako-metadata-scraper ranking evidence --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`.
  Review: Providers may report facts and match signals, but runtime owns final
  confidence and tie-break sorting.
  Evidence: tests for exact title/year/external-ID/language scoring,
  deterministic ties, and redaction-safe evidence serialization.
  Result: DONE 2026-05-23. Added `engine::ranking`, provider-neutral candidate
  facts, final runtime-owned confidence scoring, deterministic tie-breaks, and
  redaction-safe evidence serialization. Providers now return normalized facts
  rather than final confidence scores.
  Handoff: Continue with OMAPB-040 TMDB detail enrichment.

## M3 — TMDB Production Baseline

- [x] OMAPB-040 [owner=codex] [deps=OMAPB-030] [scope=crates/nako-metadata-scraper/src/providers/tmdb.rs,crates/nako-metadata-scraper/src/engine,addons/metadata-scraper]
  Goal: Expand TMDB from search-only proof to movie production baseline:
  bounded search, selected detail enrichment, external IDs, runtime, tagline,
  genres, and safe image/artifact metadata.
  Validation: `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast`; no live TMDB network in default gates; `git diff --check`.
  Review: TMDB must use shared `ProviderHttpRuntime`, fake HTTP tests, bounded
  top-result enrichment, and no copied provider fixtures.
  Evidence: synthetic tests proving search/detail/external-ID mapping,
  response-size/retry behavior remains shared, and artifacts do not expose
  secrets or raw provider bodies.
  Result: DONE 2026-05-23. TMDB now performs bounded movie search, detail, and
  external-ID enrichment through `ProviderHttpRuntime`; maps runtime, tagline,
  detail overview, detail genres, selected external IDs, and safe image-path
  metadata into patch/facts/tags without raw provider body exposure.
  Handoff: Continue with OMAPB-050 docs/examples update.

## M4 — Docs And Operator Truth

- [x] OMAPB-050 [owner=codex] [deps=OMAPB-020,OMAPB-040] [scope=README.md,addons/metadata-scraper,docs/workstreams/official-metadata-addon-production-baseline]
  Goal: Update README/addon README/examples to reflect live smoke behavior,
  ranking/evidence semantics, and TMDB baseline capabilities.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `git diff --check`.
  Review: Docs must not claim Bangumi/Douban/artwork/subtitle support before
  those providers exist.
  Evidence: docs match runtime truth.
  Result: DONE 2026-05-23. Root README and addon README now describe the TMDB
  movie baseline and provider-neutral ranking model without claiming future
  provider support.
  Handoff: Continue with OMAPB-060 closeout.

## M5 — Closeout Or Follow-On Split

- [x] OMAPB-060 [owner=planner] [deps=OMAPB-050] [scope=docs/workstreams/official-metadata-addon-production-baseline]
  Goal: Verify final evidence, close the lane, and split follow-ons for
  Bangumi/Douban, artwork, subtitles, rename/NFO, live-provider QA, or protocol
  expansion.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Use verify-rust-workstream before marking the lane complete.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Result: DONE 2026-05-23. Final JSON, fmt, package nextest, workspace
  nextest, and diff-check gates passed. Workstream is closed with live Nako
  Admin-mediated smoke split as an external-environment follow-on.
  Handoff: Record next executable product/provider lane.
