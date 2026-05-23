# Official Metadata Browser Worker - Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The dedicated browser worker exists under `addons/browser-worker` and now
exposes a stable generic `POST /render` contract. `nako-metadata-scraper` can
be configured to call the worker, and the Rust `douban` provider parses
fixture-backed Douban search/detail HTML rendered through that contract. The
public metadata addon remains the only Nako-facing addon surface;
Playwright/Crawlee stay outside the Rust sidecar.

## Active Task

- Task ID: OMBW-050
- Owner: planner
- Files: `README.md`, `addons/metadata-scraper`, `docs/workstreams/official-metadata-browser-worker`
- Validation: closeout gate set from `EVIDENCE_AND_GATES.md`; live Douban risk must remain explicit if not separately proven.
- Status: READY
- Review: Pending
- Evidence: `docs/workstreams/official-metadata-browser-worker/EVIDENCE_AND_GATES.md`

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

## Next Recommended Action

- Run review/verify closeout for OMBW-050, then either close this workstream with live Douban as residual risk or split a follow-on for live Douban smoke hardening.
