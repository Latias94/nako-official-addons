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

## Active

- [ ] OMAV-050 - Verify and close implementation
  - Run package tests, format check, JSON validation, and diff hygiene.
  - Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`

## Follow-Up Candidates

- [ ] OMAV-060 - Add multi-provider AV routing lane
  - Use the shared AV facts to route FC2, censored, uncensored, amateur, western, and domestic families to provider groups.
  - Keep provider failures isolated and preserve per-provider field provenance.

- [ ] OMAV-070 - Add resumable batch failure accounting lane
  - Add richer failed-reason summaries and duplicate-number coalescing to bulk task output while keeping Nako-owned scheduling.
