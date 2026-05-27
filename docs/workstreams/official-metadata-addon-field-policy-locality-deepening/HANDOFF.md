# Official Metadata Addon Field Policy Locality Deepening - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

The lane is opened after the execution-locality lane closed. The next target is the central default
AV field provider preference table in `ProviderRegistry`.

Completed:

- OMAFPL-010 froze the scope and gates.
- OMAFPL-020 moved default AV field provider order facts into provider-owned
  `ProviderDefaultFieldPreference` descriptors and removed the central provider order arrays.
- OMAFPL-030 verified and closed the lane.

Remaining:

- None for this lane.

## Closed Task

- Task ID: OMAFPL-030
- Owner: codex
- Files: `crates/nako-metadata-scraper`, `docs/workstreams/official-metadata-addon-field-policy-locality-deepening`
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- Status: DONE
- Review: No blocking findings. Registry is now a composition layer for default field preferences.
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Preserve request-visible `provider_field_policy` parsing.
- Preserve the quality-score preset.
- Keep field alias groups as central metadata vocabulary helpers.
- Move provider order facts into provider-owned catalog descriptors.

## Blockers

- None.

## Next Recommended Action

- Commit this lane after user confirmation.
