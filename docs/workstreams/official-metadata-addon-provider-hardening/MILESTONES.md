# Official Metadata Addon Provider Hardening — Milestones

Status: Complete
Last updated: 2026-05-23

## M0 — Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Relevant ADRs/docs/workstreams are linked.
- First proof target is chosen.

Primary evidence:

- `docs/workstreams/official-metadata-addon-provider-hardening/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-hardening/TODO.md`

## M1 — Network Policy Proof

Exit criteria:

- A shared provider network policy is surfaced through config and diagnostics.
- Proxy-aware behavior is testable through the provider HTTP runtime seam.
- Failure modes are redaction-safe and actionable.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper provider_http_runtime config routes --no-fail-fast`

## M2 — Provider Quality Proof

Exit criteria:

- TMDB and Bangumi candidate shaping is measurably deeper.
- Candidate ranking and image selection are still provider-local.
- No browser automation is embedded into the Rust sidecar.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking --no-fail-fast`

## M3 — Closeout

Exit criteria:

- Gate set is recorded.
- Remaining work is either completed, deferred, or split into a follow-on.
- `WORKSTREAM.json` status is updated.

Status: Met on 2026-05-23. The lane is closed and follow-on breadth work is deferred to a new lane.
