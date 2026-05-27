# Official Addons v0.1.0-alpha.2 Release Readiness - Evidence And Gates

Status: Blocked on publish approval
Last updated: 2026-05-24
Last refreshed: 2026-05-27

## Gates

```powershell
cargo metadata --format-version 1 --no-deps
cargo nextest run -p nako-metadata-scraper manifest routes task bulk nako_runtime writeback artwork --no-fail-fast
cargo fmt --all -- --check
cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty
cargo publish -p nako-notification-bridge --locked --dry-run --allow-dirty
cargo publish -p nako-chromecast-renderer --locked --dry-run --allow-dirty
git diff --check
```

Nako SDK gates:

```powershell
cargo nextest run -p nako-addon-client runtime --no-fail-fast
cargo nextest run -p nako-addon-protocol protected_write_payload_contracts_keep_wire_shape --no-fail-fast
cargo publish -p nako-addon-protocol --locked --dry-run --allow-dirty
cargo publish -p nako-addon-client --locked --dry-run --allow-dirty
cargo publish -p nako-official-addon-catalog --locked --dry-run --allow-dirty
```

Smoke gates:

```powershell
$errors = $null; [System.Management.Automation.PSParser]::Tokenize((Get-Content -Raw -Path '../nako/scripts/official-addon-e2e-smoke.ps1'), [ref]$errors) | Out-Null; if ($errors) { $errors; exit 1 }
pwsh -NoProfile -ExecutionPolicy Bypass -File ../nako/scripts/official-addon-e2e-smoke.ps1 -AddonRepo F:\SourceCodes\Rust\nako-official-addons -AddonBinarySource workspace -PreflightOnly
pwsh -NoProfile -ExecutionPolicy Bypass -File ../nako/scripts/official-addon-e2e-smoke.ps1 -AddonRepo F:\SourceCodes\Rust\nako-official-addons -AddonBinarySource workspace
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-24 | OAR2-010 | Opened this lane after live smoke preflight and package dry-run findings. | Pass |
| 2026-05-24 | OAR2-010 | `pwsh -NoProfile -ExecutionPolicy Bypass -File ../nako/scripts/official-addon-e2e-smoke.ps1 -AddonRepo F:\SourceCodes\Rust\nako-official-addons -AddonBinarySource workspace -PreflightOnly` before daemon guard. | Pass: preflight completed while Docker daemon was later found unreachable |
| 2026-05-24 | OAR2-010 | `docker version --format '{{.Server.Version}}'` | Blocked: Docker daemon is not reachable on `//./pipe/docker_engine` |
| 2026-05-24 | OAR2-010 | `cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty` before version repair. | Fail: package verification used published `nako-addon-client/protocol 0.1.0-alpha.1`, which lacks the newly required runtime/task DTO API |
| 2026-05-24 | OAR2-020 | Advanced package/addon version metadata to `0.1.0-alpha.2` while keeping runtime Addon Protocol compatibility at `0.1.0-alpha.1`. | Pass |
| 2026-05-24 | OAR2-020 | `cargo metadata --format-version 1 --no-deps` in `nako-official-addons` | Pass: `nako-metadata-scraper` is `0.1.0-alpha.2`; SDK dependencies require `^0.1.0-alpha.2` |
| 2026-05-24 | OAR2-020 | `cargo metadata --format-version 1 --no-deps` in `../nako` | Pass: `nako-addon-client` and `nako-addon-protocol` are `0.1.0-alpha.2`; `nako` SDK dependency constraints point to `^0.1.0-alpha.2` |
| 2026-05-24 | OAR2-020 | `cargo nextest run -p nako-metadata-scraper manifest routes task bulk nako_runtime writeback artwork --no-fail-fast` | Pass: 23 passed, 117 skipped |
| 2026-05-24 | OAR2-020 | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Pass: 138 passed, 2 skipped |
| 2026-05-24 | OAR2-020 | `cargo nextest run -p nako --features addon-client --no-fail-fast` in `../nako` | Pass: 1 passed |
| 2026-05-24 | OAR2-020 | `npm test` in `addons/browser-worker` | Pass: 1 passed |
| 2026-05-24 | OAR2-030 | `cargo publish -p nako-addon-protocol --locked --dry-run --allow-dirty` in `../nako` | Pass: packaged 6 files and verified `nako-addon-protocol v0.1.0-alpha.2` |
| 2026-05-24 | OAR2-030 | `cargo publish -p nako-addon-client --locked --dry-run --allow-dirty` in `../nako` | Blocked: crates.io has `nako-addon-protocol 0.1.0-alpha.1` only, so `^0.1.0-alpha.2` cannot resolve until protocol alpha.2 is published |
| 2026-05-24 | OAR2-030 | `cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty` | Blocked: crates.io has `nako-addon-client 0.1.0-alpha.1` only, so `^0.1.0-alpha.2` cannot resolve until client alpha.2 is published |
| 2026-05-24 | OAR2-040 | PowerShell parser check for `../nako/scripts/official-addon-e2e-smoke.ps1` | Pass |
| 2026-05-24 | OAR2-040 | `pwsh -NoProfile -ExecutionPolicy Bypass -File ../nako/scripts/official-addon-e2e-smoke.ps1 -AddonRepo F:\SourceCodes\Rust\nako-official-addons -AddonBinarySource workspace -PreflightOnly` | Blocked as intended: now fails early with Docker daemon diagnostic |
| 2026-05-24 | OAR2-040 | `cargo fmt --all -- --check` in both repositories | Pass |
| 2026-05-24 | OAR2-040 | `git diff --check` in both repositories | Pass with existing LF/CRLF warnings |
| 2026-05-25 | OAR2-030 | `cargo metadata --format-version 1 --no-deps` in `nako-official-addons` | Pass: `nako-metadata-scraper` and `nako-notification-bridge` are `0.1.0-alpha.2`; local SDK dependencies require `^0.1.0-alpha.2` |
| 2026-05-25 | OAR2-030 | `cargo nextest run --workspace --no-fail-fast` in `nako-official-addons` | Pass: 183 passed, 2 skipped |
| 2026-05-25 | OAR2-030 | `cargo fmt --all -- --check`; `git diff --check` in `nako-official-addons` | Pass |
| 2026-05-25 | OAR2-030 | `cargo publish -p nako-addon-protocol --locked --dry-run --allow-dirty` in `../nako` | Pass: packaged and verified `nako-addon-protocol v0.1.0-alpha.2`; dry-run aborted before upload |
| 2026-05-25 | OAR2-030 | `cargo publish -p nako-addon-client --locked --dry-run --allow-dirty` in `../nako` | Blocked: crates.io has `nako-addon-protocol 0.1.0-alpha.1` only, so `^0.1.0-alpha.2` cannot resolve until protocol alpha.2 is published |
| 2026-05-25 | OAR2-030 | `cargo publish -p nako-official-addon-catalog --locked --dry-run --allow-dirty` in `../nako` | Blocked: crates.io has `nako-addon-protocol 0.1.0-alpha.1` only, so `^0.1.0-alpha.2` cannot resolve until protocol alpha.2 is published |
| 2026-05-25 | OAR2-030 | `cargo publish -p nako-notification-bridge --locked --dry-run --allow-dirty` | Blocked: crates.io has `nako-addon-protocol 0.1.0-alpha.1` only, so `^0.1.0-alpha.2` cannot resolve until protocol alpha.2 is published |
| 2026-05-25 | OAR2-030 | `cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty` | Blocked: crates.io has `nako-addon-client 0.1.0-alpha.1` only, so `^0.1.0-alpha.2` cannot resolve until client alpha.2 is published; `nako-official-addon-catalog 0.1.0-alpha.2` will also be required before metadata scraper publish |

## Known Constraints

- Do not publish crates or push images without user approval.
- Do not change `ADDON_PROTOCOL_VERSION` unless runtime wire compatibility is
  intentionally changed.
- Docker live smoke requires a reachable Docker daemon.
- `../nako` has unrelated dirty files; stage only files touched for this lane.
- Publish order after approval: `nako-addon-protocol 0.1.0-alpha.2`, then
  `nako-addon-client 0.1.0-alpha.2` and `nako-official-addon-catalog
  0.1.0-alpha.2`, then `nako-notification-bridge 0.1.0-alpha.2`,
  `nako-metadata-scraper 0.1.0-alpha.2`, and
  `nako-chromecast-renderer 0.1.0-alpha.2`.
