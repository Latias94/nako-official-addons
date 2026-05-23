# Official Metadata Browser Worker - Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The dedicated browser worker exists under `addons/browser-worker` and has a
deterministic rendered-page proof. `nako-metadata-scraper` can be configured to
call it through the `browser_worker` provider, and the Compose example wires the
two services together. The public metadata addon remains the only Nako-facing
addon surface; Playwright/Crawlee stay outside the Rust sidecar.

## Active Task

- Task ID: OMBW-040
- Owner: unassigned
- Files: `addons/browser-worker`, `crates/nako-metadata-scraper/src/providers`, `addons/metadata-scraper/smoke.local.ps1`
- Validation: targeted browser-worker smoke and metadata-scraper tests; live Douban proof may be gated or replaced with a recorded fixture if external blocking persists.
- Status: READY
- Review: Pending
- Evidence: `docs/workstreams/official-metadata-browser-worker/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- The browser automation lane is separate from the existing side-effect writer lane.
- The browser worker should be an internal companion service, not a public addon requirement.
- Docker Compose is the expected local/self-hosted deployment mechanism.
- The first proof uses a deterministic local rendered-page fixture before adding Douban-specific behavior.
- The Rust sidecar uses an HTTP provider adapter and does not depend on Playwright or Crawlee.

## Blockers

- None for the deterministic worker proof and sidecar integration.
- Douban live access may require proxy, headers, cookies, rate limits, or recorded fixtures before OMBW-040 can be accepted.

## Next Recommended Action

- Implement OMBW-040: add the first Douban-backed or recorded-fixture browser-worker consumer without expanding the public addon surface.
