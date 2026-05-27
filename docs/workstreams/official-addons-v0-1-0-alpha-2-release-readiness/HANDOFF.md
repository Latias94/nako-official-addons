# Official Addons v0.1.0-alpha.2 Release Readiness - Handoff

Status: Complete
Last updated: 2026-05-28
Last refreshed: 2026-05-28

## Current State

The closed cross-repo refactor and contract-hardening lanes left two release
readiness follow-ons:

- live Docker/server smoke proof;
- package dry-run proof after moving new SDK API into public crates.

The first E2E preflight passed before Docker daemon availability was checked.
A direct `docker version` command then proved the daemon was not reachable.
The E2E script now has a daemon preflight guard. On 2026-05-28 the daemon was
reachable and the hosted smoke passed.

`cargo publish --dry-run` for `nako-metadata-scraper`,
`nako-notification-bridge`, and `nako-chromecast-renderer` now verifies against
published SDK/catalog crates. `nako-official-addon-catalog 0.1.0-alpha.2` was
published on 2026-05-27 after `nako-addon-protocol`, `nako-addon-client`, and
`nako` were already visible on crates.io. `nako-notification-bridge`,
`nako-metadata-scraper`, and `nako-chromecast-renderer` `0.1.0-alpha.2` were
then published and are visible in crates.io search.

## Active Task

- None. OAR2-030 package verification/publication and OAR2-040 live
  Docker/server smoke are complete.

## Next Commands

No required release-readiness commands remain for this lane.

## Blockers

None for this lane.

## Evidence

- `cargo metadata --format-version 1 --no-deps` passed in both repositories;
  2026-05-25 refresh passed in `nako-official-addons`.
- `cargo nextest run --workspace --no-fail-fast` in `nako-official-addons`
  passed: 183 passed, 2 skipped.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed: 138
  passed, 2 skipped.
- `cargo nextest run -p nako-addon-client runtime --no-fail-fast` passed: 6
  passed, 9 skipped.
- `cargo nextest run -p nako-addon-protocol protected_write_payload_contracts_keep_wire_shape --no-fail-fast`
  passed: 1 passed, 10 skipped.
- `python scripts/publish_crates.py --mode publish --allow-dirty --registry-settle-seconds 5`
  in `../nako` published `nako-official-addon-catalog 0.1.0-alpha.2` and
  skipped already published protocol/client/nako crates.
- `python scripts/publish_crates.py --mode dry-run --allow-dirty` in `../nako`
  passed for protocol/client/catalog/nako crates at `0.1.0-alpha.2`.
- `cargo publish -p nako-notification-bridge --locked --dry-run` passed.
- `cargo publish -p nako-metadata-scraper --locked --dry-run` passed.
- `cargo publish -p nako-chromecast-renderer --locked --dry-run` passed.
- `cargo publish -p nako-notification-bridge --locked` published
  `0.1.0-alpha.2`.
- `cargo publish -p nako-metadata-scraper --locked` published
  `0.1.0-alpha.2`.
- `cargo publish -p nako-chromecast-renderer --locked` published
  `0.1.0-alpha.2`.
- `cargo search` reports all three official addon crates at `0.1.0-alpha.2`.
- `docker version --format '{{.Server.Version}}'` reported `25.0.3`.
- `official-addon-e2e-smoke.ps1 -PreflightOnly` passed.
- Full `official-addon-e2e-smoke.ps1` passed against
  `ghcr.io/latias94/nako-server:0.1.0-alpha.2`, with logs under
  `../nako/target/oae2e-alpha2-hosted/20260528-000651/logs`.
- `npm test` in `addons/browser-worker` passed: 1 passed.
- `cargo fmt --all -- --check` passed in both repositories.
- `git diff --check` passed in both repositories with LF/CRLF warnings.

## Dirty Worktree Notes

- `nako-official-addons` is ahead of `origin/main` with committed release,
  notification-bridge, Chromecast renderer, and mainline contract sentinel work.
- `../nako` has the committed CI publish-script change plus unrelated
  workstream changes.
