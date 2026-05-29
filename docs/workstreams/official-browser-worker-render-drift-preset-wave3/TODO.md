# Official Browser Worker Render Drift Preset Wave3 - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Planning

- [x] OBWRDW3-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-browser-worker-render-drift-preset-wave3]
  Goal: Open the wave3 provider render drift preset lane.
  Validation: `python -m json.tool docs/workstreams/official-browser-worker-render-drift-preset-wave3/WORKSTREAM.json`
  Review: Scope stays on generated drift presets for existing rendered providers.
  Evidence: `DESIGN.md`

## M1 - Remaining Rendered Presets

- [x] OBWRDW3-020 [owner=codex] [deps=OBWRDW3-010] [scope=crates/nako-metadata-scraper/src/providers]
  Goal: Add JavDB, FC2, FC2PPVDB, Caribbean, 1Pondo, and 10Musume generated render drift cases.
  Validation: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`
  Review: Cases must reuse provider-owned URL builders or shared site definitions.
  Evidence: `render_drift.rs`, `javdb.rs`, `fc2.rs`, `fc2ppvdb.rs`, `official_uncensored.rs`

## M2 - Docs And Closeout

- [x] OBWRDW3-030 [owner=codex] [deps=OBWRDW3-020] [scope=addons/browser-worker/README.md,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md,docs/workstreams/official-browser-worker-render-drift-preset-wave3]
  Goal: Document the expanded generated provider set, run gates, record evidence, and commit.
  Validation: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`; generated CLI JSON parses in Browser Worker; `cargo fmt -p nako-metadata-scraper -- --check`; `npm test`; `python -m json.tool docs/workstreams/official-browser-worker-render-drift-preset-wave3/WORKSTREAM.json`; `git diff --check`
  Review: Generated cases must remain Browser Worker-compatible and secret-free.
  Evidence: `EVIDENCE_AND_GATES.md`
