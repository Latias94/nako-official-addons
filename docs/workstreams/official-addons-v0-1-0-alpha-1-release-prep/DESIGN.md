# Official Addons v0.1.0-alpha.1 Release Prep

Status: Complete
Last updated: 2026-05-23

## Problem

The metadata scraper is functionally ready for an alpha release, but repository
metadata, crate metadata, addon manifest versioning, license files, and Docker
build caching are not aligned with Nako core's `0.1.0-alpha.1` release shape.

## Target State

- Workspace and crate version are `0.1.0-alpha.1`.
- Cargo metadata matches Nako core where appropriate: authors, license,
  repository, homepage, description, and readme.
- The protocol dependency is constrained to the published `0.1.0-alpha.1`
  crates.io crate.
- Addon manifest examples and operator docs expose the alpha version.
- Repository license and changelog files exist and match the release position.
- Docker builds use cargo-chef for dependency caching with this repository as
  the build context.
- Release gates prove package/workspace tests and manifest drift checks.

## Nako Core Facts

Read from `../nako`:

- Workspace version: `0.1.0-alpha.1`.
- Authors: `Mingzhen Zhuang <superfrankie621@gmail.com>`.
- Nako core license: `AGPL-3.0-or-later`.
- Protocol crate license: `Apache-2.0 OR MIT`.
- Protocol crate inherits the `0.1.0-alpha.1` workspace version.

## Scope

In scope:

- Cargo metadata and package version.
- README and crate README.
- License file presence.
- Changelog for `0.1.0-alpha.1`.
- Addon manifest example version.
- Dockerfile cargo-chef stages.
- Compose/systemd version-visible examples.

Out of scope:

- Publishing to crates.io.
- Building and pushing container images.
- Tagging git releases.
- Changing `../nako`.

## Release Position

`v0.1.0-alpha.1` is the right label because the addon protocol and server
integration are still alpha and live Nako Admin-mediated smoke is still an
operator-run evidence item.

## Closeout

Completed 2026-05-23. The repository metadata, crate metadata, changelog,
addon examples, license file, and Docker build have been aligned to Nako
`0.1.0-alpha.1`. The Dockerfile now uses cargo-chef planner/cacher/builder
stages, then produces a slim runtime image with CA certificates for outbound
provider TLS.
