# Official Metadata Addon Bulk Task Design - Milestones

Status: Active
Last updated: 2026-05-23

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

## M2 - Manifest Declaration

Exit criteria:

- `bulk-metadata-scrape` is declared only after host execution exists;
- checked-in example manifest matches runtime manifest;
- manifest required scopes are bounded and validated.

## M3 - Task Endpoint

Exit criteria:

- task endpoint uses bounded provider fan-out;
- metadata and artwork writes go through existing Addon Side Effect APIs;
- progress/failure summaries are redaction-safe and host-owned.

## M4 - Closeout

Exit criteria:

- docs describe shipped behavior and deferred boundaries;
- fresh gates pass or skipped gates have concrete reasons;
- WORKSTREAM.json and HANDOFF.md reflect final state.
