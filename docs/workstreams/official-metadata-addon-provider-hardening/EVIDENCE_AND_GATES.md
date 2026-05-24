# Official Metadata Addon Provider Hardening — Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper provider_http_runtime config routes --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper provider_http_runtime config routes --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo nextest run --workspace --no-fail-fast
```

Use the broader closeout gate only when the package gate is not enough to prove the lane.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record blocking findings,
missing gates, and residual risks here or link to the review note.

## Evidence Anchors

- `docs/workstreams/official-metadata-addon-provider-hardening/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-hardening/TODO.md`
- `docs/workstreams/official-metadata-addon-provider-hardening/MILESTONES.md`
- `crates/nako-metadata-scraper/src/config.rs`
- `crates/nako-metadata-scraper/src/providers/http_runtime.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/routes.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

## 2026-05-23 Closeout Verification

Review result:

- Workstream compliance: PASS with no blocking findings. OMPH-020 surfaced the
  provider proxy policy seam, and OMPH-030 deepened ranking by considering
  `original_title` and `sort_title` from the patch surface.
- Code quality: PASS with no blocking findings. Route handlers only render
  boolean policy state; proxy URL parsing and transport use remain in config
  and provider HTTP runtime seams; ranking depth stays in the ranking module.
- Residual risk: broader provider-quality breadth such as alias expansion,
  localized coverage, or artwork-selection nuance can still be split into a
  follow-on if needed.

Closeout result: PASS. The lane can be closed and breadth follow-on work can be
split separately.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 57 tests
  passed. Proves proxy-aware TMDB/Bangumi config parsing, provider runtime
  proxy propagation through `new(...)`, redaction-safe health diagnostics, and
  alternate-title-aware ranking.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable for the
  current workspace.
- `git diff --check`: PASS. Proves the current diff has no whitespace errors.

Fresh closeout re-run on 2026-05-23 matched the same gate results above.

Fresh verification is required before marking a task, Codex goal, or lane complete.

## 2026-05-24 Provider Config Boundary Normalization Addendum

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb_config_trims_network_boundary_values bangumi_config_trims_network_boundary_values --no-fail-fast`: expected RED before implementation, then PASS 2. Proves TMDB and Bangumi env-derived provider network values trim boundary whitespace before building auth headers, base endpoints, language parameters, and User-Agent policy.
- `cargo nextest run -p nako-metadata-scraper config http_runtime tmdb bangumi ranking title --no-fail-fast`: PASS 82. Proves provider config normalization composes with registry/config tests, shared HTTP runtime behavior, TMDB/Bangumi request construction, ranking, title variants, degraded candidates, and payload resilience.
