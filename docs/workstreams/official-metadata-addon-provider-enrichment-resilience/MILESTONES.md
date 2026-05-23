# Official Metadata Addon Provider Enrichment Resilience — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Failure Policy Freeze

Exit criteria:

- Candidate-level failure behavior is explicit.
- Search-level failure behavior remains explicit.
- Non-goals are explicit.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-enrichment-resilience/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-enrichment-resilience/TODO.md`

## M1 — Candidate Enrichment Failure Isolation

Exit criteria:

- TMDB isolates failed candidate enrichment and returns other usable candidates.
- Bangumi isolates failed candidate enrichment and returns other usable candidates.
- Provider `suggest` still fails when search itself fails.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi --no-fail-fast`

## M2 — Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Remaining resilience work is deferred or split.
- `WORKSTREAM.json` status is updated.

Status: Complete on 2026-05-24.
