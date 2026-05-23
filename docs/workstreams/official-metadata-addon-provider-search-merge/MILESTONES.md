# Official Metadata Addon Provider Search Merge — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Policy Freeze

Exit criteria:

- Search merge target and request-budget policy are explicit.
- Non-goals are explicit.
- Relevant prior lanes are linked.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-search-merge/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-search-merge/TODO.md`

## M1 — Provider Search Merge Proof

Exit criteria:

- TMDB merges non-empty search-title variant results.
- Bangumi merges non-empty search-title variant results.
- Duplicate provider IDs are not enriched twice.
- Detail enrichment stays capped.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast`

## M2 — Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Remaining localization work is deferred or split.
- `WORKSTREAM.json` status is updated.

Status: Complete on 2026-05-24.
