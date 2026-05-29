# Official Metadata Addon Provider Live Drift Checks — Milestones

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Relevant ADRs and related workstreams are linked.
- The manual live gate is defined.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-live-drift-checks/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-live-drift-checks/TODO.md`

## M1 — Live Smoke Harness

Exit criteria:

- Ignored TMDB and Bangumi live smoke tests exist.
- The tests use public constructors and stay out of default CI.
- The environment gate is explicit and redaction-safe.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo fmt --all -- --check`
- `git diff --check`

## M2 — Manual Invocation

Exit criteria:

- The manual ignored-test command is documented.
- TMDB credential requirements are documented.
- Bangumi live execution is documented as opt-in.

Primary gates:

- `NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT=1 cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored`

## M3 — Closeout

Exit criteria:

- Gate set is recorded.
- Remaining drift checks are either complete or split into a follow-on.
- `WORKSTREAM.json` status is updated.
