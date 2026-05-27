# Official Metadata Addon Non-AV Provider Expansion

Status: Active
Last updated: 2026-05-27

## Why This Lane Exists

AV coverage is now broad, field-policy aware, and provider-rich. The next provider value is outside
AV: mainstream movies/TV, anime, browser-rendered metadata recipes, and a small set of new
API-backed providers.

The current non-AV provider shape is:

- TMDB: stable official API, currently movie-centric.
- Bangumi: anime-focused official API provider with deep baseline mapping.
- Douban: Chinese movie provider implemented through rendered-page support.
- BrowserWorker: explicit URL-rendering provider and shared rendering support for rendered providers.

This lane turns that set into a future-facing non-AV provider platform.

## Target State

- TMDB supports TV series, season, and episode search/direct lookup without breaking existing movie
  behavior.
- Bangumi emits richer anime metadata from subject details and keeps direct subject lookup reliable.
- BrowserWorker can execute typed extraction recipes so rendered providers can be prototyped before
  becoming native modules.
- New provider candidates are gated by maintainability:
  - AniList first, because its GraphQL API is stable and useful for anime cross-checking.
  - TVDB next, because it strengthens series/episode metadata when an API key is available.
  - MAL only if its API constraints fit the sidecar.
  - IMDb as a browser-worker recipe first, not a native scraper, unless a legal/stable API path is
    chosen.

## Reference Boundary

- TMDB: official developer API docs for movie/TV/search/detail/external ID endpoints.
- Bangumi: existing provider contract plus official v0 API shape already encoded in tests.
- BrowserWorker: existing local render contract in `providers/rendered_page.rs`.
- New providers: official API docs only; no selector copying from third-party scrapers.

## In Scope

- Provider capability/config/external-id declarations.
- Provider-local parser/mapper/client/enrichment code.
- Runtime routing and candidate ranking behavior needed for new non-AV candidates.
- BrowserWorker extraction recipe schema and fake-runtime tests.
- README/manifest/workstream docs.

## Out Of Scope

- Adding more AV providers.
- Changing the Addon Protocol wire format unless a narrow extension is unavoidable.
- Replacing TMDB/Bangumi/Douban with one generic scraper.
- Native IMDb scraping as the first implementation step.

## Architecture Direction

Keep the pattern that worked for AV and previous provider refactors:

- Providers own catalog entries, config, external ID aliases, parser, mapper, and tests.
- Shared runtime utilities stay provider-neutral.
- Browser-rendered extraction becomes recipe-driven but still emits normalized
  `ProviderMetadataCandidate` values.
- New API providers must be implemented as small vertical slices: config + client + parser + mapper
  + registry + focused tests.

## Protocol Gap: TV Seasons And Episodes

TMDB season and episode endpoints are stable enough to support native extraction, but the current
metadata patch contract cannot preserve that data cleanly. `AddonMetadataPatch` has no typed series,
season, episode, season number, episode number, or still-image hierarchy, and it denies unknown
fields. Encoding these facts as tags would make ranking, writeback, and downstream UI behavior
ambiguous.

The correct follow-up is a narrow protocol extension for non-AV episodic metadata, then a TMDB
season/episode provider slice using explicit external IDs such as `tmdb_tv`, `tmdb_season`, and
`tmdb_episode`.

## Protocol Gap: Related Subjects

Bangumi exposes related subjects through `/v0/subjects/{subject_id}/subjects`, and those relations
are useful for sequels, adaptations, side stories, and franchise navigation. The current addon
candidate model has no typed relation graph, so this lane does not encode related subjects as
ambiguous tags. A future relation-capable candidate fact should carry relation label, provider,
provider ID, subject type, and display titles.

## Task Strategy

1. Ship TMDB TV as the first executable slice because it has the best stability/reward ratio.
2. Improve Bangumi fields after TMDB TV establishes the non-movie mapping pattern.
3. Add BrowserWorker recipes before risky HTML-only provider expansion.
4. Add AniList as the first new provider; defer TVDB/MAL/IMDb until the recipe/API contracts are
   proven.

## Closeout Condition

This lane closes when the selected first-wave non-AV providers are implemented, tested, documented,
and committed, or when the remaining provider candidates are split into explicit follow-up lanes.
