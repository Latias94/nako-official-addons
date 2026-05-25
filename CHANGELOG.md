# Changelog

All notable changes to Nako Official Addons are documented here.

This project follows SemVer for release labels. The addon protocol is still
alpha, so `0.1.0-alpha.1` is not a compatibility promise for future alpha
builds.

## 0.1.0-alpha.2 - 2026-05-24

### Added

- Public `nako-addon-protocol` task envelope and protected-write side-effect
  DTOs used by the official metadata scraper.
- Public `nako-addon-client` runtime helpers for access checks and
  metadata/artwork side-effect submission.
- Admin-mediated smoke support for direct Nako-owned
  `bulk-metadata-scrape` Addon Task execution.
- Initial ACK-only `nako-notification-bridge` sidecar that declares a
  `library.scanned` event subscription and returns redaction-safe event ACKs
  without provider fan-out.
- `nako-notification-bridge` `http_webhook` configuration contract,
  fixture-backed send path, and redaction-safe provider diagnostics without
  live CI secrets.
- `nako-notification-bridge` `discord_webhook` platform adapter with
  default-disabled configuration, fixture-backed send path, redaction-safe
  diagnostics, and fail-closed multi-provider protection.
- `nako-notification-bridge` safe summary template controls with whitelisted
  event fact tokens and no raw event payload value access.
- `nako-notification-bridge` bounded in-memory provider attempt history for
  redaction-safe sidecar diagnostics without adding provider retry state to Nako
  core.
- `nako-notification-bridge` sidecar-local provider test-send endpoint that
  sends a synthetic redaction-safe notification through the single configured
  provider and fails closed for unsafe provider configuration states.
- Opt-in `nako-notification-bridge` live provider smoke script that skips by
  default and never requires CI secrets.

### Changed

- The official metadata scraper now depends on `nako-addon-client` and
  `nako-addon-protocol` `0.1.0-alpha.2`.
- Bangumi and Douban provider implementations are split into provider-local
  modules for client, enrichment, parser, mapper, and test support.
- Provider manifest schema and secret-reference declarations now come from
  provider-owned catalog entries through the provider registry.
- `nako-notification-bridge` provider attempt history now records actual
  provider send outcomes and failures without filling recent diagnostics with
  ACK-only disabled-provider records.
- `nako-notification-bridge` health and diagnostics now expose provider send
  path count plus aggregate configuration status, and health degrades for
  invalid provider configuration, multiple send paths, or invalid enabled
  provider templates.

### Fixed

- Package verification no longer relies on unpublished local path-only SDK
  APIs while retaining Addon Protocol runtime compatibility
  `0.1.0-alpha.1`.
- The official E2E smoke preflight now fails early when the Docker daemon is
  not reachable.

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
