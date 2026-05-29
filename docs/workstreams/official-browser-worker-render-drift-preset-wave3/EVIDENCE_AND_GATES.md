# Official Browser Worker Render Drift Preset Wave3 - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Rust render drift tests | `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast` | Pass | 2026-05-27: 6 passed, 268 skipped. |
| CLI to Browser Worker parser | Generated expanded-provider CLI JSON, then parse with Browser Worker `parseRenderDriftCases` | Pass | 2026-05-27: parser returned 14 cases. |
| Rust fmt | `cargo fmt -p nako-metadata-scraper -- --check` | Pass | 2026-05-27. |
| Browser Worker tests | `npm test` in `addons/browser-worker` | Pass | 2026-05-27: 12 passed. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-browser-worker-render-drift-preset-wave3/WORKSTREAM.json` | Pass | 2026-05-27. |
| Diff hygiene | `git diff --check` | Pass | 2026-05-27. |

## Evidence Log

- 2026-05-27: Opened wave3 lane for remaining rendered AV provider presets.
- 2026-05-27: Added JavDB, FC2, FC2PPVDB, Caribbean, 1Pondo, and 10Musume
  generated render drift cases.
- 2026-05-27: Generated 14-provider CLI JSON and verified Browser Worker
  parser accepts all cases.
- 2026-05-27: Focused Rust render drift tests, Rust fmt check, and Browser
  Worker tests passed.
- 2026-05-27: Workstream JSON validation and diff hygiene passed.
