# Official Metadata Addon Non-AV Provider Expansion - Handoff

Status: Complete
Last updated: 2026-05-27

## Current Phase

OMANV-020 TMDB TV foundation is implemented and passing focused tests. OMANV-030 season/episode
was reviewed and deferred because the current metadata patch contract cannot represent typed
series/season/episode hierarchy without lossy tags. OMANV-040 Bangumi enrichment, OMANV-050
BrowserWorker recipe layer, and OMANV-060 AniList first-wave provider are implemented and passing
focused tests. OMANV-070 split TVDB/MAL/IMDb into explicit follow-ups. OMANV-080 closed the lane
after full package, format, JSON, and diff hygiene gates passed.

## Current Provider Map

- TMDB is provider-owned and split into `client`, `enrichment`, `mapper`, `parser`, `search`, and
  `test_support` modules. Existing behavior is movie-centric.
- Bangumi is provider-owned and split into the same client/enrichment/mapper/parser/search shape.
- Douban uses rendered-page support and is useful evidence for BrowserWorker recipe generalization.
- BrowserWorker maps explicit rendered URL results and generic rendered metadata recipes.
- AniList is API-backed and maps anime title variants, description, dates, episodes, runtime,
  genres, tags, studios, score, artwork, AniList IDs, MAL IDs, and AniList URLs.

## Execution Notes

- Keep `tmdb_id` movie-compatible. Add explicit `tmdb_tv_id` for TV direct lookup.
- Do not force episode fields into metadata until the protocol has typed hierarchy fields.
- Bangumi subject relations are available through `/v0/subjects/{subject_id}/subjects`, but native
  relation mapping should wait for typed candidate/protocol relation support.
- BrowserWorker now has two paths: `browser_worker_url` for text extraction and
  `browser_worker_recipe_url` for rendered HTML recipe extraction.
- TVDB should wait for typed episodic protocol fields before native season/episode mapping.
- MAL remains a follow-up unless its official API adds fields not already covered by AniList's MAL
  crosswalk.
- IMDb should begin as a BrowserWorker recipe proof rather than native scraping unless a stable legal
  API path is selected.

## Next Action

Open a fresh follow-up workstream if we want to pursue TVDB episodic metadata, native MAL, or IMDb
rendered recipes. Do not reopen this lane for those provider families.
