# Official Metadata Addon Provider Live Drift Checks — TODO

Status: Complete
Last updated: 2026-05-24

## M0 — Scope And Evidence Freeze

- [x] OMLDC-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-provider-live-drift-checks]
  Goal: Freeze the live drift check problem statement, opt-in boundary, and evidence anchors.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: docs/workstreams/official-metadata-addon-provider-live-drift-checks/DESIGN.md
  Handoff: Scope is fixed around ignored, env-gated live smoke checks.

## M1 — Live Smoke Harness

- [x] OMLDC-020 [owner=agent] [deps=OMLDC-010] [scope=crates/nako-metadata-scraper/tests/live_provider_drift.rs]
  Goal: Add ignored TMDB and Bangumi live smoke checks behind an explicit environment gate.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt --all -- --check`; `git diff --check`
  Review: review-workstream before accepting lane completion.
  Evidence: crates/nako-metadata-scraper/tests/live_provider_drift.rs
  Handoff: Implemented as a crate test, not a runtime behavior change.

## M2 — Manual Live Invocation

- [x] OMLDC-030 [owner=agent] [deps=OMLDC-020] [scope=docs,tests]
  Goal: Document the manual ignored-test invocation and required environment variables for TMDB and Bangumi.
  Validation: `NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT=1 cargo test -p nako-metadata-scraper --test live_provider_drift -- --ignored`
  Review: review-workstream for completeness and residual risk.
  Evidence: EVIDENCE_AND_GATES.md, HANDOFF.md
  Handoff: Manual invocation is documented and the live gate passed in this workspace.

## M3 — Closeout

- [x] OMLDC-040 [owner=planner] [deps=OMLDC-030] [scope=docs/workstreams/official-metadata-addon-provider-live-drift-checks]
  Goal: Close the lane or split any provider-specific follow-on.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: Summarize any remaining drift-risk follow-ons in HANDOFF.md.
