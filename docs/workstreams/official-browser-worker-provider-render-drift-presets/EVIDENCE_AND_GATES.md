# Official Browser Worker Provider Render Drift Presets - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Rust render drift tests | `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast` | Pass | 2026-05-27: 2 passed, 268 skipped. |
| CLI JSON output | `cargo run -q -p nako-metadata-scraper -- render-drift-cases` | Pass | 2026-05-27: emitted enabled Douban/JavBus/JavLibrary case array with example config. |
| Rust fmt | `cargo fmt -p nako-metadata-scraper -- --check` | Pass | 2026-05-27. |
| Browser Worker parser/tests | `npm test` in `addons/browser-worker` | Pass | 2026-05-27: 12 passed; generated CLI JSON parsed to 3 cases in a targeted parser check. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-browser-worker-provider-render-drift-presets/WORKSTREAM.json` | Pass | 2026-05-27. |
| Diff hygiene | `git diff --check` | Pass | 2026-05-27. |

## Evidence Log

- 2026-05-27: Added provider-owned render drift case generation for Douban,
  JavBus, and JavLibrary.
- 2026-05-27: Added `render-drift-cases` CLI path before tracing/server startup
  so stdout can be used as Browser Worker env JSON.
- 2026-05-27: Updated Browser Worker and metadata-scraper docs with the
  provider-owned preset flow.
- 2026-05-27: Generated cases now preserve safe `proxy_policy` defaults while
  still omitting session keys, cookies, and header values.
- 2026-05-27: Final gates passed: `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`;
  `cargo run -q -p nako-metadata-scraper -- render-drift-cases | python -m json.tool`;
  `cargo fmt -p nako-metadata-scraper -- --check`; `npm test`;
  `python -m json.tool docs/workstreams/official-browser-worker-provider-render-drift-presets/WORKSTREAM.json`;
  `git diff --check`.
