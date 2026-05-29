# Official Metadata Addon Provider Live Drift Checks — Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The crate now has an ignored, env-gated integration test file for TMDB and Bangumi live smoke
checks. The manual live invocation path is documented, default CI stays synthetic, and the manual
live gate passed in this workspace.

## Active Task

- Task ID: OMLDC-040
- Owner: agent
- Files: `docs/workstreams/official-metadata-addon-provider-live-drift-checks/EVIDENCE_AND_GATES.md`, `docs/workstreams/official-metadata-addon-provider-live-drift-checks/HANDOFF.md`, `docs/workstreams/official-metadata-addon-provider-live-drift-checks/WORKSTREAM.json`
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT=1 cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored`
- Status: DONE
- Review: no blocking findings
- Evidence: `crates/nako-metadata-scraper/tests/live_provider_drift.rs`

## Decisions Since Last Update

- Live drift checks live in `tests/` so they reuse the public crate surface.
- The checks are ignored by default and require an explicit environment gate.
- TMDB live checks are token-backed; Bangumi live checks stay public and opt-in.

## Blockers

- None.

## Next Recommended Action

- None. The lane is closed; split follow-on work only if broader live provider monitoring or
  provider-specific drift coverage is prioritized later.
