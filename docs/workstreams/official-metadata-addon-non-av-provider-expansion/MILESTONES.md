# Official Metadata Addon Non-AV Provider Expansion - Milestones

Status: Complete
Last updated: 2026-05-27

## M0 - Scope

- Workstream opened.
- First-wave provider order is TMDB TV, Bangumi enrichment, BrowserWorker recipes, AniList.

## M1 - TMDB TV

- Done: TV series search/direct lookup with `tmdb_tv` direct IDs, TV search fallback, external IDs,
  artwork, aliases, score/vote facts, and provider outcomes.
- Done: Season/episode native mapping deferred as a protocol gap instead of forcing lossy tags.

## M2 - Bangumi

- Done: richer anime fields, SlimSubject compatibility, score/rank/collection facts, direct ID
  aliases, subject type labels, and structured credits/studios.

## M3 - BrowserWorker

- Done: typed rendered-page recipe extraction path with generic metadata selectors and direct
  `browser_worker_recipe_url` input.

## M4 - New Providers

- Done: AniList first-wave provider with GraphQL search/direct lookup, MAL ID
  crosswalk, anime fields, artwork, score facts, config, manifest, and docs.
- Done: TVDB/MAL/IMDb split decision. TVDB waits for typed episodic protocol
  fields, MAL remains a follow-up behind AniList's crosswalk, and IMDb starts as
  a BrowserWorker recipe unless a stable legal API path is selected.

## M5 - Closeout

- Done: full package, format, JSON, and diff hygiene gates passed. README,
  manifest example, and workstream evidence are updated. TVDB/MAL/IMDb are
  explicit follow-ups instead of unfinished in-lane tasks.
