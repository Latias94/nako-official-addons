# Official Browser Worker Live AV Drift Sampling - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Case generation | `cargo run -q -p nako-metadata-scraper -- render-drift-cases` | Pass | 2026-05-27: generated 14 rendered AV provider cases. |
| Live render drift | `npm --prefix addons/browser-worker run live:render-drift` | Drift observed | 2026-05-27: 14 cases, 5 ok, 9 drift/error after worker classification fixes. |
| Focused Rust tests | `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast` | Pass | 2026-05-27: 6 passed, 268 skipped. |
| Browser Worker tests | `npm test` in `addons/browser-worker` | Pass | 2026-05-27: 13 passed. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-browser-worker-live-av-drift-sampling/WORKSTREAM.json` | Pass | 2026-05-27. |
| Diff hygiene | `git diff --check` | Pass | 2026-05-27. |

## Evidence Log

- 2026-05-27: Opened live AV render drift sampling lane.
- 2026-05-27: Initial live sample exposed that Crawlee WARN/ERROR logs printed
  target URLs even though final suite JSON was redaction-safe.
- 2026-05-27: Disabled Crawlee logs by default and kept `CRAWLEE_LOG_LEVEL` as
  an explicit local debugging escape hatch.
- 2026-05-27: Added redaction-safe mapping for blocked, timeout, and network
  failures so live drift reports are actionable instead of collapsing to
  `render_extraction_empty`.
- 2026-05-27: Disabled Browser Worker internal Crawlee request retries for
  single render intents; retry policy belongs to provider/bulk orchestration.
- 2026-05-27: Re-ran 14-provider live sample with configured local proxy and
  verified no target URL, selector, sample number, or proxy URL leaked in
  command output.
- 2026-05-27: Final gates passed: focused Rust render drift tests, Browser
  Worker tests, workstream JSON validation, and diff hygiene.

## Live Sample Summary

The final redaction-safe live sample produced:

| Case | Status | Failure kind | Safe code | HTML bytes | Text bytes |
| --- | --- | --- | --- | ---: | ---: |
| douban-search | ok | - | - | 32175 | 2665 |
| dmm-search | error | selector_timeout | render_wait_selector_timeout | 0 | 0 |
| javbus-detail | error | selector_timeout | render_wait_selector_timeout | 0 | 0 |
| javlibrary-search | error | auth_or_forbidden | render_request_blocked | 0 | 0 |
| xcity-search | ok | - | - | 47037 | 3800 |
| airav-search | ok | - | - | 6716 | 636 |
| avsox-search | error | provider_error | render_network_failed | 0 | 0 |
| mgstage-detail | error | auth_or_forbidden | render_request_blocked | 0 | 0 |
| javdb-search | ok | - | - | 31660 | 2194 |
| fc2-detail | error | render_timeout | render_timeout | 0 | 0 |
| fc2ppvdb-detail | error | provider_error | render_network_failed | 0 | 0 |
| caribbean-detail | error | render_timeout | render_timeout | 0 | 0 |
| 1pondo-detail | error | selector_timeout | render_wait_selector_timeout | 0 | 0 |
| 10musume-detail | ok | - | - | 2175 | 342 |

Interpretation:

- The generated cases are operationally useful: five providers returned
  rendered HTML with selectors satisfied through the configured proxy.
- The remaining failures are live-site/operator conditions or site drift, not
  JSON-shape issues: blocked access, selector drift, network closure, and
  timeout.
- Follow-up should tune provider samples/selectors and optional cookies per
  provider, not expand generated provider coverage further.
