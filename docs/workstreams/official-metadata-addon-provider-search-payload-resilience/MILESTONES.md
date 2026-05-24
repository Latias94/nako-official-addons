# Official Metadata Addon Provider Search Payload Resilience — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Salvage Policy Freeze

Exit criteria:

- Item-level search payload drift is explicitly salvageable.
- Top-level malformed search responses remain provider errors.
- Detail response tolerance remains out of scope.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-search-payload-resilience/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-search-payload-resilience/TODO.md`

## M1 — TMDB Search Payload Resilience

Exit criteria:

- TMDB skips malformed individual search result items.
- Valid results from the same response still reach detail enrichment.

Primary gate:

- `cargo nextest run -p nako-metadata-scraper tmdb ranking --no-fail-fast`

## M2 — Bangumi Search Payload Resilience

Exit criteria:

- Bangumi skips malformed individual search subject items.
- Valid subjects from the same response still reach detail enrichment.

Primary gate:

- `cargo nextest run -p nako-metadata-scraper bangumi ranking --no-fail-fast`

## M3 — Closeout

Exit criteria:

- Fresh gate evidence is recorded.
- Package, workspace, format, and whitespace gates pass.
- Live provider drift checks are deferred or split.

Status:

- Complete on 2026-05-24. The lane closed after targeted provider gates, package/workspace nextest,
  rustfmt check, and whitespace check passed. Live provider drift checks are deferred.
