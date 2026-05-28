# Official Media Extension Addons - TODO

Status: Active
Last updated: 2026-05-28

## M0 - Scope And Evidence Freeze

- [x] OMEA-010 [owner=codex] [deps=none] [scope=docs/workstreams/official-media-extension-addons]
  Goal: Freeze the official Subtitle Provider, DLNA Renderer, and External
  Acquisition Runner follow-on boundaries.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, HANDOFF.md, and FOLLOW_ON_CONTRACTS.md exist and agree.
  Evidence: docs/workstreams/official-media-extension-addons/DESIGN.md
  Handoff: DONE 2026-05-28. Proceed with OMEA-020.

## M1 - Read-Only Subtitle Provider Foundation

- [x] OMEA-020 [owner=codex] [deps=OMEA-010] [scope=crates/nako-subtitle-provider,addons/subtitle-provider,Cargo.toml,README.md]
  Goal: Add a fixture-backed official subtitle provider sidecar with a valid
  `AddonResource::Subtitle` manifest, `/subtitle`, `/manifest.json`, `/health`,
  diagnostics, checked-in manifest example, and local smoke script.
  Validation: `cargo nextest run -p nako-subtitle-provider --no-fail-fast`;
  `cargo check -p nako-subtitle-provider --tests`; `cargo fmt --all -- --check`;
  `git diff --check`.
  Review: The addon is read-only and must not write subtitle files or call live
  providers.
  Evidence: crates/nako-subtitle-provider/src/manifest.rs
  Handoff: DONE 2026-05-28. Added fixture-backed read-only subtitle provider,
  checked-in manifest, packaging docs, and local smoke.

## M2 - Plan-Only DLNA Renderer Foundation

- [x] OMEA-030 [owner=codex] [deps=OMEA-010] [scope=crates/nako-dlna-renderer,addons/dlna-renderer,Cargo.toml,README.md]
  Goal: Add an official DLNA renderer adapter foundation that reuses the
  `renderer_adapter` protocol, supports manual target discovery, and returns
  redaction-safe plan-only command results without live SSDP/UPnP control.
  Validation: `cargo nextest run -p nako-dlna-renderer --no-fail-fast`;
  `cargo check -p nako-dlna-renderer --tests`; `cargo fmt --all -- --check`;
  `git diff --check`.
  Review: No live network discovery or control is implemented in this task.
  Evidence: crates/nako-dlna-renderer/src/manifest.rs
  Handoff: DONE 2026-05-28. Added plan-only DLNA renderer foundation with
  manual target discovery, command envelope validation, redaction-safe
  diagnostics, checked-in manifest, packaging docs, and local smoke script.

## M3 - Acquisition Runner Follow-On Contract

- [x] OMEA-040 [owner=codex] [deps=OMEA-010] [scope=docs/workstreams/official-media-extension-addons/FOLLOW_ON_CONTRACTS.md,README.md]
  Goal: Record External Acquisition Runner as a future action addon contract
  without adding runtime behavior.
  Validation: `git diff --check`; docs name the non-goals explicitly.
  Review: The contract must not imply resource-search can execute downloads.
  Evidence: docs/workstreams/official-media-extension-addons/FOLLOW_ON_CONTRACTS.md
  Handoff: DONE 2026-05-28. README and FOLLOW_ON_CONTRACTS document the
  future action-addon envelope, resource-search separation, no downloader
  execution, no cloud-drive transfer, and no durable password/code storage.

## M4 - Catalog Sync Follow-On Decision

- [x] OMEA-050 [owner=planner] [deps=OMEA-020,OMEA-030] [scope=docs/workstreams/official-media-extension-addons]
  Goal: Decide whether to sync `../nako` official addon catalog in this lane or
  split it after the addon manifests stabilize.
  Validation: HANDOFF.md records the decision and exact target repo.
  Review: Do not modify `../nako/web`.
  Evidence: HANDOFF.md
  Handoff: DONE 2026-05-28. Synced `../nako` core catalog only, with no web
  changes. Nako commit `52da469d` adds shared catalog facts and server catalog
  resolve coverage for `nako.official.subtitle-provider` and
  `nako.official.dlna-renderer`.

## M5 - Closeout

- [ ] OMEA-060 [owner=codex] [deps=OMEA-020,OMEA-030,OMEA-040,OMEA-050] [scope=docs/workstreams/official-media-extension-addons]
  Goal: Run fresh gates, update evidence, and close or split remaining work.
  Validation: final package gates pass or blockers are concrete.
  Review: No blocking workstream or code-quality findings remain.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md
  Handoff: Summarize remaining risks and next addon priorities.
