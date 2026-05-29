# Official Addons Mainline Contract Sentinels - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Scope Locked

Exit criteria:

- Workstream docs exist and agree on the cross-repo drift problem.
- The lane explicitly avoids publishing and main-repo edits.

## M1 - Catalog Authority Restored

Exit criteria:

- `nako-notification-bridge` uses `nako-official-addon-catalog` for official
  notification bridge manifest facts.
- Manifest and route tests pass for notification bridge.

## M2 - Release Gate Covers The Suite

Exit criteria:

- CI checks out `nako` as a sibling of `nako-official-addons` before Cargo
  commands.
- CI runs package or workspace tests that include all current official sidecars.
- Container release gate builds and smokes metadata scraper, notification
  bridge, and Chromecast renderer.
- Docker build-context requirements are explicit.

## M3 - Publish Dry-Run Covers The Suite

Exit criteria:

- Dry-run automation names every publishable official addon crate in this repo.
- The workflow still requires explicit approval for real publication.
- Upstream publish blockers are recorded rather than hidden.

## M4 - Closeout

Exit criteria:

- Fresh local Cargo gates pass.
- Docker evidence is pass or a concrete environment blocker.
- Handoff names remaining mainline drift risks and the next recommended lane.

Closed 2026-05-27. Docker evidence is blocked by local daemon availability:
`docker version` cannot connect to `//./pipe/docker_engine`.
