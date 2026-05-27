# Official Metadata Addon Non-AV Provider Expansion - Evidence And Gates

Status: Active
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| TMDB provider | `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast` | Pass | 2026-05-27: 38 passed, 222 skipped. Movie regression plus TV additions. |
| Bangumi provider | `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast` | Pass | 2026-05-27: 28 passed, 233 skipped. Anime enrichment. |
| BrowserWorker rendered | `cargo nextest run -p nako-metadata-scraper browser_worker rendered --no-fail-fast` | Pass | 2026-05-27: 23 passed, 240 skipped. Recipe layer. |
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
- 2026-05-27: OMANV-030 reviewed TMDB season/episode support against the current protocol. The
  provider can fetch season/episode facts, but `AddonMetadataPatch` has no typed series, season,
  episode, season number, episode number, or episode-still fields, and it denies unknown fields. A
  native implementation is deferred until that hierarchy exists in the protocol instead of encoding
  these facts as tags.
- 2026-05-27: OMANV-040 enriched Bangumi mapping with official SlimSubject fields
  (`short_summary`, top-level score/rank, `collection_total`), explicit `bangumi_id`/`bgm_id`
  direct lookup aliases, subject type labels, and structured infobox-derived credits/studios.
- 2026-05-27: `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast` passed: 28
  passed, 233 skipped.
- 2026-05-27: OMANV-050 added a BrowserWorker rendered-page recipe path using `/render` plus a
  typed generic metadata selector recipe. It maps title, overview, release date/year, runtime,
  genres, tags, poster artwork, score/vote facts, canonical URL, provider outcomes, and a
  `browser_worker_recipe_url` direct lookup alias while preserving the existing text extraction
  path.
- 2026-05-27: `cargo nextest run -p nako-metadata-scraper browser_worker rendered --no-fail-fast`
  passed: 23 passed, 240 skipped.
- 2026-05-27: `cargo nextest run -p nako-metadata-scraper config registry manifest routes browser_worker rendered --no-fail-fast`
  passed: 66 passed, 197 skipped.
