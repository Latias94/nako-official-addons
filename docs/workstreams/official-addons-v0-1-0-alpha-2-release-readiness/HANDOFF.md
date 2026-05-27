# Official Addons v0.1.0-alpha.2 Release Readiness - Handoff

Status: Blocked on publish approval
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
`nako-notification-bridge`, and `nako-chromecast-renderer` is expected to
verify against published SDK/catalog crates. The local path dependencies
contain new APIs, but the registry crates may not. This requires an alpha.2
package release line.

## Active Task

- OAR2-030: package verification is blocked on publishing upstream alpha.2 SDK
  crates.

## Next Commands

```powershell
cargo metadata --format-version 1 --no-deps
cargo nextest run --workspace --no-fail-fast
cargo fmt --all -- --check
```

After user approval, publish in this order:

1. `cargo publish -p nako-addon-protocol --locked`
2. `cargo publish -p nako-addon-client --locked`
3. `cargo publish -p nako-official-addon-catalog --locked`
4. `cargo publish -p nako-notification-bridge --locked`
5. `cargo publish -p nako-metadata-scraper --locked`
6. `cargo publish -p nako-chromecast-renderer --locked`

Then re-run official addon package dry-runs or publish verification.

## Blockers

- Full live Docker/server smoke is blocked until Docker daemon is running.
- `nako-addon-client` dry-run is blocked until `nako-addon-protocol
  0.1.0-alpha.2` is published to crates.io.
- `nako-official-addon-catalog` dry-run is blocked until
  `nako-addon-protocol 0.1.0-alpha.2` is published to crates.io.
- `nako-notification-bridge` dry-run is blocked until `nako-addon-protocol
  0.1.0-alpha.2` is published to crates.io.
- `nako-metadata-scraper` dry-run is blocked until `nako-addon-client
  0.1.0-alpha.2` and `nako-official-addon-catalog 0.1.0-alpha.2` are published
  to crates.io.
- `nako-chromecast-renderer` dry-run is blocked until `nako-addon-protocol
  0.1.0-alpha.2` and `nako-official-addon-catalog 0.1.0-alpha.2` are published
  to crates.io.

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
- `cargo publish -p nako-addon-protocol --locked --dry-run --allow-dirty`
  passed in both the original check and 2026-05-25 refresh.
- `cargo publish -p nako-addon-client --locked --dry-run --allow-dirty` remains
  blocked on missing `nako-addon-protocol 0.1.0-alpha.2` in crates.io.
- `cargo publish -p nako-official-addon-catalog --locked --dry-run --allow-dirty`
  remains blocked on missing `nako-addon-protocol 0.1.0-alpha.2` in crates.io.
- `cargo publish -p nako-notification-bridge --locked --dry-run --allow-dirty`
  is blocked on missing `nako-addon-protocol 0.1.0-alpha.2` in crates.io.
- `cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty`
  is blocked on missing `nako-addon-client 0.1.0-alpha.2` in crates.io; it will
  also require `nako-official-addon-catalog 0.1.0-alpha.2`.
- `cargo publish -p nako-chromecast-renderer --locked --dry-run --allow-dirty`
  should be run after upstream alpha.2 SDK/catalog crates are published.
- `npm test` in `addons/browser-worker` passed: 1 passed.
- `cargo fmt --all -- --check` passed in both repositories.
- `git diff --check` passed in both repositories with LF/CRLF warnings.

## Dirty Worktree Notes

- `nako-official-addons` is ahead of `origin/main` with committed release,
  notification-bridge, and Chromecast renderer work.
- `../nako` has unrelated dirty server/library/playback and workstream files.
  Do not restore or stage unrelated files.
