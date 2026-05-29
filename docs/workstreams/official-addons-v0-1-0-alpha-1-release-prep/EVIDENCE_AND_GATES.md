# Official Addons v0.1.0-alpha.1 Release Prep - Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Baseline

- Current branch: `main`, ahead of origin by two commits before release prep.
- Current head before release prep: `6e1beaa feat(metadata-scraper): add bangumi provider baseline`.
- Nako core metadata source: `../nako/Cargo.toml`.
- Protocol crate source: published `nako-addon-protocol` crate
  `0.1.0-alpha.1`.
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
  and a registry dependency on `nako-addon-protocol` `^0.1.0-alpha.1`.
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
- Docker build command passed after the protocol crate was published:
  `docker buildx build -f addons/metadata-scraper/Dockerfile -t
  nako-metadata-scraper:0.1.0-alpha.1-release-prep --load .`.
- Built image:
  `sha256:c7967e6da7bdad644427474e10e95c74c7dd15a67774d9fef4d422509a7c6888`
  size `91440447` bytes.
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
- `cargo publish -p nako-metadata-scraper --locked --dry-run --allow-dirty`
  passed: packaged 18 files and verified the crate against crates.io
  dependencies.
- `docker buildx build -f addons/metadata-scraper/Dockerfile -t
  nako-metadata-scraper:0.1.0-alpha.1-release-prep --load .` passed.
- Temporary Docker container `/manifest.json` smoke passed with version
  `0.1.0-alpha.1` and protocol version `0.1.0-alpha.1`.
- `git diff --check` passed with only the existing `Cargo.lock` LF/CRLF warning.

Follow-ons:

- Tag and push `v0.1.0-alpha.1` after review.
- Build and push the production container image after deciding the registry
  namespace and tags.
- Re-run live Nako Admin-mediated smoke with a real local Nako server and
  `NAKO_ADMIN_TOKEN`.
