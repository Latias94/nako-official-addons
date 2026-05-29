# Official Metadata Addon Execution Locality Deepening - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

Exit criteria:

- The three residual architecture targets are explicit.
- Non-goals are explicit.
- Related closed workstreams are linked.
- First executable task is selected.

Primary evidence:

- `docs/workstreams/official-metadata-addon-execution-locality-deepening/DESIGN.md`
- `docs/workstreams/official-metadata-addon-execution-locality-deepening/TODO.md`

## M1 - Bulk Provider Execution Typed Facts

Exit criteria:

- Bulk Metadata Scrape no longer mutates JSON payload to carry provider execution policy.
- Runtime exposes an internal typed path suitable for Bulk.
- Existing request-visible provider execution policy behavior is either preserved intentionally or removed with tests proving the new behavior.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper bulk provider_execution --no-fail-fast`
- `cargo fmt -p nako-metadata-scraper -- --check`
- `git diff --check`

## M2 - Provider Catalog Residual Locality

Exit criteria:

- Provider-specific catalog/config facts are moved closer to provider Modules where feasible.
- Central registry/config code acts more like composition/query logic.
- Tests prove provider config and registry behavior.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper provider registry config --no-fail-fast`
- `cargo fmt -p nako-metadata-scraper -- --check`
- `git diff --check`

## M3 - Render Drift Case Locality

Exit criteria:

- Provider-specific drift sample/case facts are provider-owned where feasible.
- Render drift runner owns lookup/filter/serialization only.
- Render drift case output remains covered by tests.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper render_drift --no-fail-fast`
- `cargo fmt -p nako-metadata-scraper -- --check`
- `git diff --check`

## M4 - Integration And Closeout

Exit criteria:

- Full metadata-scraper package gate passes.
- Remaining work is completed, explicitly deferred, or split into a follow-on.
- `WORKSTREAM.json`, `HANDOFF.md`, and `EVIDENCE_AND_GATES.md` are current.

Status: complete on 2026-05-27. Package, format, JSON, and diff hygiene gates passed. No blocking
review findings remain for the lane target.
