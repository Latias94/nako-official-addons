# Task Ledger

Prefix: AVOPS

## Active

- None.

## Pending

- None.

## Completed

- [x] AVOPS-010 [owner=codex] [deps=-] [scope=docs/workstreams/official-metadata-addon-av-ops-presets-and-drift]
  Goal: Open the durable lane and split preset/drift work into independently verifiable slices.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-ops-presets-and-drift/WORKSTREAM.json`
  Review: Ledger names concrete deliverables and keeps MDCx reference-only guardrails explicit.
  Evidence: PASS on 2026-05-26: workstream docs created with JSON ledger.
  Handoff: DONE. AVOPS-020 is complete.

- [x] AVOPS-020 [owner=codex] [deps=AVOPS-010] [scope=crates/nako-metadata-scraper/src/config.rs,crates/nako-metadata-scraper/src/manifest.rs,addons/metadata-scraper/manifest.example.json,README.md,addons/metadata-scraper/README.md]
  Goal: Add configurable AV provider presets with explicit per-provider overrides and manifest/docs support.
  Validation: `cargo nextest run -p nako-metadata-scraper av_provider_preset manifest --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`
  Review: Preset semantics live in config; registry/providers remain catalog-focused.
  Evidence: PASS on 2026-05-26: targeted preset/manifest nextest gate passed with 8 tests.
  Handoff: DONE. AVOPS-030 is complete.

- [x] AVOPS-030 [owner=codex] [deps=AVOPS-020] [scope=crates/nako-metadata-scraper/tests/live_provider_drift.rs,README.md,addons/metadata-scraper/README.md]
  Goal: Add a manual AV live drift field-health harness that reports no raw adult payload values.
  Validation: `cargo nextest run -p nako-metadata-scraper av_drift field_health --no-fail-fast`; optional ignored live command documented.
  Review: Harness uses existing provider seams and keeps CI deterministic/redaction-safe.
  Evidence: PASS on 2026-05-26: case parser, drift-only live config, and redaction-safe field-health tests passed.
  Handoff: DONE. AVOPS-040 is active.

- [x] AVOPS-040 [owner=codex] [deps=AVOPS-020,AVOPS-030] [scope=docs/workstreams/official-metadata-addon-av-ops-presets-and-drift]
  Goal: Verify, record evidence, and close or split any remaining AV operations work.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-ops-presets-and-drift/WORKSTREAM.json`; `git diff --check`
  Review: No open preset/drift decision remains only in chat or journal.
  Evidence: PASS on 2026-05-26: full package gate passed with 227 tests; fmt, JSON, and diff hygiene passed.
  Handoff: DONE. Workstream complete.

## Follow-Up Candidates

- Add Wave 4 providers after preset/drift operations are stable.
- Add scheduled drift automation when there is an external secure store for live case IDs and no adult payloads enter CI artifacts.
- Add Nako core UI affordances for preset selection once the runtime contract is stable.
