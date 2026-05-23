# Official Metadata Addon Bulk Task Design - Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

The design line is closed for the current official metadata addon release. The parent side-effect
writer lane completed explicit `metadata_write` and typed `artwork_write` flows, so Bulk Metadata
Scrape can eventually reuse those seams.

OMAB-010 is complete. The host assessment found that Nako currently supports
Addon Task declarations and routing plans, but not a generic Addon Task
scheduler/invoker with task progress and outcome ownership. The official Addon
manifest therefore keeps `tasks: []`.

OMAB-020, OMAB-030, and OMAB-040 are deferred outside this repository until the host runtime exists.
OMAB-050 closes this lane by documenting that boundary.

## Completed Or Deferred Tasks

- OMAB-010: completed host readiness assessment and manifest safety check.
- OMAB-020: deferred to `../nako` host runtime work.
- OMAB-030: deferred; do not declare `bulk-metadata-scrape` yet.
- OMAB-040: deferred; do not implement a hidden sidecar task endpoint.
- OMAB-050: completed current-release closeout.

## Known Constraints

- Do not add hidden background jobs in the Addon sidecar.
- Do not declare `bulk-metadata-scrape` until Nako can invoke Addon Tasks.
- Keep writes behind existing Addon Side Effects.
- Preserve redaction safety for tokens, source locators, provider payloads, and
  progress diagnostics.

## Next Likely Phase

Open a host-side workstream in `../nako` for the Addon Task scheduler/invoker. After that lands,
open a new addon implementation lane to add `bulk-metadata-scrape` to the manifest and implement the
task endpoint.
