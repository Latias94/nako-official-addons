# Official Addons v0.1.0-alpha.1 Release Prep - Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Baseline

- Current branch: `main`, ahead of origin by two commits before release prep.
- Current head before release prep: `6e1beaa feat(metadata-scraper): add bangumi provider baseline`.
- Nako core metadata source: `../nako/Cargo.toml`.
- Protocol crate metadata source:
  `../nako/crates/nako-addon-protocol/Cargo.toml`.
- cargo-chef reference source: `../nako/repo-ref/cargo-chef`.

## Gates

- `cargo metadata --format-version 1 --no-deps`
- `cargo fmt --all -- --check`
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`

## OAREL-010

Status: DONE 2026-05-23.

Evidence:

- Workstream opened with Nako core metadata facts, target release label, scope,
  non-goals, task ledger, milestones, and gates.

## OAREL-020

Status: DONE 2026-05-23.

Evidence:

- `cargo metadata --format-version 1 --no-deps` passed. It reports
  `nako-metadata-scraper` version `0.1.0-alpha.1`, author
  `Mingzhen Zhuang <superfrankie621@gmail.com>`, readme `README.md`,
  homepage `https://github.com/Latias94/nako`, repository
  `https://github.com/Latias94/nako-official-addons`, rust-version `1.95`,
  and a local path dependency on `../nako/crates/nako-addon-protocol`
  constrained to `^0.1.0-alpha.1`.
- `cargo nextest run -p nako-metadata-scraper manifest --no-fail-fast` passed:
  5 tests passed, 29 skipped.

## OAREL-030

Status: DONE 2026-05-23.

Evidence:

- Added repository `LICENSE` matching the declared `AGPL-3.0-or-later`
  workspace license.
- Added `CHANGELOG.md` for `0.1.0-alpha.1` with added, changed, and known-gap
  release notes.
- Root README and addon README document release target `v0.1.0-alpha.1`,
  Addon Protocol `0.1.0-alpha.1`, protocol crate `0.1.0-alpha.1`, and the
  protocol crate's separate `Apache-2.0 OR MIT` license.
- `addons/metadata-scraper/manifest.example.json`, compose example, systemd
  example, and Bangumi User-Agent examples now use `0.1.0-alpha.1`.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed:
  34 tests passed.

## OAREL-040

Status: DONE 2026-05-23.

Evidence:

- `addons/metadata-scraper/Dockerfile` uses cargo-chef `chef`, `planner`,
  `cacher`, `builder`, and `runtime` stages.
- Docker build command passed:
  `docker buildx build --build-context nako-core=../nako -f
  addons/metadata-scraper/Dockerfile -t
  nako-metadata-scraper:0.1.0-alpha.1-release-prep --load .`.
- Built image:
  `sha256:83eacfdcf3b8a668c76e3f988b21222323754182d8998cbcb4a7b4240f330e21`
  size `91440391` bytes.
- Container smoke passed against a temporary container on `127.0.0.1:19101`:
  `/manifest.json` returned id `nako.official.metadata-scraper`, version
  `0.1.0-alpha.1`, protocol version `0.1.0-alpha.1`.
- The first Docker attempt hit Debian mirror `502 Bad Gateway` while installing
  `curl`. The runtime image does not need curl, so the Dockerfile now installs
  only `ca-certificates`.

## OAREL-050

Status: DONE 2026-05-23.

Final gates:

- `cargo metadata --format-version 1 --no-deps` passed.
- `cargo fmt --all -- --check` passed.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed:
  34 tests passed.
- `cargo nextest run --workspace --no-fail-fast` passed: 34 tests passed.
- `git diff --check` passed with only the existing `Cargo.lock` LF/CRLF warning.

Follow-ons:

- Tag and push `v0.1.0-alpha.1` after review.
- Build and push the production container image after deciding the registry
  namespace and tags.
- Re-run live Nako Admin-mediated smoke with a real local Nako server and
  `NAKO_ADMIN_TOKEN`.
