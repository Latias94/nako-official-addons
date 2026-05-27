# Official Metadata Addon Non-AV Provider Expansion - Handoff

Status: Active
Last updated: 2026-05-27

## Current Phase

OMANV-020 TMDB TV foundation is implemented and passing focused tests. OMANV-030 season/episode
was reviewed and deferred because the current metadata patch contract cannot represent typed
series/season/episode hierarchy without lossy tags. OMANV-040 Bangumi enrichment is implemented and
passing focused tests. Current task is OMANV-050 BrowserWorker recipe layer.

## Current Provider Map

- TMDB is provider-owned and split into `client`, `enrichment`, `mapper`, `parser`, `search`, and
  `test_support` modules. Existing behavior is movie-centric.
- Bangumi is provider-owned and split into the same client/enrichment/mapper/parser/search shape.
- Douban uses rendered-page support and is useful evidence for BrowserWorker recipe generalization.
- BrowserWorker currently maps explicit rendered URL results; recipe support will make it a safer
  prototype layer for HTML-heavy providers.

## Execution Notes

- Keep `tmdb_id` movie-compatible. Add explicit `tmdb_tv_id` for TV direct lookup.
- Do not force episode fields into metadata until the protocol has typed hierarchy fields.
- Bangumi subject relations are available through `/v0/subjects/{subject_id}/subjects`, but native
  relation mapping should wait for typed candidate/protocol relation support.
- AniList is the first new provider target; TVDB/MAL/IMDb should be decided after recipe/API
  boundaries are proven.

## Next Action

Execute OMANV-050 BrowserWorker recipe layer with fake-runtime tests.
