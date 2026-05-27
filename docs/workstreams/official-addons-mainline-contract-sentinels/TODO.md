# Official Addons Mainline Contract Sentinels - TODO

Status: Complete
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

- [x] OAMC-010 [owner=codex] [deps=none] [scope=docs/workstreams/official-addons-mainline-contract-sentinels]
  Goal: Freeze the cross-repo drift problem, target state, non-goals, and validation anchors.
  Validation: DESIGN.md, MILESTONES.md, EVIDENCE_AND_GATES.md, WORKSTREAM.json agree.
  Evidence: `docs/workstreams/official-addons-mainline-contract-sentinels/DESIGN.md`
  Handoff: This lane is independent of the blocked alpha.2 publish approval lane.

## M1 - Shared Catalog Manifest Locality

- [x] OAMC-020 [owner=codex] [deps=OAMC-010] [scope=crates/nako-notification-bridge/src/manifest.rs,crates/nako-notification-bridge/Cargo.toml]
  Goal: Move notification bridge official manifest facts to `nako-official-addon-catalog` so all three official sidecars share the same catalog authority.
  Validation: `cargo nextest run -p nako-notification-bridge manifest routes --no-fail-fast`.
  Review: Confirm local provider-only facts remain local and official manifest facts come from the catalog.
  Evidence: Pass 2026-05-27; see `EVIDENCE_AND_GATES.md`.
  Handoff: DONE. Provider test-send diagnostics remain sidecar-local; official manifest facts come from the catalog.

## M2 - Cross-Repo CI And Container Sentinels

- [x] OAMC-030 [owner=codex] [deps=OAMC-010] [scope=.github/workflows/release-gate.yml,addons/*/Dockerfile,scripts/smoke_official_addon_container.py,addons/*/compose.example.yml]
  Goal: Make release gate checkout/build all required repositories and smoke all current official sidecars.
  Validation: YAML inspection plus `cargo metadata --format-version 1 --no-deps`, `cargo nextest run --workspace --no-fail-fast`; Docker build if daemon is available.
  Review: Check that Docker build context requirements are explicit and do not rely on unavailable paths.
  Evidence: Pass/blocked 2026-05-27; see `EVIDENCE_AND_GATES.md`.
  Handoff: DONE_WITH_CONCERNS. Release gate and Dockerfiles now use explicit mainline checkout/named context; local Docker daemon was unavailable, so container build/smoke is not claimed locally.

## M3 - Publish Automation Breadth

- [x] OAMC-040 [owner=codex] [deps=OAMC-030] [scope=.github/workflows/crates-publish.yml,docs/workstreams/official-addons-v0-1-0-alpha-2-release-readiness]
  Goal: Update crates.io dry-run automation so it covers every publishable official addon crate in this repository.
  Validation: Workflow inspection and `cargo publish --dry-run` evidence where upstream alpha.2 crates allow it.
  Review: Do not publish. Preserve explicit user approval for real crate publication.
  Evidence: Expected blockers recorded 2026-05-27; see `EVIDENCE_AND_GATES.md`.
  Handoff: DONE_WITH_CONCERNS. Dry-runs still fail on crates.io missing upstream alpha.2 SDK/catalog crates, not on this lane's workflow shape.

## M4 - Verification And Closeout

- [x] OAMC-050 [owner=codex] [deps=OAMC-020,OAMC-030,OAMC-040] [scope=docs/workstreams/official-addons-mainline-contract-sentinels]
  Goal: Run final gates, update evidence, and decide whether to close this lane or split follow-ons.
  Validation: `cargo fmt --all -- --check`, `cargo nextest run --workspace --no-fail-fast`, `git diff --check`.
  Review: review-workstream before closeout if the lane spans CI, Docker, and Rust code.
  Evidence: Pass 2026-05-27; see `EVIDENCE_AND_GATES.md`, `HANDOFF.md`.
  Handoff: DONE. Recommended next lane is mainline protocol/capability drift monitoring after `../nako` lands the current transcode/casting follow-ons.
