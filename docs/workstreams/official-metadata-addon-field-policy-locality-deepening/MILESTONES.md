# Official Metadata Addon Field Policy Locality Deepening - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

Exit criteria:

- The default field policy locality target is explicit.
- Non-goals are explicit.
- Validation gates are recorded.

Primary evidence:

- `docs/workstreams/official-metadata-addon-field-policy-locality-deepening/DESIGN.md`
- `docs/workstreams/official-metadata-addon-field-policy-locality-deepening/TODO.md`

## M1 - Provider-Owned Default Field Preferences

Exit criteria:

- Provider modules declare default field preference descriptors.
- `ProviderRegistry` composes descriptors rather than owning provider order arrays.
- Existing default preset order remains test-equivalent.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper provider_field_policy registry default_av_provider_field_policy --no-fail-fast`
- `cargo fmt -p nako-metadata-scraper -- --check`
- `git diff --check`

## M2 - Integration And Closeout

Exit criteria:

- Full metadata-scraper package gate passes.
- `WORKSTREAM.json`, `HANDOFF.md`, and `EVIDENCE_AND_GATES.md` are current.
- No central default provider order arrays remain.

Status: complete on 2026-05-27. Targeted and package gates passed; no blocking review findings
remain.
