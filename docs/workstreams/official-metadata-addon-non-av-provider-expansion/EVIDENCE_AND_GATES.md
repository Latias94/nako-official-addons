# Official Metadata Addon Non-AV Provider Expansion - Evidence And Gates

Status: Complete
Last updated: 2026-05-27

## Planned Gates

| Gate | Command | Status | Notes |
| --- | --- | --- | --- |
| TMDB provider | `cargo nextest run -p nako-metadata-scraper tmdb --no-fail-fast` | Pass | 2026-05-27: 38 passed, 222 skipped. Movie regression plus TV additions. |
| Bangumi provider | `cargo nextest run -p nako-metadata-scraper bangumi --no-fail-fast` | Pass | 2026-05-27: 28 passed, 233 skipped. Anime enrichment. |
| BrowserWorker rendered | `cargo nextest run -p nako-metadata-scraper browser_worker rendered --no-fail-fast` | Pass | 2026-05-27: 23 passed, 240 skipped. Recipe layer. |
| AniList/config/manifest | `cargo nextest run -p nako-metadata-scraper anilist config registry manifest routes --no-fail-fast` | Pass | 2026-05-27: 48 passed, 218 skipped. First new provider plus diagnostics. |
| Full package | `cargo nextest run -p nako-metadata-scraper --no-fail-fast` | Pass | 2026-05-27: 263 passed, 3 skipped. |
| Rust fmt | `cargo fmt -p nako-metadata-scraper -- --check` | Pass | 2026-05-27. |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-non-av-provider-expansion/WORKSTREAM.json` | Pass | 2026-05-27. |
| Diff hygiene | `git diff --check` | Pass | 2026-05-27. |

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
- 2026-05-27: OMANV-060 added AniList as the first new non-AV provider. It
  supports official GraphQL anime search, direct AniList ID lookup, direct MAL
  ID lookup, optional bearer token, proxy configuration, title variants,
  description cleanup, release date/year, episodes, runtime, genres, spoiler-
  filtered tags, studios, score facts, poster/backdrop artwork, AniList/MAL URL
  external IDs, manifest toggles, and redaction-safe provider outcomes.
- 2026-05-27: `cargo nextest run -p nako-metadata-scraper anilist config registry manifest routes --no-fail-fast`
  passed: 48 passed, 218 skipped.
- 2026-05-27: OMANV-070 split TVDB, MAL, and IMDb out of this lane. TVDB should
  wait for typed episodic protocol fields; MAL should remain a follow-up unless
  its API adds fields not already covered by AniList's MAL crosswalk; IMDb
  should start as a BrowserWorker recipe instead of native scraping unless a
  stable legal API path is selected.
- 2026-05-27: OMANV-080 closed the lane after docs and manifest updates.
  `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passed: 263
  passed, 3 skipped.
- 2026-05-27: `cargo fmt -p nako-metadata-scraper -- --check`,
  `python -m json.tool docs/workstreams/official-metadata-addon-non-av-provider-expansion/WORKSTREAM.json`,
  and `git diff --check` passed.
