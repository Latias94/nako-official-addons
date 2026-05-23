# Official Metadata Addon Provider Search Merge — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Hygiene Gates

```bash
cargo fmt --all -- --check
git diff --check
```

## Evidence Anchors

- `docs/workstreams/official-metadata-addon-provider-search-merge/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-search-merge/TODO.md`
- `docs/workstreams/official-metadata-addon-provider-breadth`
- `crates/nako-metadata-scraper/src/engine/title.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## 2026-05-24 OMPSM-020

Review result:

- Workstream compliance: PASS with no blocking findings. Search merging stays inside TMDB and
  Bangumi providers; routes, browser-worker, Douban, and host task runtime were not expanded.
- Code quality: PASS with no blocking findings. The implementation keeps the existing provider
  `suggest` seam, dedupes provider IDs before detail enrichment, and keeps the existing three
  candidate enrichment budget explicit.
- Residual risk: provider-specific transliteration and smarter ranking between merged search
  variants remain follow-on work.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast`: PASS, 19 tests
  passed and 48 skipped. Proves TMDB and Bangumi merge non-empty search-title variant results,
  preserve fallback behavior, skip case-only duplicate search keys, and keep provider tests
  synthetic.

## 2026-05-24 OMPSM-030 Closeout

Review result:

- Workstream compliance: PASS with no blocking findings. OMPSM-010 and OMPSM-020 are complete, and
  remaining provider-search work is explicitly deferred.
- Code quality: PASS with no blocking findings. The code keeps the existing provider public seam,
  avoids a premature cross-provider abstraction over different raw result types, and leaves network
  policy in the shared HTTP runtime.
- Residual risk: merged candidates still rely on provider order before ranking. Smarter
  cross-variant ranking should be a separate ranking-focused lane.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast`: PASS, 19 tests
  passed and 48 skipped. Proves the search merge behavior and title variant helper.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 67 tests passed. Proves the
  package-level metadata scraper surface after search merge.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.

## 2026-05-24 Final Release Audit

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi title --no-fail-fast`: PASS, 22 tests
  passed and 48 skipped. Proves the title-variant merge behavior after degraded candidate tests were
  added.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 70 tests passed. Proves the
  package-level metadata scraper surface.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 70 tests passed. Proves the full workspace
  test suite remains green for the current release scope.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.
