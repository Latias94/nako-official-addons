# Official Browser Worker Live AV Provider Tuning - Evidence And Gates

Status: Completed
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Focused Rust tests | `cargo nextest run -p nako-metadata-scraper rendered_page render_drift dmm_provider official_1pondo fc2_provider javbus_provider addon_manifest_configuration_schema_reflects_configured_provider_defaults config --no-fail-fast` | Pass | 2026-05-27: 45 passed, 230 skipped. |
| Browser Worker tests | `npm --prefix addons/browser-worker test` | Pass | 2026-05-27: 13 passed. |
| Live render drift | `npm --prefix addons/browser-worker run live:render-drift` | Drift observed | 2026-05-27: 14 cases, 9 ok, 5 remaining access/network failures with local proxy. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-browser-worker-live-av-provider-tuning/WORKSTREAM.json` | Pass | 2026-05-27: JSON validated successfully. |
| Formatting | `cargo fmt -p nako-metadata-scraper` | Pass | 2026-05-27: formatting applied. |
| Diff hygiene | `git diff --check` | Pass | 2026-05-27: no patch hygiene issues. |

## Evidence Log

- 2026-05-27: Opened provider tuning lane as a follow-up to
  `official-browser-worker-live-av-drift-sampling`.
- 2026-05-27: Added safe `headers_from_env` support so generated live cases can
  reference provider cookies without emitting cookie values.
- 2026-05-27: Changed Browser Worker selector waits to DOM attachment by
  default. DMM search links are parseable but not always visible, so visibility
  waits produced false drift.
- 2026-05-27: DMM now sends an age-confirmation cookie by default and exposes a
  configurable cookie secret reference. Generated DMM drift references
  `NAKO_METADATA_SCRAPER_DMM_COOKIE`.
- 2026-05-27: JavBus generated drift references
  `NAKO_METADATA_SCRAPER_JAVBUS_COOKIE`; a real operator cookie is still
  required for gated live detail pages.
- 2026-05-27: FC2 and official uncensored providers now use larger production
  render budgets and generated live drift budgets. 1Pondo uses the canonical
  directory-style detail path.
- 2026-05-27: Live drift improved from the prior 5/14 passing baseline to 9/14
  passing with the local proxy. Passing cases included Douban, DMM, XCity,
  AirAV, JavDB, FC2, Caribbean, 1Pondo, and 10Musume. Remaining failures were
  JavBus operator cookie/browser access, JavLibrary and MGStage blocking, and
  AVSox/FC2PPVDB network failures.
- 2026-05-27: Final formatting, JSON validation, and diff hygiene all passed.
