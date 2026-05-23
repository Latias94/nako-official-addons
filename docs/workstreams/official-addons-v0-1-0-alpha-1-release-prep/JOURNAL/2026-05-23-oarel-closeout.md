# OAREL Closeout - 2026-05-23

## Summary

Prepared the official addons repository for `v0.1.0-alpha.1`:

- Aligned workspace and crate metadata with Nako core alpha.1.
- Added crate README metadata, repository AGPL license file, and alpha.1
  changelog.
- Documented Addon Protocol `0.1.0-alpha.1` and protocol crate
  `0.1.0-alpha.1`.
- Updated manifest, compose, systemd, and User-Agent examples to alpha.1.
- Converted the metadata scraper Dockerfile to cargo-chef stages using this
  repository as the build context.
- Removed runtime `curl` dependency after Debian mirror failures showed it was
  unnecessary for the sidecar image.

## Evidence

- `cargo metadata --format-version 1 --no-deps`: passed.
- `cargo nextest run -p nako-metadata-scraper manifest --no-fail-fast`: 5
  passed.
- `cargo fmt --all -- --check`: passed.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: 34 passed.
- `cargo nextest run --workspace --no-fail-fast`: 34 passed.
- After the protocol crate was published, `cargo publish -p
  nako-metadata-scraper --locked --dry-run --allow-dirty` passed against
  crates.io dependencies.
- Docker build and temporary container `/manifest.json` smoke passed after the
  switch to the published crates.io protocol dependency.
- `git diff --check`: passed with only the existing `Cargo.lock` LF/CRLF
  warning.

## Follow-ons

- Review, commit, and tag `v0.1.0-alpha.1`.
- Build and push production image tags after registry naming is decided.
- Re-run live Nako Admin-mediated smoke when a local Nako server and
  `NAKO_ADMIN_TOKEN` are available.
