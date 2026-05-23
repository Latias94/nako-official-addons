# Official Metadata Browser Worker - TODO

Status: Active
Last updated: 2026-05-23

Task IDs use the `OMBW` prefix.

## M0 - Scope And Evidence Freeze

- [ ] OMBW-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-browser-worker]
  Goal: Freeze the browser worker lane, its public/private boundary, and the first proof target.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md exist and agree.
  Evidence: `docs/workstreams/official-metadata-browser-worker/DESIGN.md`
  Handoff: Planner owns this before implementation starts.

## M1 - Browser Worker First Proof

- [ ] OMBW-020 [owner=unassigned] [deps=OMBW-010] [scope=addons/browser-worker]
  Goal: Scaffold the browser worker service with a health endpoint and a deterministic local rendered-page extraction proof.
  Validation: worker-specific smoke command and local fixture extraction test.
  Review: `review-workstream` before accepting completion.
  Evidence: worker service files, fixture test, smoke notes.
  Handoff: Keep the proof isolated from Douban until the worker contract is stable.

## M2 - Metadata Scraper Integration

- [ ] OMBW-030 [owner=unassigned] [deps=OMBW-020] [scope=crates/nako-metadata-scraper/src/providers,addons/metadata-scraper/compose.example.yml,addons/metadata-scraper/README.md]
  Goal: Add a browser-worker client adapter and wire the compose topology so the metadata scraper can call the worker.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `docker compose -f addons/metadata-scraper/compose.example.yml config`; direct sidecar smoke still passes.
  Review: The public addon must remain suggestion-only and must not embed Playwright.
  Evidence: provider adapter tests, compose example, README updates.
  Handoff: Split follow-on work if the worker contract needs to grow.

## M3 - Douban Browser-Backed Baseline

- [ ] OMBW-040 [owner=unassigned] [deps=OMBW-030] [scope=addons/browser-worker,crates/nako-metadata-scraper/src/providers,addons/metadata-scraper/smoke.local.ps1]
  Goal: Prove one Douban-backed search/detail extraction path through the browser worker.
  Validation: targeted browser-worker smoke and metadata-scraper tests; live Douban proof may be gated or replaced with a recorded fixture if external blocking persists.
  Review: Do not hide login, proxy, or anti-bot assumptions inside ordinary metadata calls.
  Evidence: worker proof, provider tests, smoke script or notes.
  Handoff: Continue with broader browser-backed providers or split them out.

## M4 - Docs And Closeout

- [ ] OMBW-050 [owner=planner] [deps=OMBW-030,OMBW-040] [scope=README.md,addons/metadata-scraper,docs/workstreams/official-metadata-browser-worker]
  Goal: Update docs, record final gates, and close or split follow-ons.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: Use `verify-rust-workstream` before marking complete.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Handoff: Summarize remaining risk in HANDOFF.md.
