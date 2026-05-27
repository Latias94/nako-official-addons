# Official Addons v0.1.0-alpha.2 Release Readiness - TODO

Status: Complete
Last updated: 2026-05-28
Last refreshed: 2026-05-28

Task IDs use the `OAR2` prefix.

## M0 - Scope

- [x] OAR2-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-addons-v0-1-0-alpha-2-release-readiness]
  Goal: Open the alpha.2 release-readiness lane from package dry-run and live
  smoke findings.
  Validation: Workstream docs agree.
  Evidence: Workstream docs.
  Result: DONE 2026-05-24.
  Handoff: Continue with OAR2-020 version boundary repair.

## M1 - Version Boundary

- [x] OAR2-020 [owner=codex] [deps=OAR2-010] [scope=Cargo.toml,../nako/crates/nako-addon-client,../nako/crates/nako-addon-protocol,docs]
  Goal: Advance Rust crate package versions and addon package version to
  `0.1.0-alpha.2` while keeping Addon Protocol runtime compatibility at
  `0.1.0-alpha.1`.
  Validation: `cargo metadata --format-version 1 --no-deps`; focused manifest
  tests.
  Review: Do not change `ADDON_PROTOCOL_VERSION` unless the wire protocol
  compatibility decision changes.
  Evidence: Metadata output and manifest drift tests.
  Result: DONE 2026-05-24.
  Evidence: Cargo metadata reports `nako-metadata-scraper`,
  `nako-addon-client`, and `nako-addon-protocol` package versions
  `0.1.0-alpha.2`; manifest `protocol_version` remains
  `0.1.0-alpha.1`; focused and full metadata scraper tests passed.
  Handoff: Continue with OAR2-030 package verification.

## M2 - Package Verification

- [x] OAR2-030 [owner=codex] [deps=OAR2-020] [scope=crates/nako-metadata-scraper,crates/nako-notification-bridge,crates/nako-chromecast-renderer,../nako/crates/nako-addon-client,../nako/crates/nako-addon-protocol,../nako/crates/nako-official-addon-catalog]
  Goal: Prove official addon packaging verifies against registry-shaped SDK and
  official catalog dependencies.
  Validation: `cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty`; `cargo publish -p nako-notification-bridge --locked --dry-run --allow-dirty`; `cargo publish -p nako-chromecast-renderer --locked --dry-run --allow-dirty`.
  Review: If SDK crates are not yet published, record the exact blocker and
  verify the SDK crate dry-runs separately.
  Evidence: Package dry-run output.
  Result: DONE 2026-05-27 after `nako-official-addon-catalog 0.1.0-alpha.2`
  was published.
  Evidence: `nako-addon-protocol`, `nako-addon-client`,
  `nako-official-addon-catalog`, and `nako` are visible as
  `0.1.0-alpha.2` crates on crates.io. `python scripts/publish_crates.py
  --mode dry-run --allow-dirty` now verifies all public Nako crates from the
  main repository. `cargo publish --dry-run` verifies
  `nako-notification-bridge`, `nako-metadata-scraper`, and
  `nako-chromecast-renderer` against the published SDK/catalog dependency
  boundary.
  Refresh 2026-05-27: publish and dry-run lists now include
  `nako-chromecast-renderer`, which requires `nako-addon-protocol
  0.1.0-alpha.2` and `nako-official-addon-catalog 0.1.0-alpha.2`.
  Refresh 2026-05-27 after catalog publish: `cargo publish --dry-run` passed
  for `nako-notification-bridge`, `nako-metadata-scraper`, and
  `nako-chromecast-renderer`.
  Refresh 2026-05-25: workspace nextest passed 183/183 with 2 skipped;
  metadata/fmt/diff checks passed.
  Refresh 2026-05-27 after user approval: `nako-notification-bridge`,
  `nako-metadata-scraper`, and `nako-chromecast-renderer` `0.1.0-alpha.2`
  were published and are visible in crates.io search.
  Handoff: All alpha.2 crates are published and live Docker/server smoke has
  passed.

## M3 - Smoke And Closeout

- [x] OAR2-040 [owner=codex] [deps=OAR2-020] [scope=scripts,docs/workstreams]
  Goal: Prove smoke harness preflight behavior and record live smoke status.
  Validation: PowerShell parser checks; E2E preflight; live E2E smoke if Docker
  daemon is reachable.
  Review: Docker daemon absence is an environment blocker, not a code pass.
  Evidence: EVIDENCE_AND_GATES.md.
  Result: DONE 2026-05-28 after Docker live smoke passed.
  Evidence: PowerShell parser check passed. E2E preflight fails early with a
  Docker daemon diagnostic when Docker is not reachable. After Docker became
  reachable, `docker version --format '{{.Server.Version}}'` reported `25.0.3`;
  preflight passed; full hosted smoke passed with Nako health, metadata
  scraper manifest/health/resource/event checks, Nako registration/enabling,
  resource diagnostic, routing plan sync, direct Addon Task, and manager plan
  confirmation.
  Handoff: Lane is complete.
