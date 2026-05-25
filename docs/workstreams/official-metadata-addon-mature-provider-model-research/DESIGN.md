# Official Metadata Addon Mature Provider Model Research

Status: Complete
Last updated: 2026-05-25

## Why This Lane Exists

The metadata scraper provider extension architecture is now decentralized enough
to add providers without repeatedly editing query parsing and provider config
internals. The next architecture decision should be informed by mature metadata
systems instead of only local intuition.

This lane studies Jellyfin core provider code, selected Jellyfin metadata
plugins, and one Kodi scraper reference to decide how Nako should deepen its
metadata provider model before the next round of fearless refactoring.

## Target State

- Reference repositories live under ignored `repo-ref/` paths and are treated as
  read-only research material.
- The workstream records which mature-system patterns are worth copying,
  adapting, or rejecting.
- Findings distinguish host-owned responsibilities from addon-sidecar
  responsibilities.
- Refactor candidates are ranked by leverage, locality, risk, and compatibility
  with the Nako Addon Protocol.
- No production code is changed until the research has a concrete architectural
  recommendation.

## Scope

- Study Jellyfin core metadata provider interfaces, ordering, image provider,
  local metadata, refresh, and item lookup concepts.
- Study Jellyfin metadata plugins for provider declaration, configuration,
  authentication, lookup, mapping, and image handling.
- Study Kodi TMDB Python scraper as a parser and site-drift reference, not as a
  direct architecture template.
- Compare those patterns with `nako-metadata-scraper` provider registry,
  runtime, ranking, writeback, bulk task, and browser-rendered support.
- Produce `FINDINGS.md` and `REFACTOR_CANDIDATES.md` before any follow-on
  implementation lane.

## Non-Goals

- Do not vendor or copy reference repository source into production code.
- Do not add providers in this lane.
- Do not perform release or live provider smoke work in this lane.
- Do not redesign Nako core contracts without splitting a separate ADR or
  implementation workstream.
- Do not treat Emby private or stale public code as authoritative unless a
  reliable public source is explicitly recorded.

## Reference Repositories

- `repo-ref/jellyfin` from `https://github.com/jellyfin/jellyfin.git`, sparse
  checkout focused on provider-related directories.
- `repo-ref/jellyfin-plugin-tvdb` from
  `https://github.com/jellyfin/jellyfin-plugin-tvdb.git`.
- `repo-ref/jellyfin-plugin-anidb` from
  `https://github.com/jellyfin/jellyfin-plugin-anidb.git`.
- `repo-ref/jellyfin-plugin-anilist` from
  `https://github.com/jellyfin/jellyfin-plugin-anilist.git`.
- `repo-ref/kodi-metadata-themoviedb-python` from
  `https://github.com/xbmc/metadata.themoviedb.org.python.git`.

## Architecture Questions

1. Which provider concepts should stay provider-local?
2. Which concepts belong in Nako core instead of this sidecar?
3. Should Nako split metadata, image, local metadata, and rendered-page provider
   roles?
4. Should ranking evolve into a resolver that supports field-level merge policy?
5. What cache, drift, and operational surfaces are required before this behaves
   like a mature metadata subsystem?

## Close Criteria

This lane can close when:

- reference repositories are cloned or explicitly skipped with reasons;
- `FINDINGS.md` records mature-system patterns with source anchors;
- `REFACTOR_CANDIDATES.md` ranks follow-on work;
- gates prove workstream docs are valid and the production tree remains clean;
- follow-on implementation work is either split or explicitly deferred.

## Closeout

Closed on 2026-05-25.

Result:

- Reference repositories are ignored under `repo-ref/` and were used only as
  read-only research material.
- `FINDINGS.md` records mature-system and local architecture findings with
  source anchors.
- `REFACTOR_CANDIDATES.md` ranks follow-on work.
- No production Rust code changed in this research lane.
- Follow-on implementation is deferred to a new workstream, recommended scope:
  sidecar-local provider fact resolver plus external ID capability catalog.
