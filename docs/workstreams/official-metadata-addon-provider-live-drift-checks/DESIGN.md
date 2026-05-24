# Official Metadata Addon Provider Live Drift Checks

## Why This Lane Exists

TMDB and Bangumi fixture-backed tests prove our parsers and synthetic behavior, but they do not
prove the live public APIs still match the shapes our providers expect. This lane adds opt-in live
smoke checks that can be run manually when network access and provider credentials are available.

## Relevant Authority

- ADRs:
  - `docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-provider-search-payload-resilience`
  - `docs/workstreams/official-metadata-addon-provider-partial-search-diagnostics`
  - `docs/workstreams/official-metadata-addon-provider-external-id-lookup`
  - `docs/workstreams/official-metadata-addon-provider-relevance-budget`

## Problem

Synthetic tests can pass while a live provider response silently drifts. That leaves operators with
no cheap manual check for whether TMDB or Bangumi still parse against the real API.

## Target State

- `crates/nako-metadata-scraper/tests/live_provider_drift.rs` contains ignored TMDB and Bangumi
  smoke checks.
- The checks are opt-in through an explicit environment variable.
- TMDB uses live credentials when available; Bangumi uses the public live API path.
- Default CI gates remain synthetic.

## In Scope

- Add ignored integration tests for live TMDB and Bangumi smoke checks.
- Keep the checks grounded in public constructors and provider APIs.
- Document the manual invocation path and required environment variables.

## Out Of Scope

- Adding live provider checks to default CI.
- Changing runtime retry/backoff behavior.
- Introducing a new monitoring service or long-running drift daemon.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Public provider constructors are enough for live smoke checks. | High | Current provider modules expose `new` and `suggest`. | The lane would need a helper or test-only harness. |
| Manual live execution is acceptable outside default gates. | High | Existing workstreams defer live drift checks to follow-on scope. | The lane would need to be split or reclassified. |
| TMDB live requests need an operator-supplied read token. | High | Current provider config makes the token optional but the API needs it for live access. | TMDB smoke checks would need a different auth path. |

## Architecture Direction

Keep live checks in `crates/nako-metadata-scraper/tests/` so they reuse the public crate surface and
stay out of runtime behavior. Gate execution behind an explicit opt-in environment variable and keep
the tests ignored by default so synthetic CI remains stable.

## Closeout Condition

This lane can close when:

- the ignored live smoke tests compile,
- the manual invocation path is documented,
- default gates stay green,
- and any remaining provider-specific live work is explicitly split or deferred.
