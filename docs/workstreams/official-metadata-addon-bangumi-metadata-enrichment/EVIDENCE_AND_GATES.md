# Official Metadata Addon Bangumi Metadata Enrichment — Evidence And Gates

Status: Complete
Last updated: 2026-05-26

## Smallest Current Repro

```bash
cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast
```

## Gate Set

### Targeted Iteration Gate

```bash
cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast
```

### Package Gate

```bash
cargo nextest run -p nako-metadata-scraper --no-fail-fast
```

### Broader Closeout Gate

```bash
cargo fmt --all -- --check
git diff --check
```

Use the package gate instead of a full workspace gate unless changes cross crate
boundaries.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or link to the review
note.

## Evidence Anchors

- `docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment/DESIGN.md`
- `docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment/TODO.md`
- `docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment/MILESTONES.md`
- `crates/nako-metadata-scraper/src/providers/bangumi.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi/parser.rs`
- `crates/nako-metadata-scraper/src/providers/bangumi/mapper.rs`

## Evidence Log

| Date | Gate | Result | Notes |
| --- | --- | --- | --- |
| 2026-05-26 | Reference clone | PASS | `repo-ref/bangumi-api`, `repo-ref/bangumi-server`, and `repo-ref/jellyfin-plugin-bangumi` cloned for local comparison. `repo-ref/` is gitignored. |
| 2026-05-26 | License review | PASS_WITH_CONSTRAINT | `jellyfin-plugin-bangumi` is GPL-2.0; use only for behavior comparison, not source copying. Official Bangumi API/server repos are used for field semantics. |
| 2026-05-26 | `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast` | PASS | 27 Bangumi-related tests passed after parser/mapper enrichment. |
| 2026-05-26 | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | PASS | 153 tests passed, 2 skipped. |
| 2026-05-26 | `cargo fmt --all -- --check` | PASS | Formatting check passed. |
| 2026-05-26 | `python -m json.tool docs/workstreams/official-metadata-addon-bangumi-metadata-enrichment/WORKSTREAM.json` | PASS | Workstream JSON is valid. |
| 2026-05-26 | `git diff --check` | PASS | No whitespace errors. |
| 2026-05-26 | Final closeout review | PASS | No blocking workstream compliance or code quality findings. Scope stayed inside Bangumi parser/mapper/tests/docs. |

## Reference Findings

- Official API exposes `GET /v0/subjects/{subject_id}` and
  `POST /v0/search/subjects` as the current subject detail/search surfaces.
- Official subject responses include optional `nsfw`, `locked`, `volumes`,
  `eps`, `total_episodes`, `air_weekday`, `rating`, `collection`, `images`,
  `meta_tags`, `tags`, and parsed `infobox`.
- Mature media-library mapping commonly uses Bangumi subject facts for
  community rating, homepage, end date, NSFW rating, popular tags, genre-like
  tags, image selection, and year filtering.
- Nako's current metadata patch protocol cannot represent homepage, end date,
  staff, or cast directly. This lane preserves short, safe provider-prefixed
  facts in tags where appropriate. Concrete homepage URLs are intentionally not
  written into `patch.tags`.

## Notes

Record what each gate proves. Do not list commands without explaining the
behavior they cover.

Fresh verification is required before marking a task, Codex goal, or lane
complete.
