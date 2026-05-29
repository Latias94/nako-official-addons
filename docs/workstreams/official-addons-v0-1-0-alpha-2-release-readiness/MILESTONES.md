# Official Addons v0.1.0-alpha.2 Release Readiness - Milestones

Status: Complete
Last updated: 2026-05-28
Last refreshed: 2026-05-28

## M0 - Lane Open

Exit criteria:

- Package dry-run failure is captured as the reason for alpha.2 readiness.
- Docker daemon blocker is captured as the reason live smoke is not yet proven.

## M1 - Version Boundary

Exit criteria:

- Addon/package version metadata uses `0.1.0-alpha.2`.
- Addon Protocol runtime version remains `0.1.0-alpha.1`.
- Manifest drift tests pass.

Status: Done 2026-05-24.

## M2 - Package Verification

Exit criteria:

- SDK/catalog crate package dry-runs pass or are published.
- Metadata scraper, notification bridge, and Chromecast renderer package dry-runs pass once
  registry-shaped dependencies are available.

Status: Done 2026-05-27.

The protocol, client, official addon catalog, notification bridge, metadata
scraper, and Chromecast renderer alpha.2 crates are published and visible on
crates.io.

## M3 - Release Evidence

Exit criteria:

- Parser, metadata, focused nextest, fmt, and diff checks pass.
- Live Docker/server smoke passes or is explicitly blocked by Docker daemon
  availability.
- Handoff states the next release action.

Status: Done 2026-05-28.

Hosted Docker/server smoke passed against
`ghcr.io/latias94/nako-server:0.1.0-alpha.2` with the workspace metadata
scraper sidecar. The run proved container health, sidecar manifest/resource
paths, Nako registration/enabling, routing plan sync, direct Addon Task, and
manager plan confirmation.
