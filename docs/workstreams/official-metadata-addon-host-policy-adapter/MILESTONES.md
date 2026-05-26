# Official Metadata Addon Host Policy Adapter - Milestones

Status: Complete
Last updated: 2026-05-26

## OMAHPA-M1 - Boundary Audited

Reviewed:

- `crates/nako-metadata-scraper/src/engine/writeback.rs`
- `crates/nako-metadata-scraper/src/engine/side_effect.rs`
- `crates/nako-metadata-scraper/src/engine/native_writeback.rs`

## OMAHPA-M2 - Tests Added

Metadata writeback now has direct tests for:

- explicit writeback request parsing;
- rejection of host-policy-looking fields such as `refresh_mode`;
- target validation as media-source-only.

## OMAHPA-M3 - Lane Closed

No host policy was moved into the sidecar.
