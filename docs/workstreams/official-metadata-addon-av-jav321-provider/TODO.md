# Official Metadata Addon AV Jav321 Provider - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Contract

- [x] OMJ321-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-av-jav321-provider]
  Goal: Freeze Jav321 field contract, runtime boundary, and reference-only guardrail.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-jav321-provider/WORKSTREAM.json`
  Evidence: `DESIGN.md`
  Handoff: Planner owns this before implementation.

## M1 - Runtime And Provider Proof

- [x] OMJ321-020 [owner=codex] [deps=OMJ321-010] [scope=crates/nako-metadata-scraper/src/providers/http_runtime.rs,crates/nako-metadata-scraper/src/providers/jav321.rs]
  Goal: Add bounded form/text support and a Jav321 parser/provider proof with synthetic detail HTML.
  Validation: `cargo nextest run -p nako-metadata-scraper jav321 http_runtime --no-fail-fast`
  Review: Confirm tests pin all expected fields before broad wiring.
  Evidence: Provider and runtime tests pin form POST, raw text response, not-found handling, direct URL lookup, and the Jav321 field contract.
  Handoff: DONE - provider proof uses raw HTTP runtime instead of browser-worker render flow.

## M2 - Registry, Config, Policy, Docs

- [x] OMJ321-030 [owner=codex] [deps=OMJ321-020] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/providers/mod.rs,crates/nako-metadata-scraper/src/providers/registry.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md]
  Goal: Wire Jav321 into provider IDs, catalog, external ID aliases, default presets, field policy, manifest, and docs.
  Validation: `cargo nextest run -p nako-metadata-scraper config registry manifest jav321 --no-fail-fast`
  Review: Confirm default enablement remains disabled unless preset says otherwise.
  Evidence: Registry/config/manifest/route tests and README/example-manifest updates.
  Handoff: DONE - `community_first` enables Jav321; manual default remains disabled.

## M3 - Verification And Closeout

- [x] OMJ321-040 [owner=codex] [deps=OMJ321-030] [scope=docs/workstreams/official-metadata-addon-av-jav321-provider]
  Goal: Run fresh package gates, record evidence, and close or split follow-ups.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-jav321-provider/WORKSTREAM.json`; `git diff --check`
  Review: No blocking findings before completion.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE - live Jav321 drift smoke also passed through `http://127.0.0.1:10809`.
