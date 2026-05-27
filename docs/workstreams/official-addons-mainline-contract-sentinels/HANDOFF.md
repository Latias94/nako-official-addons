# Official Addons Mainline Contract Sentinels - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

This lane was opened because the main `nako` repository is actively evolving
casting, playback capability planning, and transcode readiness while official
addons still need to move quickly without accumulating cross-repo drift.

Completed work:

- OAMC-020: notification bridge now delegates official manifest facts to
  `nako-official-addon-catalog`; sidecar-local provider test-send diagnostics
  remain local.
- OAMC-030: release gate checks out `Latias94/nako` as a sibling, runs all
  three official Rust sidecars, builds all three sidecar images with explicit
  BuildKit `nako=../nako` named context, and smokes all three images through a
  shared Python smoke script.
- OAMC-040: crates-publish dry-run/publish automation now iterates
  `nako-notification-bridge`, `nako-metadata-scraper`, and
  `nako-chromecast-renderer`; alpha.2 release-readiness docs now name the same
  package set.
- OAMC-050: local Cargo verification passed.

## Evidence

- `cargo nextest run -p nako-notification-bridge manifest routes --no-fail-fast`:
  31 passed, 13 skipped.
- `cargo metadata --format-version 1 --no-deps`: pass.
- `cargo fmt --all -- --check`: pass.
- `cargo nextest run --workspace --no-fail-fast`: 337 passed, 3 skipped.
- `git diff --check`: pass with existing Cargo.lock LF/CRLF warning.
- `cargo publish --dry-run --allow-dirty` for all three official addon crates:
  blocked as expected by missing upstream alpha.2 SDK/catalog crates on
  crates.io.
- `docker version --format '{{.Server.Version}}'`: blocked because Docker
  daemon is not reachable on `//./pipe/docker_engine`.

## Follow-On Recommendation

Do not open another addon refactor lane until either:

- upstream alpha.2 SDK/catalog crates are published and release-readiness can be
  unblocked; or
- main `../nako` changes the Addon Protocol, official catalog fields, renderer
  adapter contract, or playback/transcode addon-facing capability surface.

## Guardrails

- Do not publish crates or push images.
- Do not touch untracked files in `../nako`.
- CI checkout of private `Latias94/nako` requires `NAKO_REPO_TOKEN` or another
  token with access if the default `github.token` cannot read the private main
  repository.
