# Official Resource Link Check Provider - TODO

Status: Closed
Last updated: 2026-05-28

## M0 - Workstream Freeze

- [x] ORLCP-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-resource-link-check-provider]
  Goal: Freeze first-slice scope and non-goals.
  Validation: workstream docs exist and agree.
  Evidence: DESIGN.md
  Handoff: Opened after Nako host product route was completed.

## M1 - Addon Contract And Route

- [x] ORLCP-020 [owner=Codex] [deps=ORLCP-010] [scope=crates/nako-resource-search,addons/resource-search]
  Goal: Declare and route first-class `resource_link_check`.
  Validation: cargo nextest run -p nako-resource-search resource_link_check --no-fail-fast
  Review: Route rejects non-link-check envelopes and wrong payload schema.
  Evidence: crates/nako-resource-search/src/manifest.rs, crates/nako-resource-search/src/routes.rs
  Handoff: DONE - manifest and router declare first-class `resource_link_check`.

## M2 - Checker Provider Boundary

- [x] ORLCP-030 [owner=Codex] [deps=ORLCP-020] [scope=crates/nako-resource-search/src]
  Goal: Add internal link-check request/result model and conservative checker provider.
  Validation: cargo nextest run -p nako-resource-search resource_link_check --no-fail-fast
  Review: No downloader, cloud transfer, or password persistence paths.
  Evidence: crates/nako-resource-search/src
  Handoff: DONE - conservative checker provider is separate from search providers.

## M3 - Docs And Smoke

- [x] ORLCP-040 [owner=Codex] [deps=ORLCP-030] [scope=README.md,addons/resource-search]
  Goal: Update docs and smoke script for the new resource.
  Validation: cargo nextest run -p nako-resource-search manifest --no-fail-fast
  Review: User-facing text keeps link checking separate from downloading.
  Evidence: README.md, addons/resource-search/README.md
  Handoff: DONE - docs and local smoke describe link-check without download/transfer semantics.

## M4 - Verification And Closeout

- [x] ORLCP-050 [owner=Codex] [deps=ORLCP-040] [scope=workspace]
  Goal: Verify, close workstream, and commit.
  Validation: cargo nextest run -p nako-resource-search resource_link_check --no-fail-fast; cargo nextest run -p nako-resource-search manifest --no-fail-fast; cargo fmt --all -- --check; cargo check -p nako-resource-search --tests; git diff --check
  Review: Record residual risks and deferred site-specific providers.
  Evidence: EVIDENCE_AND_GATES.md, CLOSEOUT.md
  Handoff: DONE - focused nextest, fmt, check, and diff gates passed.
