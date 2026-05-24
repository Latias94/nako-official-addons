# Official Metadata Addon Bangumi Year Air Date Filter — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_query_year_as_air_date_search_filter --no-fail-fast
```

## Gate Set

### Targeted Iteration Gates

```bash
cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_query_year_as_air_date_search_filter bangumi_provider_omits_air_date_search_filter_when_query_year_is_missing --no-fail-fast
cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast
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

- `docs/workstreams/official-metadata-addon-bangumi-year-air-date-filter/DESIGN.md`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- Official Bangumi OpenAPI `POST /v0/search/subjects` request schema, `filter.air_date`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking the task or lane complete.

## Evidence Log

### 2026-05-24 — OMBYF-010 Scope And Filter Policy

- Workstream opened to map query year into Bangumi official `filter.air_date` search constraints.
- Public payload shape, HTTP runtime policy, and live Bangumi network checks remain out of scope.

### 2026-05-24 — OMBYF-020 Bangumi Air-Date Filter

- `cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_http_runtime_and_maps_subject_candidates --no-fail-fast`: expected RED before implementation, then PASS 1. The RED failure proved query year was not included in Bangumi `filter.air_date`; the PASS proves year `1995` maps to `>=1995-01-01` and `<1996-01-01`.
- `cargo nextest run -p nako-metadata-scraper bangumi_provider_uses_http_runtime_and_maps_subject_candidates bangumi_provider_omits_air_date_search_filter_when_query_year_is_missing --no-fail-fast`: PASS 2. Proves year-bearing queries include `air_date` while yearless queries omit it and preserve existing `type`/`nsfw` filters.

### 2026-05-24 — OMBYF-030 Closeout

- `cargo nextest run -p nako-metadata-scraper bangumi ranking title --no-fail-fast`: PASS 34. Proves Bangumi year air-date filtering composes with Bangumi provider behavior, ranking evidence, and title variants.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS 92. Proves the package-level metadata scraper surface after Bangumi year air-date filtering.
- `cargo nextest run --workspace --no-fail-fast`: PASS 92. Proves the full workspace test suite remains green.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the diff has no whitespace errors before closeout documentation.
