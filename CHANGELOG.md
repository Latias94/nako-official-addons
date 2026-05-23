# Changelog

All notable changes to Nako Official Addons are documented here.

This project follows SemVer for release labels. The addon protocol is still
alpha, so `0.1.0-alpha.1` is not a compatibility promise for future alpha
builds.

## 0.1.0-alpha.1 - 2026-05-23

### Added

- Initial official metadata scraper Addon Sidecar for Nako Addon Protocol
  `0.1.0-alpha.1`.
- Fixture metadata provider enabled by default for local smoke tests.
- TMDB movie metadata provider baseline behind explicit opt-in configuration.
- Bangumi subject metadata provider baseline behind explicit opt-in
  configuration.
- Provider registry, provider-neutral HTTP runtime, and provider-neutral
  ranking/evidence model.
- Runtime-generated addon manifest and checked-in manifest drift test.
- Local sidecar smoke script for manifest, health, and metadata checks.
- Docker build using cargo-chef planner/cacher/builder stages and a BuildKit
  named context for the local Nako protocol crate.

### Changed

- Workspace and crate metadata now align with Nako `0.1.0-alpha.1`.
- The `nako-addon-protocol` dependency uses the published crates.io
  `0.1.0-alpha.1` crate.
- Operator examples, image tags, systemd sample, compose sample, and provider
  User-Agent examples now use `0.1.0-alpha.1`.

### Known Gaps

- Live Nako Admin-mediated smoke requires a running local Nako server and
  `NAKO_ADMIN_TOKEN`; direct sidecar and Docker container smoke are covered.
- TMDB and Bangumi are baseline provider adapters, not exhaustive scraper
  implementations.
