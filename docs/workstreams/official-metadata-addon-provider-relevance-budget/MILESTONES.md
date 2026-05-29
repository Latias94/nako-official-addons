# Official Metadata Addon Provider Relevance Budget — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Budget Policy Freeze

Exit criteria:

- The enrichment-budget problem is explicit.
- Non-goals are explicit.
- Existing search merge and degraded-candidate behavior remain authoritative.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-relevance-budget/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-relevance-budget/TODO.md`

## M1 — TMDB Relevance-Budget Proof

Exit criteria:

- TMDB can collect more deduped search results than the enrichment budget.
- A stronger normalized-title result can displace earlier weak raw-title results before enrichment.
- Search-level failures still propagate.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb ranking title --no-fail-fast`

## M2 — Bangumi Relevance-Budget Proof

Exit criteria:

- Bangumi can collect more deduped search results than the enrichment budget.
- A stronger normalized-title result can displace earlier weak raw-title results before enrichment.
- Search-level failures still propagate.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast`

## M3 — Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Package and hygiene gates pass.
- Remaining relevance or live-provider work is deferred or split.

## Closeout Summary

Closed on 2026-05-24.

- TMDB and Bangumi now rank deduped merged search results before spending the three-candidate detail
  enrichment budget.
- The budget selection reuses provider-neutral candidate facts and ranking behavior.
- Final runtime ranking, payload shape, and HTTP retry policy were not changed.
- Live provider payload drift checks remain follow-on scope.
