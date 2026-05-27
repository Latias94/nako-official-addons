# Official Metadata Addon AV Fine-Grained Field Policy - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Scope

- [x] OMAFG-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-av-fine-grained-field-policy]
  Goal: Freeze the follow-up scope for field-by-field AV source priority and score fusion.
  Validation: `python -m json.tool docs/workstreams/official-metadata-addon-av-fine-grained-field-policy/WORKSTREAM.json`
  Evidence: `DESIGN.md`
  Handoff: Scope opened from the active user goal.

## M1 - Field Matrix

- [x] OMAFG-020 [owner=codex] [deps=OMAFG-010] [scope=crates/nako-metadata-scraper/src/providers/registry.rs,crates/nako-metadata-scraper/src/engine/query.rs]
  Goal: Replace coarse default AV field groups with explicit supported-provider orders adapted from the reference config.
  Validation: `cargo nextest run -p nako-metadata-scraper registry_builds_default_av_field_policy registry_builds_quality_score_av_field_policy --no-fail-fast`
  Review: Unsupported reference providers must not appear in default policy output.
  Evidence: Registry/query tests.
  Handoff: DONE - default field matrix now covers aliases and score/vote fields.

## M2 - Score Fusion

- [x] OMAFG-030 [owner=codex] [deps=OMAFG-020] [scope=crates/nako-metadata-scraper/src/engine/fusion.rs,crates/nako-metadata-scraper/src/engine/mod.rs]
  Goal: Fuse community score and vote-count facts through provider-field policy, including `score` alias support.
  Validation: `cargo nextest run -p nako-metadata-scraper provider_field_policy --no-fail-fast`
  Review: Field-source evidence stays redaction-safe and request override remains authoritative.
  Evidence: Runtime fusion tests.
  Handoff: DONE - score/vote source evidence is emitted for fused clusters.

## M3 - Docs And Gates

- [x] OMAFG-040 [owner=codex] [deps=OMAFG-030] [scope=addons/metadata-scraper/README.md,crates/nako-metadata-scraper/README.md,docs/workstreams/official-metadata-addon-av-fine-grained-field-policy]
  Goal: Document supported field names, record evidence, run full gates, and close the lane.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `cargo fmt -p nako-metadata-scraper -- --check`; `python -m json.tool docs/workstreams/official-metadata-addon-av-fine-grained-field-policy/WORKSTREAM.json`; `git diff --check`
  Review: Closeout docs reflect any remaining provider breadth follow-up separately.
  Evidence: `EVIDENCE_AND_GATES.md`
  Handoff: DONE - gates passed and closeout recorded.
