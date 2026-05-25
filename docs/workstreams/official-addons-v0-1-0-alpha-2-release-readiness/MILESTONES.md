# Official Addons v0.1.0-alpha.2 Release Readiness - Milestones

Status: Blocked on publish approval
Last updated: 2026-05-24
Last refreshed: 2026-05-25

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

- SDK crate package dry-runs pass or are blocked by a concrete publish-order
  constraint.
- Metadata scraper and notification bridge package dry-runs pass once
  registry-shaped dependencies are available.

Status: Blocked 2026-05-24; refreshed 2026-05-25.

The protocol crate dry-run passes. The client, official addon catalog,
notification bridge, and metadata scraper package dry-runs require publishing
upstream alpha.2 crates first.

## M3 - Release Evidence

Exit criteria:

- Parser, metadata, focused nextest, fmt, and diff checks pass.
- Live Docker/server smoke passes or is explicitly blocked by Docker daemon
  availability.
- Handoff states the next release action.

Status: Done for local gates 2026-05-24; refreshed 2026-05-25 with workspace
nextest 183/183 passing. Live Docker/server smoke remains blocked by Docker
daemon availability.
