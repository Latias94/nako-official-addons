# Official Browser Worker Render Drift Preset Wave2 - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Planning

- [x] OBWRDW2-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-browser-worker-render-drift-preset-wave2]
  Goal: Open the wave2 provider render drift preset lane.
  Validation: `python -m json.tool docs/workstreams/official-browser-worker-render-drift-preset-wave2/WORKSTREAM.json`
  Review: Continue provider-owned URL/selector/action ownership.
  Evidence: `DESIGN.md`

## M1 - Wave2 Presets

- [x] OBWRDW2-020 [owner=codex] [deps=OBWRDW2-010] [scope=crates/nako-metadata-scraper/src/providers]
  Goal: Add DMM, MGStage, XCity, AirAV, and AVSox generated render drift cases.
  Validation: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`
  Review: Generic `RenderedSearchAvSite` cases should reuse the site search URL builder.
  Evidence: `rendered_search_av.rs`, `dmm.rs`, `mgstage.rs`, `render_drift.rs`

## M2 - Docs And Closeout

- [x] OBWRDW2-030 [owner=codex] [deps=OBWRDW2-020] [scope=addons/browser-worker/README.md,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md,docs/workstreams/official-browser-worker-render-drift-preset-wave2]
  Goal: Document the expanded provider set, run gates, record evidence, and commit.
  Validation: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`; `cargo run -q -p nako-metadata-scraper -- render-drift-cases`; `cargo fmt -p nako-metadata-scraper -- --check`; `npm test`; `python -m json.tool docs/workstreams/official-browser-worker-render-drift-preset-wave2/WORKSTREAM.json`; `git diff --check`
  Review: Generated cases must stay Browser Worker-compatible and secret-free.
  Evidence: `EVIDENCE_AND_GATES.md`
