# Task Ledger

Prefix: OMAV

## Completed

- [x] OMAV-010 - Open MDCx-style AV scraping workstream
  - Record reference findings, GPL guardrails, scope, assumptions, and validation gates.
  - Validation: `python -m json.tool docs/workstreams/official-metadata-addon-mdcx-style-av-scraping/WORKSTREAM.json`

- [x] OMAV-020 - Add AV number recognition and query facts
  - Extract normalized numbers from explicit AV fields and file-like title/name/path inputs.
  - Classify AV route families and expose redaction-safe query facts in metadata responses.
  - Validation: `cargo nextest run -p nako-metadata-scraper av --no-fail-fast`

- [x] OMAV-030 - Add disabled-by-default JavDB provider baseline
  - Add provider config/catalog integration and browser-worker rendered search/detail flow.
  - Emit metadata, artwork, and external IDs from synthetic rendered HTML fixtures.
  - Validation: `cargo nextest run -p nako-metadata-scraper javdb --no-fail-fast`

- [x] OMAV-040 - Add bulk AV planning summary and docs
  - Echo per-item AV facts in `bulk-metadata-scrape` output.
  - Document AV request fields, provider enablement, and batch diagnostics.
  - Validation: `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast`

- [x] OMAV-070 - Add bounded-batch duplicate reuse and failure accounting
  - Reuse duplicate AV-number scrape results within one bounded batch when items do not request metadata/artwork side effects.
  - Report per-item `reused_from_index` and `safe_failure_reason: no_candidates`.
  - Validation: `cargo nextest run -p nako-metadata-scraper bulk --no-fail-fast`

- [x] OMAV-060 - Add route-specific FC2 provider lane
  - Use shared AV facts to route `fc2` numbers to a browser-worker rendered direct article lookup provider.
  - Emit FC2 metadata, poster artwork, and `fc2`/`fc2_url`/`av_number` external IDs.
  - Validation: `cargo nextest run -p nako-metadata-scraper fc2 --no-fail-fast`

- [x] OMAV-090 - Add broader multi-provider AV routing lane
  - Let providers declare AV route support so FC2 numbers stay on the FC2 path while non-FC2 AV numbers stay on the JavDB path.
  - Preserve redaction-safe per-provider source and field provenance when shared external IDs merge provider facts.
  - Validation: `cargo nextest run -p nako-metadata-scraper engine --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper javdb --no-fail-fast`; `cargo nextest run -p nako-metadata-scraper fc2 --no-fail-fast`

- [x] OMAV-050 - Verify and close implementation
  - Run package tests, format check, JSON validation, and diff hygiene.
  - Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `rustfmt --edition 2024 --check <modified nako-metadata-scraper rust files>`; `git diff --check`

## Follow-Up Candidates

- [ ] OMAV-080 - Add resumable batch failure accounting lane
  - Add cross-batch resume state, richer failed-reason categories, and provider-level failure summaries while keeping Nako-owned scheduling.
