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
