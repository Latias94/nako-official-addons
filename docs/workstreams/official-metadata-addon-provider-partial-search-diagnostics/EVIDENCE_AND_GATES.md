# Official Metadata Addon Provider Partial Search Diagnostics — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper tmdb_provider_preserves_search_results_when_later_title_variant_search_fails --no-fail-fast
```

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-metadata-scraper tmdb_provider_preserves_search_results_when_later_title_variant_search_fails tmdb_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast
cargo nextest run -p nako-metadata-scraper bangumi_provider_preserves_search_results_when_later_title_variant_search_fails bangumi_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast
cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Workspace Gate

```bash
cargo nextest run --workspace --no-fail-fast
```

### Hygiene Gates

```bash
cargo fmt --all -- --check
git diff --check
```

## Evidence Anchors

- `docs/workstreams/official-metadata-addon-provider-partial-search-diagnostics/DESIGN.md`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## Evidence Log

### 2026-05-24 — OMPSD-010 Scope And Diagnostic Policy

- Workstream opened to surface redaction-safe partial title-variant search diagnostics through existing provider evidence.
- Public payload shape, HTTP runtime policy, and live provider network checks remain out of scope.

### 2026-05-24 — OMPSD-020 TMDB Partial Search Provider Notes

- `cargo nextest run -p nako-metadata-scraper tmdb_provider_preserves_search_results_when_later_title_variant_search_fails --no-fail-fast`: expected RED before implementation. Proved preserved TMDB candidates did not yet surface partial title-variant search diagnostics.
- `cargo nextest run -p nako-metadata-scraper tmdb_provider_preserves_search_results_when_later_title_variant_search_fails tmdb_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast`: PASS 2. Proves TMDB preserved candidates include a safe partial-search note while all-search-failed behavior still propagates a provider error.

### 2026-05-24 — OMPSD-030 Bangumi Partial Search Provider Notes

- `cargo nextest run -p nako-metadata-scraper bangumi_provider_preserves_search_results_when_later_title_variant_search_fails --no-fail-fast`: expected RED before implementation. Proved preserved Bangumi candidates did not yet surface partial title-variant search diagnostics.
- `cargo nextest run -p nako-metadata-scraper bangumi_provider_preserves_search_results_when_later_title_variant_search_fails bangumi_provider_propagates_error_when_all_title_variant_searches_fail --no-fail-fast`: PASS 2. Proves Bangumi preserved candidates include a safe partial-search note while all-search-failed behavior still propagates a provider error.

### 2026-05-24 — OMPSD-040 Closeout

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`: PASS 46. Proves partial-search diagnostics compose with TMDB/Bangumi provider behavior, ranking evidence, title variants, direct lookup, relevance-budget selection, and degraded fallback behavior.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 89. Proves the package-level metadata scraper surface after provider partial-search diagnostics.
- `cargo nextest run --workspace --no-fail-fast`: PASS 89. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the diff has no whitespace errors before closeout documentation.
