# Official Browser Worker Live Rendered Drift - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Planning

- Done: workstream opened around opt-in Browser Worker rendered-page health.

## M1 - Render Drift Harness

- Done: `render-drift.mjs` owns case normalization, suite execution, reports,
  and exit-code policy.
- Done: `npm run live:render-drift` starts an ephemeral worker and runs the
  fixture plus optional live cases.

## M2 - Redaction And Fixture Coverage

- Done: tests cover fixture-only default behavior, JSON live case parsing,
  report redaction, and a real local render path.

## M3 - Docs And Closeout

- Done: README documents operator-run live cases and final gates record fresh
  evidence.
