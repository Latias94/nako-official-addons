# Official Addons Cross-Repo Fearless Refactor - Handoff

Status: Complete
Last updated: 2026-05-24

## Current State

This workstream completed three official-addon fearless refactors:

- align protected-write host/client responsibility with Nako public addon
  client/protocol crates;
- split Bangumi and Douban provider adapters into deep provider-local modules;
- extend official smoke coverage to prove host-dispatched Addon Task path
  execution for `bulk-metadata-scrape`.

The workstream was opened from local review only. Reference code under
`F:/SourceCodes/Rust/repo-ref/nako-scraper` was used for product-boundary
thinking only; implementation code was not copied.

OACR-020 is complete. Protected-write runtime DTOs and submission helpers now
live in public Nako addon protocol/client crates, while the metadata scraper
keeps a sidecar-local facade for configuration and fake transport tests:

- `../nako/crates/nako-addon-protocol/src/lib.rs`: protected-write permission,
  target, access-check, side-effect, metadata-write, artwork-write, response,
  and summary wire DTOs.
- `../nako/crates/nako-addon-client/src/lib.rs`: `NakoRuntimeClient`,
  `NakoRuntimeClientConfig`, bearer-header runtime calls, request-body token
  guard, safe client error codes, and version-tolerant JSON response parsing.
- `crates/nako-metadata-scraper/src/nako_runtime.rs`: thin facade and public
  type aliases over the shared client/protocol contracts.

OACR-030 is complete. Bangumi now follows the same provider-local module
shape as TMDB:

- `bangumi.rs`: public provider facade and existing behavior tests.
- `bangumi/client.rs`: construction, runtime injection, endpoint building, and
  bearer header construction.
- `bangumi/enrichment.rs`: direct lookup, title-variant search orchestration,
  detail fetch, ranking handoff, and degraded candidate fallback.
- `bangumi/search.rs`: Bangumi external ID extraction, air-date filter, and
  enrichment budget/ranking helpers.
- `bangumi/parser.rs`: Bangumi request/response DTOs and deserialization
  guards.
- `bangumi/mapper.rs`: subject-to-candidate mapping, provider notes, title
  normalization, year parsing, genres, and artwork candidates.
- `bangumi/test_support.rs`: fake HTTP transport shared by Bangumi tests.

OACR-040 is complete. Douban now uses the same facade-plus-local-modules
shape:

- `douban.rs`: public provider facade and browser-worker contract test.
- `douban/client.rs`: construction, render endpoint construction, Douban
  search URL encoding, browser-worker render request DTO, and render response
  validation.
- `douban/enrichment.rs`: search render, result limit, detail render, parsing
  handoff, and candidate collection.
- `douban/parser.rs`: rendered search HTML parsing, detail HTML parsing, field
  extraction, date/year/rating/vote/runtime normalization.
- `douban/mapper.rs`: Douban detail facts to metadata/artwork candidate mapping.
- `douban/test_support.rs`: fake HTTP transport and rendered-page fixtures for
  Douban tests.

OACR-050 is complete. The official smoke scripts can now exercise
host-dispatched `bulk-metadata-scrape` task execution:

- `addons/metadata-scraper/smoke.local.ps1`: `-RunTaskPath`, routing-plan
  creation, direct task-run creation, polling, terminal status assertions, and
  bounded output assertions.
- `../nako/scripts/official-addon-e2e-smoke.ps1`: forwards the task-path gate.
- `README.md` and `addons/metadata-scraper/README.md`: document the task smoke
  flag and expectations.

## Active Task

- None. This workstream is closed.

## Recommended Execution Order

1. OACR-030 Bangumi adapter split. Done.
2. OACR-040 Douban adapter split. Done.
3. OACR-020 protected-write client alignment. Done.
4. OACR-050 official Addon Task path smoke. Done.
5. OACR-060 closeout. Done.

## Dirty Worktree Notes

At current time in this repository:

- `main` is ahead of `origin/main`.
- OACR-030 edited Bangumi provider files and added provider-local Bangumi
  submodules.
- OACR-040 edited Douban provider files and added provider-local Douban
  submodules.
- OACR-020 edited workspace dependency wiring, metadata scraper runtime facade,
  and bulk-engine error test fixtures.
- OACR-050 edited smoke scripts and README documentation.

OACR-030 evidence:

- `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast`
  passed: 49 passed, 94 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check -- crates/nako-metadata-scraper/src/providers/bangumi.rs crates/nako-metadata-scraper/src/providers/bangumi`
  passed.

OACR-040 evidence:

- `cargo nextest run -p nako-metadata-scraper douban browser_worker ranking title --no-fail-fast`
  passed: 30 passed, 113 skipped.
- `cargo fmt --all -- --check` passed.
- `git diff --check -- crates/nako-metadata-scraper/src/providers/douban.rs crates/nako-metadata-scraper/src/providers/douban`
  passed.

OACR-020 evidence:

- `cargo nextest run -p nako-addon-client runtime --no-fail-fast` in `../nako`
  passed: 6 passed, 8 skipped.
- `cargo nextest run -p nako-addon-protocol protected_write_payload_contracts_keep_wire_shape --no-fail-fast`
  in `../nako` passed: 1 passed, 10 skipped.
- `cargo nextest run -p nako-metadata-scraper nako_runtime writeback artwork --no-fail-fast`
  passed: 10 passed, 130 skipped.

OACR-050 evidence:

- PowerShell parser checks for both smoke scripts passed.
- `cargo nextest run -p nako-metadata-scraper bulk task routes manifest --no-fail-fast`
  passed: 13 passed, 127 skipped.
- `cargo nextest run -p nako-server addon_task_run_direct_dispatch --no-fail-fast`
  in `../nako` passed: 6 passed, 281 skipped.

OACR-060 evidence:

- `cargo nextest run -p nako-metadata-scraper bangumi douban browser_worker ranking title --no-fail-fast`
  passed: 52 passed, 88 skipped.
- `cargo fmt --all -- --check` passed in this repository.

At recent review time in `../nako`:

- `main` was ahead of `origin/main`.
- Several server/library/playback and workstream files were modified by other
  work. This lane only touched `../nako` server files where the public
  `AddonClientError` expansion caused direct non-exhaustive match failures.

## Residual Risks

- Live Docker/server smoke was not executed in this session. Run the documented
  `-RunTaskPath` smoke when a local Nako server/admin token is available.
- Publishing should verify the path-plus-version dependency setup against the
  intended private/public crate release flow.

## Follow-Ons Not In This Lane

- Provider catalog/config descriptor seam.
- New official subtitle, NFO, artwork, AI, or network diagnostic addons.
- Package signing, marketplace/source catalog, and Addon Manager lifecycle.
