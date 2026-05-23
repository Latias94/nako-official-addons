# Official Metadata Browser Worker - Milestones

Status: Active
Last updated: 2026-05-23

## M0 - Scope And Evidence Freeze

Status: Done on 2026-05-23.

Exit criteria:

- Problem and target state are explicit.
- Public/private boundary is explicit.
- Relevant docs are linked.
- First proof target is chosen.

Primary evidence:

- `docs/workstreams/official-metadata-browser-worker/DESIGN.md`
- `docs/workstreams/official-metadata-browser-worker/TODO.md`

## M1 - Browser Worker First Proof

Status: Done on 2026-05-23.

Exit criteria:

- The worker service exists.
- A deterministic local rendered-page extraction proof passes.
- The proof is independently testable.

Primary gates:

- worker-specific smoke command
- `npm test`
- `npm run smoke`

## M2 - Metadata Scraper Integration

Status: Done on 2026-05-23.

Exit criteria:

- The metadata scraper can call the worker through configuration.
- The compose example shows the deployment topology.
- Docs explain the integration clearly.

Primary gates:

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `docker compose -f addons/metadata-scraper/compose.example.yml config`

## M3 - Douban Browser-Backed Baseline

Status: Pending.

Exit criteria:

- One Douban-backed extraction path is proven.
- Live blocking is either solved or documented as a bounded follow-on.
- The anti-bot boundary stays outside the public addon surface.

Primary gates:

- targeted browser-worker smoke
- targeted metadata scraper tests

## M4 - Closeout

Exit criteria:

- Gate set is recorded.
- Remaining work is completed, deferred, or split.
- `WORKSTREAM.json` is updated.
