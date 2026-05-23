# Official Metadata Addon Provider Breadth and Localization — Evidence And Gates

Status: Active
Last updated: 2026-05-23

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper artwork --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper artwork --no-fail-fast
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

- `docs/workstreams/official-metadata-addon-provider-breadth/DESIGN.md`
- `docs/workstreams/official-metadata-addon-provider-breadth/TODO.md`
- `docs/workstreams/official-metadata-addon-provider-breadth/MILESTONES.md`
- `crates/nako-metadata-scraper/src/engine/artwork.rs`
- `crates/nako-metadata-scraper/src/engine/mod.rs`
- `crates/nako-metadata-scraper/src/engine/ranking.rs`
- `crates/nako-metadata-scraper/src/providers/tmdb.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`

## Notes

Record what each gate proves. Do not list commands without explaining the behavior they cover.

Fresh verification is required before marking a task, Codex goal, or lane complete.

## 2026-05-23 OMPB-020

Review result:

- Workstream compliance: PASS with no blocking findings. Artwork candidate selection now
  chooses the best matching poster/backdrop across all candidates instead of the first
  matching one.
- Code quality: PASS with no blocking findings. The selection policy stays inside
  `engine::artwork`; route handlers only consume the result.
- Residual risk: provider-local alias and localized title breadth remains as the next slice.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper artwork --no-fail-fast`: PASS, 8 tests
  passed and 51 skipped. Proves artwork candidate selection chooses higher-confidence
  candidates and uses resolution as a deterministic tiebreaker.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 59 tests passed.
  Proves the artwork selection change did not regress the package surface.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable after the change.
- `git diff --check`: PASS. Proves the current diff has no whitespace errors.
