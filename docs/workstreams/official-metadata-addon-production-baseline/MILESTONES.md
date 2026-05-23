# Official Metadata Addon Production Baseline — Milestones

Status: Complete
Last updated: 2026-05-23

## M0 — Scope And Baseline

Exit criteria:

- Workstream docs exist and agree.
- Task order prioritizes smoke, ranking/evidence, then TMDB baseline.
- Follow-on split rules are explicit.

## M1 — Live Nako Admin-Mediated Smoke

Exit criteria:

- Direct sidecar smoke remains runnable.
- Nako Admin-mediated smoke is either run and recorded, or blocked by a named
  external condition such as missing server/admin token.
- Script does not own Nako process lifecycle or print secrets.

## M2 — Provider-Neutral Ranking And Evidence

Exit criteria:

- Runtime owns final candidate confidence.
- Evidence has provider-neutral match reasons.
- Sorting is deterministic and tested.

## M3 — TMDB Production Baseline

Exit criteria:

- TMDB search candidates are enriched with bounded detail/external-ID calls.
- Supported metadata patch fields are filled from normalized TMDB facts.
- Richer provider facts become safe artifacts, not direct writes.
- Tests use synthetic fake HTTP responses only.

## M4 — Docs And Operator Truth

Exit criteria:

- README/addon README/examples match runtime behavior.
- Docs distinguish current TMDB baseline from future provider breadth.
- Live smoke caveats are explicit.

## M5 — Closeout Or Follow-On Split

Exit criteria:

- Final gates pass.
- Remaining provider/product breadth is split into named follow-ons.
- `WORKSTREAM.json`, `HANDOFF.md`, and evidence ledger reflect final state.

Closeout result:

- Final gates passed on 2026-05-23.
- Live Nako Admin-mediated smoke is split as an external-environment follow-on.
- Provider breadth beyond TMDB remains split into future lanes.
