# Official Metadata Addon Non-AV Provider Expansion - Handoff

Status: Active
Last updated: 2026-05-27

## Current Phase

OMANV-020 TMDB TV foundation is implemented and passing focused tests. Next task is OMANV-030:
decide whether TMDB season/episode can be represented cleanly by the current metadata patch
contract, then implement or record a split/follow-up before continuing to Bangumi enrichment.

## Current Provider Map

- TMDB is provider-owned and split into `client`, `enrichment`, `mapper`, `parser`, `search`, and
  `test_support` modules. Existing behavior is movie-centric.
- Bangumi is provider-owned and split into the same client/enrichment/mapper/parser/search shape.
- Douban uses rendered-page support and is useful evidence for BrowserWorker recipe generalization.
- BrowserWorker currently maps explicit rendered URL results; recipe support will make it a safer
  prototype layer for HTML-heavy providers.

## Execution Notes

- Keep `tmdb_id` movie-compatible. Add explicit `tmdb_tv_id` for TV direct lookup.
- Do not force episode fields into metadata if the current protocol cannot represent them cleanly.
- AniList is the first new provider target; TVDB/MAL/IMDb should be decided after recipe/API
  boundaries are proven.

## Next Action

Assess OMANV-030 season/episode representation. If the protocol surface is too lossy, record the
gap and move to OMANV-040 Bangumi enrichment.
