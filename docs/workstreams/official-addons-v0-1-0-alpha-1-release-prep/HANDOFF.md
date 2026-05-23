# Official Addons v0.1.0-alpha.1 Release Prep - Handoff

Status: Complete
Last updated: 2026-05-23

## Current State

The release-prep workstream is complete. OAREL-010 through OAREL-050 are done.

## Completed Scope

- Cargo metadata, crate README, and root README align to `0.1.0-alpha.1`.
  Cargo `homepage` points to the main Nako repository
  `https://github.com/Latias94/nako`; Cargo `repository` points to this
  official addons repository.
- The local `nako-addon-protocol` path dependency is constrained to
  `0.1.0-alpha.1`.
- AGPL license file and `0.1.0-alpha.1` changelog are present; protocol crate
  licensing is documented as separate `Apache-2.0 OR MIT`.
- Addon manifest, compose, systemd, Dockerfile, and User-Agent examples use
  `0.1.0-alpha.1`.
- Dockerfile uses cargo-chef with BuildKit named context `nako-core=../nako`.
- Docker image build and temporary container `/manifest.json` smoke passed.

## Next Action

Review the release-prep diff, then commit and tag after user approval.

## Validation

- `cargo metadata --format-version 1 --no-deps`
- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-metadata-scraper manifest --no-fail-fast`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `docker buildx build --build-context nako-core=../nako -f addons/metadata-scraper/Dockerfile -t nako-metadata-scraper:0.1.0-alpha.1-release-prep --load .`
- temporary Docker container `/manifest.json` smoke
- `git diff --check`

## Follow-ons

- Push production image tags after registry decision.
- Re-run live Nako Admin-mediated smoke when a local Nako server and
  `NAKO_ADMIN_TOKEN` are available.
