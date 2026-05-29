# Official Browser Worker Live Rendered Drift - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Worker tests | `npm test` in `addons/browser-worker` | Pass | 2026-05-27: 12 passed. |
| Worker smoke | `npm run smoke` in `addons/browser-worker` | Pass | 2026-05-27: existing `/health`, `/extract`, `/render` smoke passed. |
| Render drift fixture | `npm run live:render-drift` in `addons/browser-worker` | Pass | 2026-05-27: default offline fixture returned `status=ok`, `live_enabled=false`, 1 case, 0 failures. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-browser-worker-live-rendered-drift/WORKSTREAM.json` | Pass | 2026-05-27. |
| Diff hygiene | `git diff --check` | Pass | 2026-05-27. |

## Evidence Log

- 2026-05-27: Opened lane after the render runtime hardening closeout. Scope is
  explicit: worker render health only, not metadata parsing.
- 2026-05-27: Added `render-drift.mjs`, `scripts/live-render-drift.mjs`, the
  `live:render-drift` npm script, parser/redaction/fixture tests, and README
  operator instructions.
- 2026-05-27: Verified default drift output is redaction-safe and contains only
  case id, source, HTTP status, failure codes, booleans, byte counts, and timing.
- 2026-05-27: Final gates passed: `npm test`; `npm run smoke`;
  `npm run live:render-drift`; `python -m json.tool docs/workstreams/official-browser-worker-live-rendered-drift/WORKSTREAM.json`;
  `git diff --check`.
