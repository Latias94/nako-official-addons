# Official Browser Worker Provider Render Drift Presets - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Planning

- [x] OBWRDP-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-browser-worker-provider-render-drift-presets]
  Goal: Open the provider-owned render drift preset lane.
  Validation: `python -m json.tool docs/workstreams/official-browser-worker-provider-render-drift-presets/WORKSTREAM.json`
  Review: Keep Browser Worker as executor, provider modules as source of URL/selector/action truth.
  Evidence: `DESIGN.md`

## M1 - Provider-Owned Presets

- [x] OBWRDP-020 [owner=codex] [deps=OBWRDP-010] [scope=crates/nako-metadata-scraper/src/providers]
  Goal: Add Browser Worker-compatible render drift case structs and provider-owned Douban/JavBus/JavLibrary case generation.
  Validation: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`
  Review: Generated cases must not emit provider cookies or other secrets.
  Evidence: `providers/render_drift.rs`

## M2 - CLI Handoff

- [x] OBWRDP-030 [owner=codex] [deps=OBWRDP-020] [scope=crates/nako-metadata-scraper/src/main.rs]
  Goal: Add `render-drift-cases` CLI output that Browser Worker can consume as `NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT_CASES`.
  Validation: `cargo run -q -p nako-metadata-scraper -- render-drift-cases`
  Review: CLI stdout should be pure JSON and avoid tracing/log prefix noise.
  Evidence: `main.rs`

## M3 - Docs And Closeout

- [x] OBWRDP-040 [owner=codex] [deps=OBWRDP-020,OBWRDP-030] [scope=addons/browser-worker/README.md,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md,docs/workstreams/official-browser-worker-provider-render-drift-presets]
  Goal: Document operator flow, run gates, record evidence, and commit.
  Validation: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`; `cargo run -q -p nako-metadata-scraper -- render-drift-cases`; `cargo fmt -p nako-metadata-scraper -- --check`; `npm test`; `python -m json.tool docs/workstreams/official-browser-worker-provider-render-drift-presets/WORKSTREAM.json`; `git diff --check`
  Review: Worktree must contain only intended preset and docs changes.
  Evidence: `EVIDENCE_AND_GATES.md`
