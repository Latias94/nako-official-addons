# OMLDC-040 - Closeout

## Summary

- Added `crates/nako-metadata-scraper/tests/live_provider_drift.rs` with ignored, env-gated live
  smoke checks for TMDB direct lookup and Bangumi direct lookup.
- Documented the manual invocation path in `crates/nako-metadata-scraper/README.md`.
- Verified default gates with `cargo nextest run -p nako-metadata-scraper --no-fail-fast`,
  `cargo fmt --all -- --check`, and `git diff --check`.
- Ran the manual live gate with `NAKO_METADATA_SCRAPER_LIVE_PROVIDER_DRIFT=1 cargo test -p
  nako-metadata-scraper --test live_provider_drift -- --ignored`.

## Residual Risk

- TMDB live execution was not exercised with a token in this workspace, so the direct live path still
  depends on operator-provided credentials when it is run for real.
