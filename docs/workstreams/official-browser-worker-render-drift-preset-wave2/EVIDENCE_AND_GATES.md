# Official Browser Worker Render Drift Preset Wave2 - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Rust render drift tests | `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast` | Pass | 2026-05-27: 4 passed, 268 skipped. |
| CLI to Browser Worker parser | Generated 8-provider CLI JSON, then parsed with Browser Worker `parseRenderDriftCases` | Pass | 2026-05-27: parser returned 8 cases. |
| Rust fmt | `cargo fmt -p nako-metadata-scraper -- --check` | Pass | 2026-05-27. |
| Browser Worker tests | `npm test` in `addons/browser-worker` | Pass | 2026-05-27: 12 passed. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-browser-worker-render-drift-preset-wave2/WORKSTREAM.json` | Pass | 2026-05-27. |
| Diff hygiene | `git diff --check` | Pass | 2026-05-27. |

## Evidence Log

- 2026-05-27: Added DMM search and MGStage detail render drift cases.
- 2026-05-27: Added generic `RenderedSearchAvSite` render drift cases and wired
  XCity, AirAV, and AVSox.
- 2026-05-27: Generated 8-provider CLI JSON and verified Browser Worker parser
  accepts all cases.
- 2026-05-27: Final gates passed: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`;
  generated 8-provider CLI JSON and parsed it with Browser Worker; `cargo fmt -p nako-metadata-scraper -- --check`;
  `npm test`; `python -m json.tool docs/workstreams/official-browser-worker-render-drift-preset-wave2/WORKSTREAM.json`;
  `git diff --check`.
