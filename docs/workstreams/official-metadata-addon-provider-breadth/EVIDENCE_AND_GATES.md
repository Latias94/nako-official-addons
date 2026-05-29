# Official Metadata Addon Provider Breadth and Localization — Evidence And Gates

Status: Complete
Last updated: 2026-05-24

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
- `crates/nako-metadata-scraper/src/engine/title.rs`
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

## 2026-05-23 OMPB-030

Review result:

- Workstream compliance: PASS with no blocking findings. Provider-local alias and localized
  title semantics stay in TMDB, Bangumi, and ranking; routes and browser-worker behavior were not
  expanded for this task.
- Code quality: PASS with no blocking findings. Ranking consumes a provider-neutral
  `alternate_titles` fact, shared title normalization lives in `engine::title`, TMDB enriches
  candidates from the official alternative titles response, and Bangumi extracts title-like values
  from localized fields and infobox aliases.
- Residual risk: broader provider localization, provider-specific transliteration, and non-empty
  multi-search merging remain follow-on work.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`: PASS, 21
  tests passed and 43 skipped. Proves TMDB, Bangumi, ranking, and shared title normalization cover
  alternate-title evidence plus raw-title-empty normalized search fallback.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 64 tests passed. Proves the
  package-level metadata scraper surface remains compatible with alternate-title facts and search
  fallback behavior.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable for the whole workspace.
- `git diff --check`: PASS. Proves the current diff has no whitespace errors.

## 2026-05-23 OMPB-040 Closeout

Review result:

- Workstream compliance: PASS with no blocking findings. OMPB-010 through OMPB-030 are complete,
  and remaining provider breadth is recorded as follow-on scope instead of being kept implicit.
- Code quality: PASS with no blocking findings. The lane leaves provider-specific semantics inside
  providers and ranking, keeps routes thin, and avoids copying reference repository code.
- Residual risk: provider-specific transliteration, non-empty multi-search merging, Douban/browser
  automation, and host task orchestration need separate workstreams before implementation.

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper artwork --no-fail-fast`: PASS. Proves artwork
  candidate selection behavior remains covered at closeout.
- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`: PASS, 21
  tests passed and 43 skipped. Proves the provider breadth and title-normalization slice.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 64 tests passed. Proves the
  package-level metadata scraper behavior after lane closeout changes.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final lane diff has no whitespace errors.

## 2026-05-24 Final Release Audit

Fresh gates:

- `cargo nextest run -p nako-metadata-scraper tmdb bangumi ranking title --no-fail-fast`: PASS, 27
  tests passed and 43 skipped. Proves the provider breadth, alternate-title ranking, title
  normalization, search merge, and degraded candidate behavior in the current final state.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: PASS, 70 tests passed. Proves the
  package-level metadata scraper surface.
- `cargo nextest run --workspace --no-fail-fast`: PASS, 70 tests passed. Proves the full workspace
  test suite remains green for the current release scope.
- `cargo fmt --all -- --check`: PASS. Proves Rust formatting is stable.
- `git diff --check`: PASS. Proves the final diff has no whitespace errors.
