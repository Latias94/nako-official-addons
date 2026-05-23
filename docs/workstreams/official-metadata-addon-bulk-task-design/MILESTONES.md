# Official Metadata Addon Bulk Task Design - Milestones

Status: Complete
Last updated: 2026-05-24

## M0 - Design Line Opened

Exit criteria:

- host Addon Task readiness is assessed;
- official addon manifest remains task-free until execution exists;
- first executable follow-on is named.

## M1 - Host Runtime Contract

Exit criteria:

- Nako owns Addon Task invocation, job records, retry, cancellation, and
  diagnostics;
- request and response envelopes are documented or typed;
- no sidecar-local scheduler is required.

Result:

- Deferred outside this repository. Nako host runtime ownership is required before addon work
  continues.

## M2 - Manifest Declaration

Exit criteria:

- `bulk-metadata-scrape` is declared only after host execution exists;
- checked-in example manifest matches runtime manifest;
- manifest required scopes are bounded and validated.

Result:

- Deferred. The current release intentionally keeps `tasks: []`.

## M3 - Task Endpoint

Exit criteria:

- task endpoint uses bounded provider fan-out;
- metadata and artwork writes go through existing Addon Side Effect APIs;
- progress/failure summaries are redaction-safe and host-owned.

Result:

- Deferred. No addon-side hidden scheduler or task endpoint is shipped.

## M4 - Closeout

Exit criteria:

- docs describe shipped behavior and deferred boundaries;
- fresh gates pass or skipped gates have concrete reasons;
- WORKSTREAM.json and HANDOFF.md reflect final state.

Result:

- Complete on 2026-05-24 for the current release. Future work starts in `../nako` with the host
  Addon Task runtime.
