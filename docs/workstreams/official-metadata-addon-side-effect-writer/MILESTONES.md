# Official Metadata Addon Side Effect Writer - Milestones

Status: Complete
Last updated: 2026-05-23

## M0 - Lane Opened

Exit criteria:

- workstream docs exist and agree;
- the 1/2/3 plan is split into side-effect writer, artwork flow, and Bulk
  Metadata Scrape evaluation;
- first executable task is OMASE-020.

## M1 - Runtime Client

Exit criteria:

- Nako runtime config is disabled by default;
- Addon Token handling is redaction-safe;
- fake transport tests prove access-check and side-effect request shapes.

## M2 - Metadata Write

Exit criteria:

- ordinary metadata suggestions remain non-mutating;
- explicit payload can submit a selected candidate as `metadata_write`;
- skipped/failure outcomes are safe to serialize.

## M3 - Artwork Candidate

Exit criteria:

- providers expose typed artwork facts;
- response payloads include typed artwork candidates;
- explicit payload can submit `artwork_write` through the runtime client.

## M4 - Bulk Scrape Evaluated

Exit criteria:

- Addon Task host readiness is recorded;
- manifest changes are made only if the host seam is ready;
- otherwise a focused follow-on is split.

## M5 - Closeout

Exit criteria:

- docs describe the shipped behavior and safety rules;
- fresh gates pass or skipped gates have concrete reasons;
- WORKSTREAM.json and HANDOFF.md reflect the final state.

Outcome: Complete 2026-05-23. The side-effect writer lane is closed, and the
Bulk Metadata Scrape follow-on is split into
`official-metadata-addon-bulk-task-design`.
