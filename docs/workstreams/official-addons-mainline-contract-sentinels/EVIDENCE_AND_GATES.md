# Official Addons Mainline Contract Sentinels - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Planned Gates

```powershell
cargo metadata --format-version 1 --no-deps
cargo nextest run -p nako-notification-bridge manifest routes --no-fail-fast
cargo nextest run --workspace --no-fail-fast
cargo fmt --all -- --check
git diff --check
```

Docker gates, when Docker daemon is available and `../nako` exists beside this
repository:

```powershell
docker buildx build --build-context nako=../nako -f addons/metadata-scraper/Dockerfile -t nako-metadata-scraper:ci --load .
docker buildx build --build-context nako=../nako -f addons/notification-bridge/Dockerfile -t nako-notification-bridge:ci --load .
docker buildx build --build-context nako=../nako -f addons/chromecast-renderer/Dockerfile -t nako-chromecast-renderer:ci --load .
```

Publish dry-runs must remain non-publishing and may be blocked until upstream
alpha.2 SDK crates are published:

```powershell
cargo publish -p nako-notification-bridge --locked --dry-run --allow-dirty
cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty
cargo publish -p nako-chromecast-renderer --locked --dry-run --allow-dirty
```

## Evidence Log

| Date | Task | Command | Result |
| --- | --- | --- | --- |
| 2026-05-27 | OAMC-010 | `git status --short --branch` in `nako-official-addons` | Clean worktree, `main...origin/main [ahead 56]` before this lane started. |
| 2026-05-27 | OAMC-010 | `git status --short --branch` in `../nako` | Main repo ahead 63 with untracked `docs/adr/0047-cpu-transcode-readiness.md` and `docs/workstreams/cpu-transcode-readiness/`; this lane will not touch them. |
| 2026-05-27 | OAMC-010 | Inspection of `Cargo.toml`, `.github/workflows/release-gate.yml`, and `addons/*/Dockerfile` | Found local `../nako` path dependencies, metadata-only release gate smoke, and Dockerfiles that copy only the addon repository. |
| 2026-05-27 | OAMC-020 | `cargo nextest run -p nako-notification-bridge manifest routes --no-fail-fast` | Pass: 31 passed, 13 skipped. |
| 2026-05-27 | OAMC-030 | `cargo metadata --format-version 1 --no-deps` | Pass: workspace resolves all three official addon crates against local `../nako` SDK/catalog path dependencies. |
| 2026-05-27 | OAMC-030 | `python scripts/smoke_official_addon_container.py --help` | Pass: CI smoke script argument surface loads. |
| 2026-05-27 | OAMC-030 | `docker version --format '{{.Server.Version}}'` | Blocked: Docker daemon is not running/reachable on `//./pipe/docker_engine`; container build/smoke not claimed locally. |
| 2026-05-27 | OAMC-040 | `cargo publish -p nako-notification-bridge --locked --dry-run --allow-dirty` | Blocked as expected: crates.io has `nako-addon-protocol 0.1.0-alpha.1`, not `0.1.0-alpha.2`. |
| 2026-05-27 | OAMC-040 | `cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty` | Blocked as expected: crates.io has `nako-addon-client 0.1.0-alpha.1`, not `0.1.0-alpha.2`. |
| 2026-05-27 | OAMC-040 | `cargo publish -p nako-chromecast-renderer --locked --dry-run --allow-dirty` | Blocked as expected: crates.io has `nako-addon-protocol 0.1.0-alpha.1`, not `0.1.0-alpha.2`. |
| 2026-05-27 | OAMC-050 | `cargo fmt --all -- --check` | Pass. |
| 2026-05-27 | OAMC-050 | `cargo nextest run --workspace --no-fail-fast` | Pass: 337 passed, 3 skipped. |
| 2026-05-27 | OAMC-050 | `git diff --check` | Pass with existing Cargo.lock LF/CRLF warning. |

## Notes

- Do not publish crates or push images without user approval.
- Do not edit `../nako` source files in this lane.
- If GitHub Actions checks out private `Latias94/nako`, the workflow will need
  a token with access to that repository.
