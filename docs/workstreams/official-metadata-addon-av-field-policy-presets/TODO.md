# Official Metadata Addon AV Field Policy Presets - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

- [x] OMAFP-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-av-field-policy-presets]
  Goal: Freeze problem, target state, non-goals, reference boundary, and evidence anchors.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-field-policy-presets/WORKSTREAM.json`
  Evidence: `docs/workstreams/official-metadata-addon-av-field-policy-presets/DESIGN.md`
  Handoff: Planner owns this before implementation.

## M1 - Configurable Default Policy

- [x] OMAFP-020 [owner=codex] [deps=OMAFP-010] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/providers/registry.rs]
  Goal: Add an AV field policy preset config value and construct a default provider-field policy from supported provider IDs.
  Validation: `cargo nextest run -p nako-metadata-scraper config registry field_policy --no-fail-fast`
  Review: Check that provider enablement presets and field policy presets remain separate concepts.
  Evidence: Registry/config tests.
  Handoff: DONE - shipped `default`, `quality_scores`, and `none` presets.

## M2 - Runtime Wiring And Public Contract

- [x] OMAFP-030 [owner=codex] [deps=OMAFP-020] [scope=crates/nako-metadata-scraper/src/routes.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Wire the selected default policy into runtime construction, expose it through manifest/config docs, and keep request-level overrides authoritative.
  Validation: `cargo nextest run -p nako-metadata-scraper runtime manifest routes --no-fail-fast`
  Review: Confirm docs describe precedence and supported preset names.
  Evidence: Runtime/manifest tests and README diff.
  Handoff: DONE - request-level `provider_field_policy` remains authoritative.

## M3 - Verification And Closeout

- [x] OMAFP-040 [owner=codex] [deps=OMAFP-030] [scope=docs/workstreams/official-metadata-addon-av-field-policy-presets]
  Goal: Run fresh package gates, record evidence, and close or defer follow-up work.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-field-policy-presets/WORKSTREAM.json`; `git diff --check`
  Review: No blocking findings before lane completion.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE - follow-ons recorded in `HANDOFF.md`.
