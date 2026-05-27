# Official Addons v0.1.0-alpha.2 Release Readiness - Handoff

Status: Ready for addon publish approval
Last updated: 2026-05-24
Last refreshed: 2026-05-27

## Current State

The closed cross-repo refactor and contract-hardening lanes left two release
readiness follow-ons:

- live Docker/server smoke proof;
- package dry-run proof after moving new SDK API into public crates.

The first E2E preflight passed before Docker daemon availability was checked.
A direct `docker version` command then proved the daemon was not reachable.
The E2E script now has a daemon preflight guard.

`cargo publish --dry-run` for `nako-metadata-scraper`,
`nako-notification-bridge`, and `nako-chromecast-renderer` now verifies against
published SDK/catalog crates. `nako-official-addon-catalog 0.1.0-alpha.2` was
published on 2026-05-27 after `nako-addon-protocol`, `nako-addon-client`, and
`nako` were already visible on crates.io.

## Active Task

- OAR2-030: package verification is complete. Remaining action is explicit
  approval to publish the three official addon crates.

## Next Commands

After user approval, publish in this order:

```powershell
cargo publish -p nako-notification-bridge --locked
cargo publish -p nako-metadata-scraper --locked
cargo publish -p nako-chromecast-renderer --locked
```

Then re-run official addon package dry-runs or publish verification.

## Blockers

- Full live Docker/server smoke is blocked until Docker daemon is running.
- Real addon crate publication still requires explicit user approval.

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
- `npm test` in `addons/browser-worker` passed: 1 passed.
- `cargo fmt --all -- --check` passed in both repositories.
- `git diff --check` passed in both repositories with LF/CRLF warnings.

## Dirty Worktree Notes

- `nako-official-addons` is ahead of `origin/main` with committed release,
  notification-bridge, Chromecast renderer, and mainline contract sentinel work.
- `../nako` has CI publish-script changes plus unrelated workstream changes.
  Stage only the publish-script/checklist files for this lane.
