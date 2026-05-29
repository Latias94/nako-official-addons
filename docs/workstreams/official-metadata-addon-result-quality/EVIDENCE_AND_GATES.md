# Official Metadata Addon Result Quality - Evidence And Gates

Status: Complete
Last updated: 2026-05-23

## Baseline

- Current branch: `main`, ahead of origin by one commit before this lane.
- Current head before this lane: `5377503 chore(release): use published addon protocol crate`.

## Gates

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- `cargo fmt --all -- --check`
- `cargo nextest run --workspace --no-fail-fast`
- `git diff --check`

## OMRQ-010

Status: DONE 2026-05-23.

Evidence:

- Workstream opened with a bounded result-quality scope and explicit
  exclusions for Admin Web and protocol drift.

## OMRQ-020

Status: DONE 2026-05-23.

Evidence:

- `cargo nextest run -p nako-metadata-scraper ranking evidence --no-fail-fast`
  passed.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed.
- `cargo nextest run --workspace --no-fail-fast` passed.
- `cargo fmt --all -- --check` passed.
- `git diff --check` remains clean.
- Runtime candidate shaping now deduplicates exact duplicate provider
  candidates, caps the final candidate list, and keeps deterministic ordering
  before response shaping.

## OMRQ-030

Status: DONE 2026-05-23.

Evidence:

- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed with
  36 tests.
- `cargo nextest run --workspace --no-fail-fast` passed with 36 tests.
- `cargo fmt --all -- --check` passed.
- `git diff --check` remains clean.
- TMDB and Bangumi now surface a shared community score and vote count in
  provider facts, and runtime uses that signal as a small generic bonus.
- Protocol contract remains unchanged.

## OMRQ-040

Status: DONE 2026-05-23.

Evidence:

- Root README, addon README, and crate README now describe runtime candidate
  dedupe/capping and the shared community-score ranking bonus.
- `cargo fmt --all -- --check` passed.
- `cargo nextest run --workspace --no-fail-fast` passed with 36 tests.
- `git diff --check` remains clean.
- No protocol contract text was changed.
