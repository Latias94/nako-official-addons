# Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Required Gates

| Gate | Command | Status | Evidence |
| --- | --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-av-javbus-field-quality/WORKSTREAM.json` | Passed | JSON parsed successfully. |
| Targeted JavBus parser tests | `cargo nextest run -p nako-metadata-scraper javbus rendered_av --no-fail-fast` | Passed | 9 passed. |
| Render intent and manifest tests | `cargo nextest run -p nako-metadata-scraper javbus rendered_page manifest --no-fail-fast` | Passed | 17 passed. |
| Browser-worker contract tests | `npm --prefix addons/browser-worker test` | Passed | 4 passed. |
| JavBus live drift smoke | `cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored av_live_provider_field_health_smoke --nocapture` | Blocked | Browser-worker reached JavBus through proxy, but the live page is an age-verification flow without an operator cookie. Candidate count is now 0 rather than a false metadata candidate. |
| Package tests | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Passed | 245 passed, 3 skipped. |
| Formatting | `cargo fmt -p nako-metadata-scraper -- --check` | Passed | No formatting changes required after `cargo fmt -p nako-metadata-scraper`. |
| Diff hygiene | `git diff --check` | Passed | No whitespace errors. Git reported line-ending conversion warnings for edited browser-worker JS files. |

## Evidence Log

- 2026-05-27: Reconstructed current AV workstream state; prior MDCx-style AV scraping lane is complete.
- 2026-05-27: Reviewed `repo-ref/mdcx` behaviorally. JavBus is a high-value field source for actors, artwork, sample images, release date, runtime, director, series, studio, and publisher; Nako should keep batch orchestration in `bulk-metadata-scrape`.
- 2026-05-27: Added direct-detail-first JavBus lookup, search fallback, richer detail parser coverage, primary/sample image URL normalization, optional JavBus cookie forwarding, browser-worker page headers/actions, and age-verification rejection.
- 2026-05-27: Live proxy smoke without `NAKO_METADATA_SCRAPER_JAVBUS_COOKIE` returned redaction-safe field health with `candidate_count: 0`; this is the correct access-gate result after rejecting age-verification pages.
- 2026-05-27: Closeout gates passed: 245 metadata-scraper tests, 4 browser-worker tests, Rust formatting, workstream JSON, and diff hygiene.
