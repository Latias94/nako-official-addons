# Official Addons v0.1.0-alpha.1 Release Prep - TODO

Status: Complete
Last updated: 2026-05-23

Task IDs use the `OAREL` prefix.

## M0 - Scope

- [x] OAREL-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-addons-v0-1-0-alpha-1-release-prep]
  Goal: Open release prep workstream with Nako core metadata facts and gates.
  Validation: Workstream docs agree.
  Evidence: Workstream docs.
  Result: DONE 2026-05-23.
  Handoff: Continue with OAREL-020 Cargo metadata.

## M1 - Cargo Metadata And Version

- [x] OAREL-020 [owner=codex] [deps=OAREL-010] [scope=Cargo.toml,crates/nako-metadata-scraper]
  Goal: Set workspace/crate release metadata for `0.1.0-alpha.1`, add readme
  metadata, constrain `nako-addon-protocol`, and add crate README.
  Validation: `cargo metadata --format-version 1 --no-deps`; `cargo nextest run -p nako-metadata-scraper manifest --no-fail-fast`.
  Review: Keep the protocol dependency pinned to the published alpha.1 crate.
  Evidence: Cargo metadata and manifest tests.
  Result: DONE 2026-05-23.
  Evidence: `cargo metadata --format-version 1 --no-deps`;
  `cargo nextest run -p nako-metadata-scraper manifest --no-fail-fast`.
  Handoff: Continued with OAREL-030 license/docs/examples.

## M2 - License And Operator Docs

- [x] OAREL-030 [owner=codex] [deps=OAREL-020] [scope=LICENSE,CHANGELOG.md,README.md,addons/metadata-scraper]
  Goal: Add declared license and changelog files, update
  release/version/protocol docs, and update manifest/compose/systemd examples
  for alpha.1.
  Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`; `git diff --check`.
  Review: Protocol crate dual licensing should be described without relicensing
  the addon workspace.
  Evidence: README, manifest example, and package tests.
  Result: DONE 2026-05-23.
  Evidence: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`;
  `git diff --check`.
  Handoff: Continued with OAREL-040 Docker cargo-chef.

## M3 - Docker cargo-chef Build

- [x] OAREL-040 [owner=codex] [deps=OAREL-030] [scope=addons/metadata-scraper/Dockerfile,addons/metadata-scraper/README.md]
  Goal: Convert Dockerfile to cargo-chef stages for dependency caching while
  preserving sibling `nako/` protocol source in the build context.
  Validation: Dockerfile static review; `cargo nextest run --workspace --no-fail-fast`; `git diff --check`.
  Review: `cargo chef cook` and final `cargo build` must run from the same
  workdir and use the same release target path.
  Evidence: Dockerfile diff and workspace tests.
  Result: DONE 2026-05-23.
  Evidence: `docker buildx build --build-context nako-core=../nako -f
  addons/metadata-scraper/Dockerfile -t
  nako-metadata-scraper:0.1.0-alpha.1-release-prep --load .`; container
  `/manifest.json` smoke.
  Handoff: Continued with OAREL-050 closeout.

## M4 - Closeout

- [x] OAREL-050 [owner=planner] [deps=OAREL-040] [scope=docs/workstreams/official-addons-v0-1-0-alpha-1-release-prep]
  Goal: Run final release-prep gates and close or split follow-ons.
  Validation: `cargo fmt --all -- --check`; `cargo nextest run --workspace --no-fail-fast`; `cargo metadata --format-version 1 --no-deps`; `git diff --check`.
  Review: Record any gate not run with a concrete reason.
  Evidence: EVIDENCE_AND_GATES.md and WORKSTREAM.json.
  Result: DONE 2026-05-23.
  Evidence: final cargo metadata, fmt, package/workspace nextest, Docker build,
  container smoke, and `git diff --check`.
  Handoff: Prepare tag/image release after user approval.
