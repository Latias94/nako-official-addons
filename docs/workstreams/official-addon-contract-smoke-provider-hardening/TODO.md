# Official Addon Contract Smoke Provider Hardening - TODO

Status: Complete
Last updated: 2026-05-24

Task IDs use the `OACSH` prefix.

## M0 - Scope Freeze

- [x] OACSH-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-addon-contract-smoke-provider-hardening]
  Goal: Record that ADR 0033 already owns protocol version/release separation,
  select follow-on tasks 2/3/4, and capture the sidecar installation
  experience constraint for future addon-family discussion.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md agree.
  Review: Ensure this lane does not reopen the closed
  `official-addons-cross-repo-fearless-refactor` workstream.
  Evidence: this workstream directory.
  Handoff: Continue with OACSH-020.

## M1 - Task Envelope Contract Unification

- [x] OACSH-020 [owner=codex] [deps=OACSH-010] [scope=crates/nako-metadata-scraper/src/engine/bulk.rs,crates/nako-metadata-scraper/src/routes.rs,../nako/crates/nako-addon-protocol]
  Goal: Remove metadata scraper local Addon Task envelope mirrors and use
  public `nako-addon-protocol` task envelope types.
  Validation: `cargo nextest run -p nako-metadata-scraper bulk task routes manifest --no-fail-fast`;
  focused `nako-addon-protocol` task envelope tests if protocol helper coverage
  changes; `cargo fmt --all -- --check`; path-scoped `git diff --check`.
  Review: Only unify the envelope. Do not move bulk result payload schemas or
  sidecar batch planning into the protocol crate.
  Evidence: EVIDENCE_AND_GATES.md command transcript. The metadata scraper now
  imports public `nako-addon-protocol` task envelope types; sidecar-owned bulk
  result schema and planner remain local.
  Handoff: Continue with OACSH-030 after route tests proved unchanged wire
  shape.

## M2 - Live Smoke Harness

- [x] OACSH-030 [owner=codex] [deps=OACSH-020] [scope=addons/metadata-scraper/smoke.local.ps1,addons/metadata-scraper/compose.example.yml,README.md,addons/metadata-scraper/README.md,../nako/scripts/official-addon-e2e-smoke.ps1]
  Goal: Make the official live smoke repeatable enough to be a release gate for
  manifest, health, metadata resource, optional protected-write gates, and
  host-dispatched `bulk-metadata-scrape` task execution.
  Validation: PowerShell parser checks; script dry-run or static validation if
  live Nako is unavailable; live smoke transcript when a local Nako server and
  admin token are available; focused Rust task/resource tests.
  Review: Keep sidecar process management explicit and local to the smoke
  harness. Do not imply Nako product supervision.
  Evidence: PowerShell parser checks, missing `-RegisterInNako` guard proof,
  E2E `-PreflightOnly`, focused Rust task/resource tests, and docs updates.
  Live Docker/server smoke remains available but was not executed in this
  session.
  Handoff: Continue with OACSH-040 after smoke behavior is documented.

## M3 - Provider Descriptor Boundary

- [x] OACSH-040 [owner=codex] [deps=OACSH-020] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs]
  Goal: Move provider id/capability/config/availability/manifest-schema
  declarations toward provider-owned descriptors while preserving
  `ProviderRegistry` as the composition point.
  Validation: `cargo nextest run -p nako-metadata-scraper provider registry manifest config --no-fail-fast`;
  `cargo fmt --all -- --check`; path-scoped `git diff --check`.
  Review: Provider descriptors should reduce central branching without hiding
  runtime policy. Do not introduce macro-heavy registration or dynamic plugin
  loading.
  Evidence: registry/manifest/config tests, full package tests, and module map
  in HANDOFF.md.
  Handoff: Continue with OACSH-050 closeout.

## M4 - Closeout

- [x] OACSH-050 [owner=planner] [deps=OACSH-020,OACSH-030,OACSH-040] [scope=docs/workstreams/official-addon-contract-smoke-provider-hardening]
  Goal: Verify evidence, record live smoke status, document residual
  installation-boundary questions, and close or split follow-ons.
  Validation: final focused gates pass or live smoke blocker is concrete and
  external.
  Review: Do not close if task envelope, live smoke, or provider descriptors
  are only journaled and not reflected in authoritative docs.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md, closeout
  journal.
  Handoff: Any official addon-family split must be a separate product/design
  lane.
