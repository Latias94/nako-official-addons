# Official Resource Search First-Class Protocol - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] ORSFP-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-resource-search-first-class-protocol]
  Goal: Freeze problem, target state, non-goals, and evidence anchors.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json exist and agree.
  Evidence: docs/workstreams/official-resource-search-first-class-protocol/DESIGN.md
  Handoff: Planner opened this lane on 2026-05-28.

## M1 - First-Class Protocol Slice

- [x] ORSFP-020 [owner=Codex] [deps=ORSFP-010] [scope=crates/nako-resource-search,addons/resource-search]
  Goal: Replace temporary automation alpha resource declarations and route envelopes with first-class `resource_search` and `acquisition_search_read`.
  Validation: cargo nextest run -p nako-resource-search resource_search --no-fail-fast
  Review: Check that addon-local alpha schemas are deleted or no longer part of the external contract.
  Evidence: crates/nako-resource-search/src/routes/resource_protocol.rs
  Handoff: DONE. Manifest and routes now use first-class protocol DTOs and scope.

## M2 - Docs And Follow-On Boundaries

- [x] ORSFP-030 [owner=Codex] [deps=ORSFP-020] [scope=docs,README.md,addons/resource-search]
  Goal: Update user-facing docs and record separate future contracts for link checking, downloader/external runner, cloud-drive transfer, and password/code references.
  Validation: rg -n "automation alpha|alpha request|automation_run|future_protocol_resource" crates/nako-resource-search addons/resource-search README.md
  Review: Confirm Admin UI is not included.
  Evidence: docs/workstreams/official-resource-search-first-class-protocol/FOLLOW_ON_CONTRACTS.md
  Handoff: DONE. Follow-on boundaries are documented here and in Nako ADR 0050.

## M3 - Verification And Closeout

- [x] ORSFP-040 [owner=Codex] [deps=ORSFP-030] [scope=workspace]
  Goal: Verify and close the lane with fresh evidence.
  Validation: cargo nextest run -p nako-resource-search --no-fail-fast; cargo fmt --all -- --check; cargo check -p nako-resource-search --tests
  Review: Record residual risks and commit only this lane's changes.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json
  Handoff: DONE. Closeout records gates, review, and residual follow-ons.
