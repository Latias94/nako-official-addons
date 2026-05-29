# Official Metadata Browser Worker - Handoff

Status: Closed
Last updated: 2026-05-23

## Current State

The dedicated browser worker exists under `addons/browser-worker` and now
exposes a stable generic `POST /render` contract. `nako-metadata-scraper` can
be configured to call the worker, and the Rust `douban` provider parses
fixture-backed Douban search/detail HTML rendered through that contract. The
public metadata addon remains the only Nako-facing addon surface;
Playwright/Crawlee stay outside the Rust sidecar.

## Closeout

- Closed task: OMBW-050
- Status: CLOSED on 2026-05-23
- Evidence: `docs/workstreams/official-metadata-browser-worker/EVIDENCE_AND_GATES.md`
- Shipped proof: deterministic browser rendering, sidecar worker integration,
  Compose topology, and fixture-backed Douban search/detail parsing through
  `POST /render`.

## Decisions Since Last Update

- The browser automation lane is separate from the existing side-effect writer lane.
- The browser worker should be an internal companion service, not a public addon requirement.
- Docker Compose is the expected local/self-hosted deployment mechanism.
- The first proof uses a deterministic local rendered-page fixture before adding Douban-specific behavior.
- The Rust sidecar uses an HTTP provider adapter and does not depend on Playwright or Crawlee.
- `POST /render` is the worker contract; Douban parsing and provider mapping stay in Rust.
- OMBW-040 is accepted as fixture-backed, not live-network-backed.

## Blockers

- None for the deterministic worker proof, sidecar integration, and fixture-backed Douban provider.
- Live Douban smoke remains unproven and may require proxy, headers, cookies, or rate-limit policy.

## Follow-On

- Open a focused follow-on for live Douban smoke hardening if the product wants
  to claim live-network support. That follow-on should define proxy/cookie
  policy, rate-limit behavior, selector breadth, and what counts as acceptable
  external-site evidence.
