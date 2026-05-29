# Official Browser Worker Live AV Drift Sampling - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Planning

- [x] OBWLADS-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-browser-worker-live-av-drift-sampling]
  Goal: Open a live AV render drift sampling lane.
  Validation: `python -m json.tool docs/workstreams/official-browser-worker-live-av-drift-sampling/WORKSTREAM.json`
  Review: Evidence must be redaction-safe and not store URLs or secrets.
  Evidence: `DESIGN.md`

## M1 - Live Sample

- [x] OBWLADS-020 [owner=codex] [deps=OBWLADS-010] [scope=addons/browser-worker,crates/nako-metadata-scraper]
  Goal: Generate the 14-provider live case set and run Browser Worker live drift with the configured proxy.
  Validation: `npm --prefix addons/browser-worker run live:render-drift`
  Review: Record case IDs, statuses, failure kinds, safe codes, sizes, and proxy policy only.
  Evidence: `EVIDENCE_AND_GATES.md`

## M2 - Fix Or Document Findings

- [x] OBWLADS-030 [owner=codex] [deps=OBWLADS-020] [scope=addons/browser-worker,crates/nako-metadata-scraper,docs/workstreams/official-browser-worker-live-av-drift-sampling]
  Goal: Fix clear preset defects found by live sampling or document external/environmental blockers.
  Validation: focused Rust/Browser Worker gates for any changed code; workstream JSON; diff hygiene.
  Review: Do not add secrets, cookies, or proxy URLs to source-controlled evidence.
  Evidence: `EVIDENCE_AND_GATES.md`

## M3 - Closeout

- [x] OBWLADS-040 [owner=codex] [deps=OBWLADS-030] [scope=docs/workstreams/official-browser-worker-live-av-drift-sampling]
  Goal: Finalize evidence, run gates, and commit.
  Validation: `python -m json.tool docs/workstreams/official-browser-worker-live-av-drift-sampling/WORKSTREAM.json`; `git diff --check`
  Review: Workstream must distinguish code defects from live-site/operator access outcomes.
  Evidence: `HANDOFF.md`
