# Official Addons Cross-Repo Fearless Refactor - Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Standing Gates

Prefer focused nextest filters during implementation. Broaden only after the
touched slice is stable.

Common gates:

```powershell
git status --short --branch
cargo fmt --all -- --check
git diff --check
```

Protected-write alignment gates:

```powershell
cargo nextest run -p nako-addon-client --no-fail-fast
cargo nextest run -p nako-metadata-scraper writeback artwork --no-fail-fast
```

Provider adapter gates:

```powershell
cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast
cargo nextest run -p nako-metadata-scraper douban browser_worker ranking title --no-fail-fast
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

Task smoke gates:

```powershell
pwsh -File addons/metadata-scraper/smoke.local.ps1 -SidecarBaseUrl http://127.0.0.1:9100
pwsh -NoProfile -ExecutionPolicy Bypass -File ../nako/scripts/official-addon-e2e-smoke.ps1
```

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-24 | OACR-010 | Opened this lane after local architecture review found protected-write client duplication, broad Bangumi/Douban adapters, and missing official Addon Task path smoke. | Pass |
| 2026-05-24 | OACR-030 | Split Bangumi provider into facade plus `client`, `enrichment`, `search`, `parser`, `mapper`, and `test_support` modules. | Pass |
| 2026-05-24 | OACR-030 | `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast` | Pass: 49 passed, 94 skipped |
| 2026-05-24 | OACR-030 | `cargo fmt --all -- --check` | Pass |
| 2026-05-24 | OACR-030 | `git diff --check -- crates/nako-metadata-scraper/src/providers/bangumi.rs crates/nako-metadata-scraper/src/providers/bangumi` | Pass |
| 2026-05-24 | OACR-040 | Split Douban provider into facade plus `client`, `enrichment`, `parser`, `mapper`, and `test_support` modules. | Pass |
| 2026-05-24 | OACR-040 | `cargo nextest run -p nako-metadata-scraper douban browser_worker ranking title --no-fail-fast` | Pass: 30 passed, 113 skipped |
| 2026-05-24 | OACR-040 | `cargo fmt --all -- --check` | Pass |
| 2026-05-24 | OACR-040 | `git diff --check -- crates/nako-metadata-scraper/src/providers/douban.rs crates/nako-metadata-scraper/src/providers/douban` | Pass |
| 2026-05-24 | OACR-020 | Added protected-write runtime DTOs to `../nako/crates/nako-addon-protocol`, runtime client helpers to `../nako/crates/nako-addon-client`, and replaced metadata scraper private runtime DTO/client duplication with a thin facade. | Pass |
| 2026-05-24 | OACR-020 | `cargo nextest run -p nako-addon-client runtime --no-fail-fast` in `../nako` | Pass: 6 passed, 8 skipped |
| 2026-05-24 | OACR-020 | `cargo nextest run -p nako-addon-protocol protected_write_payload_contracts_keep_wire_shape --no-fail-fast` in `../nako` | Pass: 1 passed, 10 skipped |
| 2026-05-24 | OACR-020 | `cargo nextest run -p nako-metadata-scraper nako_runtime writeback artwork --no-fail-fast` | Pass: 10 passed, 130 skipped |
| 2026-05-24 | OACR-020 | `git diff --check -- Cargo.lock crates/nako-addon-client crates/nako-addon-protocol crates/nako-server/src/app/addons.rs crates/nako-server/src/app/addons/task_runtime.rs scripts/official-addon-e2e-smoke.ps1` in `../nako` | Pass |
| 2026-05-24 | OACR-020 | `git diff --check -- Cargo.toml Cargo.lock README.md addons/metadata-scraper/README.md addons/metadata-scraper/smoke.local.ps1 crates/nako-metadata-scraper/Cargo.toml crates/nako-metadata-scraper/src/nako_runtime.rs crates/nako-metadata-scraper/src/engine/bulk.rs crates/nako-metadata-scraper/src/engine/mod.rs crates/nako-metadata-scraper/src/providers/bangumi.rs crates/nako-metadata-scraper/src/providers/bangumi crates/nako-metadata-scraper/src/providers/douban.rs crates/nako-metadata-scraper/src/providers/douban` | Pass |
| 2026-05-24 | OACR-050 | Extended `addons/metadata-scraper/smoke.local.ps1` with `-RunTaskPath`, task-run polling, routing-plan creation, direct dispatch request creation, and bounded result assertions for `bulk-metadata-scrape`. Updated root and addon docs plus `../nako/scripts/official-addon-e2e-smoke.ps1`. | Pass |
| 2026-05-24 | OACR-050 | PowerShell parser checks for `addons/metadata-scraper/smoke.local.ps1` and `../nako/scripts/official-addon-e2e-smoke.ps1` | Pass |
| 2026-05-24 | OACR-050 | `cargo nextest run -p nako-metadata-scraper bulk task routes manifest --no-fail-fast` | Pass: 13 passed, 127 skipped |
| 2026-05-24 | OACR-050 | `cargo nextest run -p nako-server addon_task_run_direct_dispatch --no-fail-fast` in `../nako` | Pass: 6 passed, 281 skipped |
| 2026-05-24 | OACR-060 | `cargo nextest run -p nako-metadata-scraper bangumi douban browser_worker ranking title --no-fail-fast` | Pass: 52 passed, 88 skipped |
| 2026-05-24 | OACR-060 | `cargo fmt --all -- --check` | Pass |

## Known Constraints

- `nako-official-addons` is on `main` and ahead of origin. Do not revert or
  reset unrelated changes.
- `../nako` is private, on `main`, ahead of origin, and has unrelated modified
  files from active work. Do not restore, checkout, reset, stash, or format
  unrelated files.
- Main-repo `addon-outbound-task-dispatch-credentials` was dirty during this
  lane. This work only made direct, compile-required match-arm updates in
  overlapping server files caused by the public client error expansion.
- Reference code under `F:/SourceCodes/Rust/repo-ref/nako-scraper` is for
  product and boundary research only. Do not copy or port implementation code,
  schemas, fixtures, tests, artwork, or generated files.
- Live Docker/server smoke was not executed in this session. The script path is
  ready and covered by parser checks plus focused Rust integration tests.
