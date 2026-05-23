# Official Metadata Addon Provider Degraded Candidates — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Degraded Candidate Policy Freeze

Exit criteria:

- Degraded candidate behavior is explicit.
- Search-level failure behavior remains explicit.
- Non-goals are explicit.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-degraded-candidates/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-degraded-candidates/TODO.md`

## M1 — Provider Degraded Candidate Proof

Exit criteria:

- TMDB returns a degraded search-result candidate when detail enrichment fails.
- Bangumi returns a degraded search-result candidate when detail enrichment fails.
- Degraded candidates carry a provider note.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`

## M2 — Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Remaining error-reporting work is deferred or split.
- `WORKSTREAM.json` status is updated.

Result:

- Complete on 2026-05-24. Remaining live-provider and UI warning semantics are deferred follow-ons.
