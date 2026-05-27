# Official Metadata Addon Non-AV Provider Expansion - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| TMDB provider | `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast` | Pass | 2026-05-27: 38 passed, 222 skipped. Movie regression plus TV additions. |
| Bangumi provider | `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast` | Pending | Anime enrichment. |
| BrowserWorker rendered | `cargo nextest run -p nako-metadata-scraper browser_worker rendered --no-fail-fast` | Pending | Recipe layer. |
| AniList/config/manifest | `cargo nextest run -p nako-metadata-scraper anilist config registry manifest --no-fail-fast` | Pending | First new provider. |
| Full package | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Pending | Regression gate. |
| Rust fmt | `cargo fmt -p nako-metadata-scraper -- --check` | Pending | Formatting gate. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-non-av-provider-expansion/WORKSTREAM.json` | Pending | Workstream metadata validity. |
| Diff hygiene | `git diff --check` | Pending | Whitespace hygiene. |

## Evidence Log

- 2026-05-27: Opened lane for non-AV provider expansion: TMDB TV/season/episode, Bangumi
  enrichment, BrowserWorker recipes, and first-wave new provider implementation.
- 2026-05-27: OMANV-020 landed TMDB TV foundation in the working tree: `tmdb_tv` external ID
  capability, direct TV lookup, IMDb find fallback to TV when no movie match exists, TV search
  fallback after empty movie search, TV detail/external IDs/alternative titles mapping, TV artwork,
  score/vote facts, and redaction-safe provider outcomes.
- 2026-05-27: `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast` passed: 38 passed,
  222 skipped.
- 2026-05-27: `cargo nextest run -p nako-metadata-scraper config registry manifest routes tmdb --no-fail-fast`
  passed: 79 passed, 181 skipped.
