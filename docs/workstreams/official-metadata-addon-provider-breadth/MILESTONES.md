# Official Metadata Addon Provider Breadth and Localization — Milestones

Status: Complete
Last updated: 2026-05-23

## M0 — Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Relevant ADRs/docs/workstreams are linked.
- First proof target is chosen.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-breadth/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-breadth/TODO.md`

## M1 — Artwork Selection Proof

Exit criteria:

- Artwork candidate selection chooses the best poster/backdrop across providers.
- The slice is independently testable.
- Follow-up scope is recorded instead of silently widened.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper artwork --no-fail-fast`

## M2 — Alias And Localization Proof

Exit criteria:

- Provider-local alias and localized title coverage is deeper.
- The slice is independently testable.
- Provider-local semantics stay inside providers and ranking.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`

## M3 — Closeout

Exit criteria:

- Gate set is recorded.
- Remaining work is either completed, deferred, or split into a follow-on.
- `WORKSTREAM.json` status is updated.

Status: Complete on 2026-05-23. Residual provider breadth is deferred into follow-on scope rather
than silently widening this lane.
