# Official Addon Contract Smoke Provider Hardening - Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

This workstream follows the completed
`official-addons-cross-repo-fearless-refactor` lane.

Confirmed existing decision:

- `../nako/docs/adr/0033-version-addon-protocol-independently-from-addon-and-crate-releases.md`
  already separates Addon Version, Addon Protocol Version, and Rust crate
  package version. Do not duplicate that decision here.

Selected work:

- OACSH-020: remove local task envelope mirrors from metadata scraper and use
  public `nako-addon-protocol` types. Done.
- OACSH-030: harden live official smoke harness for resource + task path. Done.
- OACSH-040: move provider capability/config/diagnostic/schema declarations
  toward provider-owned descriptors. Done.

Explicitly deferred:

- Whether future official capabilities should be one sidecar, a bundled suite,
  or multiple installable addons. The current install model is sidecar-based,
  so splitting too finely may hurt operator experience.

## Active Task

- None. This workstream is closed.

## Recommended Execution Order

1. OACSH-020 task envelope contract unification. Done.
2. OACSH-030 live smoke harness. Done.
3. OACSH-040 provider descriptor boundary. Done.
4. OACSH-050 closeout. Done.

OACSH-020 and OACSH-040 both touch metadata scraper internals, so serialize
them. OACSH-030 touches scripts/docs and can be parallelized only after
OACSH-020 keeps the task endpoint shape stable.

## Evidence

OACSH-020:

- `cargo nextest run -p nako-metadata-scraper bulk task routes manifest --no-fail-fast`
  passed: 13 passed, 127 skipped.
- `cargo fmt --all -- --check` passed.
- Path-scoped `git diff --check` passed for task envelope files and this
  workstream.

OACSH-030:

- PowerShell parser checks passed for both smoke scripts.
- `../nako/scripts/official-addon-e2e-smoke.ps1 -PreflightOnly` passed.
- `smoke.local.ps1 -RunTaskPath` without `-RegisterInNako` failed early with
  the expected diagnostic, proving Nako-owned smoke paths cannot be silently
  skipped.
- `cargo nextest run -p nako-metadata-scraper bulk task routes manifest --no-fail-fast`
  passed: 13 passed, 127 skipped.
- Full Docker/server live smoke was not executed in this session.

OACSH-040:

- `cargo nextest run -p nako-metadata-scraper provider registry manifest config --no-fail-fast`
  passed: 90 passed, 50 skipped.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed: 138
  passed, 2 skipped.
- `cargo fmt --all -- --check` passed.
- Path-scoped `git diff --check` passed.

## Module Map

- `providers/mod.rs`: composes `provider_catalog()` from provider-owned catalog
  entries.
- `providers/registry.rs`: owns `ProviderCatalogEntry`,
  `ProviderBuildStatus`, diagnostics, schema property derivation, and secret
  reference derivation.
- `providers/{fixture,tmdb,bangumi,browser_worker,douban}.rs`: each exposes
  `catalog_entry()` with id, capabilities, secret reference if any, and build
  hook.
- `manifest.rs`: consumes `ProviderRegistry` provider schema and secret
  reference helpers instead of knowing provider-specific secret declarations.

## Dirty Worktree Notes

- `nako-official-addons` currently contains completed but uncommitted changes
  from `official-addons-cross-repo-fearless-refactor`.
- `../nako` currently contains unrelated dirty server/library/playback and
  workstream files. Treat unrelated changes as protected.

## Blockers

- None for OACSH-020.
- None for the closed code/doc lane.

## Residual Risks

- Full Docker/server live smoke still needs to run as a release-time proof.
- Future official addon-family splitting remains a product/design lane because
  current installation is sidecar-based and too many installable sidecars may
  hurt operator experience.

## Follow-Ons Not In This Lane

- Official addon-family packaging/install UX decision.
- New official subtitle, NFO, artwork, AI, or diagnostics addon
  implementation.
- Addon Manager lifecycle, signing, catalog source selection, or package
  update policy.
