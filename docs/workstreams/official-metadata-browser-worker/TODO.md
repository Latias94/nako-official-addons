# Official Metadata Browser Worker - TODO

Status: Closed
Last updated: 2026-05-23

Task IDs use the `OMBW` prefix.

## M0 - Scope And Evidence Freeze

- [x] OMBW-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-browser-worker]
  Goal: Freeze the browser worker lane, its public/private boundary, and the first proof target.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md exist and agree.
  Evidence: `docs/workstreams/official-metadata-browser-worker/DESIGN.md`
  Handoff: DONE on 2026-05-23. Lane target state and public/private boundary are captured in DESIGN.md.

## M1 - Browser Worker First Proof

- [x] OMBW-020 [owner=codex] [deps=OMBW-010] [scope=addons/browser-worker]
  Goal: Scaffold the browser worker service with a health endpoint and a deterministic local rendered-page extraction proof.
  Validation: worker-specific smoke command and local fixture extraction test.
  Review: `review-workstream` before accepting completion.
  Evidence: worker service files, fixture test, smoke notes.
  Handoff: DONE on 2026-05-23. `npm test` and `npm run smoke` prove the local fixture text is changed by JavaScript before extraction.

## M2 - Metadata Scraper Integration

- [x] OMBW-030 [owner=codex] [deps=OMBW-020] [scope=crates/nako-metadata-scraper/src/providers,addons/metadata-scraper/compose.example.yml,addons/metadata-scraper/README.md]
  Goal: Add a browser-worker client adapter and wire the compose topology so the metadata scraper can call the worker.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `docker compose -f addons/metadata-scraper/compose.example.yml config`; direct sidecar smoke still passes.
  Review: The public addon must remain suggestion-only and must not embed Playwright.
  Evidence: provider adapter tests, compose example, README updates.
  Handoff: DONE on 2026-05-23. Rust provider, manifest/config diagnostics, compose topology, and direct sidecar smoke are verified.

## M3 - Douban Browser-Backed Baseline

- [x] OMBW-040 [owner=codex] [deps=OMBW-030] [scope=addons/browser-worker,crates/nako-metadata-scraper/src/providers,addons/metadata-scraper/smoke.local.ps1]
  Goal: Prove one Douban-backed search/detail extraction path through the browser worker.
  Validation: targeted browser-worker smoke and metadata-scraper tests; live Douban proof may be gated or replaced with a recorded fixture if external blocking persists.
  Review: Do not hide login, proxy, or anti-bot assumptions inside ordinary metadata calls.
  Evidence: worker proof, provider tests, smoke script or notes.
  Handoff: DONE on 2026-05-23 with recorded fixture-backed search/detail parsing through the browser-worker `/render` contract. Live Douban smoke remains a follow-on gate because access may depend on proxy, cookies, or rate limits.

## M4 - Docs And Closeout

- [x] OMBW-050 [owner=planner] [deps=OMBW-030,OMBW-040] [scope=README.md,addons/metadata-scraper,docs/workstreams/official-metadata-browser-worker]
  Goal: Update docs, record final gates, and close or split follow-ons.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Use `verify-rust-workstream` before marking complete.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Handoff: DONE on 2026-05-23. Lane closed with live Douban smoke split to follow-on hardening; fixture-backed `/render` + Rust parsing baseline is the shipped proof.
