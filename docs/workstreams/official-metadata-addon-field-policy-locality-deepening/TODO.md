# Official Metadata Addon Field Policy Locality Deepening - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

- [x] OMAFPL-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-metadata-addon-field-policy-locality-deepening]
  Goal: Freeze the field-policy locality target and separate it from execution/render locality.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json, and HANDOFF.md exist and agree.
  Review: Confirm the lane is limited to default AV field provider preferences.
  Evidence: docs/workstreams/official-metadata-addon-field-policy-locality-deepening/DESIGN.md
  Handoff: DONE. Workstream opened after closing the execution-locality lane.

## M1 - Provider-Owned Default Field Preferences

- [x] OMAFPL-020 [owner=codex] [deps=OMAFPL-010] [scope=crates/nako-metadata-scraper/src/providers,crates/nako-metadata-scraper/src/engine/query.rs]
  Goal: Replace central default field provider order arrays with provider-owned preference descriptors composed by ProviderRegistry.
  Validation: cargo nextest run -p nako-metadata-scraper provider_field_policy registry default_av_provider_field_policy --no-fail-fast
  Review: Confirm `ProviderRegistry` no longer owns provider order facts and default preset behavior is preserved.
  Evidence: crates/nako-metadata-scraper/src/providers/registry.rs; provider catalog entries.
  Handoff: DONE. Provider-owned `ProviderDefaultFieldPreference` descriptors now declare default field order facts; `ProviderRegistry` folds catalog descriptors into `ProviderFieldPolicy`. The old central `DEFAULT_*_PROVIDER_ORDER` and `DEFAULT_FIELD_PROVIDER_PREFERENCES` arrays were removed.

## M2 - Integration And Closeout

- [x] OMAFPL-030 [owner=codex] [deps=OMAFPL-020] [scope=crates/nako-metadata-scraper,docs/workstreams/official-metadata-addon-field-policy-locality-deepening]
  Goal: Run package gates, update evidence, and close the lane.
  Validation: cargo nextest run -p nako-metadata-scraper --no-fail-fast
  Review: review-workstream before accepting completion.
  Evidence: EVIDENCE_AND_GATES.md; WORKSTREAM.json; HANDOFF.md
  Handoff: DONE. Package, format, JSON, and diff hygiene gates passed. Closeout review found no blocking issues and confirmed central default provider order arrays were removed.
