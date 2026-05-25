# Official Metadata Addon Bangumi Metadata Enrichment — Milestones

Status: Complete
Last updated: 2026-05-26

## M0 — Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Relevant docs/workstreams are linked.
- Reference source and license boundaries are recorded.

Primary evidence:

- `docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment/DESIGN.md`
- `docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment/TODO.md`

## M1 — Subject Fact Enrichment

Exit criteria:

- Bangumi parser tolerates the official optional enrichment fields.
- Mapper emits deterministic provider-prefixed tags for new subject facts.
- Infobox-derived facts are trimmed, deduplicated, and bounded.
- Existing Bangumi direct lookup, search, degraded fallback, and artwork tests
  remain valid.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast`

## M2 — Reference Findings And Docs

Exit criteria:

- Workstream docs record official schema facts and GPL reference boundaries.
- User-facing docs mention visible Bangumi enrichment behavior when needed.
- Evidence anchors are updated.

Primary gates:

- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast`

## M3 — Closeout

Exit criteria:

- Package-level targeted tests and format checks pass.
- Review has no blocking findings.
- `WORKSTREAM.json` status is complete or follow-on is split.

Status: complete. Package tests, targeted Bangumi tests, formatting, JSON, and
diff hygiene gates passed on 2026-05-26.
