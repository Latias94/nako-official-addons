# Official Metadata Addon AV Jav321 Provider - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Contract

Exit criteria:

- Reference-only boundary is explicit.
- Expected Jav321 fields are explicit.
- Runtime boundary is chosen.

## M1 - Runtime And Provider Proof

Exit criteria:

- HTTP runtime can submit bounded form requests and return bounded text.
- Jav321 parser tests cover the field contract.
- Provider returns a normalized AV candidate from synthetic HTML.

Primary gate:

- `cargo nextest run -p nako-metadata-scraper jav321 http_runtime --no-fail-fast`

## M2 - Registry, Config, Policy, Docs

Exit criteria:

- Jav321 is in provider catalog and config.
- Manifest and README expose the new provider.
- Default field policy can use Jav321 for title/overview/score-like facts.

Primary gate:

- `cargo nextest run -p nako-metadata-scraper config registry manifest jav321 --no-fail-fast`

## M3 - Verification And Closeout

Exit criteria:

- Package, fmt, JSON, and diff gates pass.
- Workstream status is complete or follow-ups are split.

Result:

- Complete. Synthetic and live Jav321 checks passed; remaining optional live-field gaps are page-data dependent, not implementation blockers.
