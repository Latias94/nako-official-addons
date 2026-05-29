# Official Metadata Addon Host Policy Adapter - TODO

Status: Complete
Last updated: 2026-05-26

## Tasks

- [x] OMAHPA-010 - Audit metadata scraper writeback path.
- [x] OMAHPA-020 - Audit native metadata patch materialization path.
- [x] OMAHPA-030 - Add tests rejecting host policy fields in sidecar writeback
  requests.
- [x] OMAHPA-040 - Record boundary decision and verification.

## Decision

No structural refactor is needed now. The sidecar remains a facts adapter; Nako
host owns application policy.
