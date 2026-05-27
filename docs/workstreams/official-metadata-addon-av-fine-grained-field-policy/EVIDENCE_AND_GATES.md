# Official Metadata Addon AV Fine-Grained Field Policy - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| Targeted policy tests | `cargo nextest run -p nako-metadata-scraper provider_field_policy registry_builds_default_av_field_policy registry_builds_quality_score_av_field_policy --no-fail-fast` | Passed | 5 passed, 250 skipped. |
| Full package tests | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Passed | 252 passed, 3 skipped. |
| Rust fmt | `cargo fmt -p nako-metadata-scraper -- --check` | Passed | Formatting gate. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-av-fine-grained-field-policy/WORKSTREAM.json` | Passed | Workstream metadata validity. |
| Diff hygiene | `git diff --check` | Passed | Whitespace hygiene. |

## Evidence Log

- 2026-05-27: Opened follow-up lane to replace coarse AV default field policy groups with explicit
  supported-provider orders and add score/vote fusion.
- 2026-05-27: Replaced default AV field groups with an explicit supported-provider matrix adapted
  from the reference config: title, outline, actor, thumb, poster, extra fanart, trailer, tag,
  release, runtime, score, director, series, studio, publisher, and wanted each have separate
  default order.
- 2026-05-27: Added score/vote fusion through provider-field policy. Request overrides can use
  `score`, `community_score_milli`, or `community_vote_count`; fused results emit redaction-safe
  field-source evidence.
- 2026-05-27: Verification passed:
  - `cargo nextest run -p nako-metadata-scraper provider_field_policy registry_builds_default_av_field_policy registry_builds_quality_score_av_field_policy --no-fail-fast`: 5 passed, 250 skipped.
  - `cargo nextest run -p nako-metadata-scraper --no-fail-fast`: 252 passed, 3 skipped.
  - `cargo fmt -p nako-metadata-scraper -- --check`: passed.
  - `python -m json.tool docs/workstreams/official-metadata-addon-av-fine-grained-field-policy/WORKSTREAM.json`: passed.
  - `git diff --check`: passed.
