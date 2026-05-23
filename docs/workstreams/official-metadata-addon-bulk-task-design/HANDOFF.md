# Official Metadata Addon Bulk Task Design - Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The design line is open. The parent side-effect writer lane completed explicit
`metadata_write` and typed `artwork_write` flows, so Bulk Metadata Scrape can
eventually reuse those seams.

OMAB-010 is complete. The host assessment found that Nako currently supports
Addon Task declarations and routing plans, but not a generic Addon Task
scheduler/invoker with task progress and outcome ownership. The official Addon
manifest therefore keeps `tasks: []`.

## Next Task

Start OMAB-020.

Goal: define or implement the Nako-owned Addon Task runtime contract before
the official Addon declares `bulk-metadata-scrape`.

## Known Constraints

- Do not add hidden background jobs in the Addon sidecar.
- Do not declare `bulk-metadata-scrape` until Nako can invoke Addon Tasks.
- Keep writes behind existing Addon Side Effects.
- Preserve redaction safety for tokens, source locators, provider payloads, and
  progress diagnostics.

## Next Likely Phase

If the host contract lands in `../nako`, continue with OMAB-030 to add the
manifest task declaration and parity tests. If not, keep this lane active as a
design handoff rather than adding dead manifest surface.
