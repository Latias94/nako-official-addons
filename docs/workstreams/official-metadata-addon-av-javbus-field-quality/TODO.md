# Task Ledger

Prefix: JBFQ

## Completed

- [x] JBFQ-010 - Open JavBus field-quality follow-up
  - Record MDCx behavioral findings, GPL guardrails, validation gates, and first executable slice.
  - Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-javbus-field-quality/WORKSTREAM.json`

- [x] JBFQ-020 - Improve JavBus detail field extraction
  - Harden detail parsing for the current browser-rendered JavBus page shape.
  - Extract release date, runtime, actors, genres, studio, publisher, director, series, primary image, and sample images where present.
  - Add synthetic fixture coverage for table/label/link/image variants observed from live rendering.
  - Validation: `cargo nextest run -p nako-metadata-scraper javbus rendered_av --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper javbus rendered_page manifest --no-fail-fast`; `npm --prefix addons/browser-worker test`

- [x] JBFQ-030 - Add redaction-safe live evidence for JavBus field health
  - Re-run live drift through browser-worker with proxy configured.
  - Record field presence summaries without raw local file names or raw AV numbers.
  - Evidence: live drift reached JavBus but returned the age-verification flow without an operator cookie; the provider now rejects that page shape instead of emitting a false candidate.
  - Validation: `cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored av_live_provider_field_health_smoke --nocapture` is blocked without `NAKO_METADATA_SCRAPER_JAVBUS_COOKIE`.

## Completed

- [x] JBFQ-040 - Verify and close the JavBus quality lane
  - Run package tests, format check, JSON validation, and diff hygiene.
  - Update workstream status, handoff, and evidence.
  - Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `git diff --check`

## Active

None.

## Follow-Up Candidates

- Configurable MDCx-style per-field AV provider preference presets.
- More live drift cases for uncensored and western-style JavBus routes.
