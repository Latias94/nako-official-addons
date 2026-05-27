# Official Browser Worker Live Rendered Drift - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Planning

- [x] OBWLD-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-browser-worker-live-rendered-drift]
  Goal: Open the live rendered drift lane and freeze the opt-in/redaction boundary.
  Validation: `python -m json.tool docs/workstreams/official-browser-worker-live-rendered-drift/WORKSTREAM.json`
  Review: Keep the worker as render execution health, not provider parsing.
  Evidence: `DESIGN.md`

## M1 - Render Drift Harness

- [x] OBWLD-020 [owner=codex] [deps=OBWLD-010] [scope=addons/browser-worker/src/render-drift.mjs,addons/browser-worker/scripts/live-render-drift.mjs,addons/browser-worker/package.json]
  Goal: Add a default offline fixture suite and explicit live-case harness for Browser Worker rendered-page drift.
  Validation: `npm test`; `npm run live:render-drift`
  Review: Live cases must be opt-in and reports must not echo URLs, selectors, headers, cookies, proxy URLs, or raw page text.
  Evidence: `render-drift.mjs`; `live-render-drift.mjs`

## M2 - Redaction And Fixture Coverage

- [x] OBWLD-030 [owner=codex] [deps=OBWLD-020] [scope=addons/browser-worker/test/render-drift.test.mjs]
  Goal: Cover case parsing, redaction-safe failure reports, and the fixture render path.
  Validation: `npm test`
  Review: Tests should prove sensitive live inputs stay out of report JSON.
  Evidence: `render-drift.test.mjs`

## M3 - Docs And Closeout

- [x] OBWLD-040 [owner=codex] [deps=OBWLD-020,OBWLD-030] [scope=addons/browser-worker/README.md,docs/workstreams/official-browser-worker-live-rendered-drift]
  Goal: Document live drift invocation, run gates, record evidence, and commit.
  Validation: `npm test`; `npm run smoke`; `npm run live:render-drift`; `python -m json.tool docs/workstreams/official-browser-worker-live-rendered-drift/WORKSTREAM.json`; `git diff --check`
  Review: Worktree must contain only intended Browser Worker live drift changes.
  Evidence: `EVIDENCE_AND_GATES.md`
