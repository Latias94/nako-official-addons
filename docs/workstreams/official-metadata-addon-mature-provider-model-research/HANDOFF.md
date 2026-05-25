# Official Metadata Addon Mature Provider Model Research - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

This research lane has been opened to compare `nako-metadata-scraper` against
mature metadata provider systems before the next fearless refactor.

Reference repositories were cloned under ignored `repo-ref/` paths:

- `repo-ref/jellyfin`
- `repo-ref/jellyfin-plugin-tvdb`
- `repo-ref/jellyfin-plugin-anidb`
- `repo-ref/jellyfin-plugin-anilist`
- `repo-ref/kodi-metadata-themoviedb-python`

OMAPMR-010 is complete. `repo-ref/` is ignored and external source is not part
of the commit surface.

## Next Task

Start OMAPMR-040:

- compare recorded findings against current `nako-metadata-scraper` modules;
- write `REFACTOR_CANDIDATES.md` with ranked recommendations;
- keep production code unchanged in this research lane.

## Risks

- Copying Jellyfin's host-owned library semantics into the addon sidecar would
  over-couple Nako core and official addons.
- Kodi scraper patterns are useful for parser drift, but not as a direct Rust
  architecture template.
- Emby is not part of the initial reference set because public source freshness
  is uncertain.

## Validation

OMAPMR-010 passed with `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json`, reference repo `rev-parse --short HEAD` checks, and `git diff --check`.
OMAPMR-020 passed by recording Jellyfin core provider model source anchors in `FINDINGS.md`.
OMAPMR-030 passed by recording Jellyfin plugin and Kodi scraper source anchors in `FINDINGS.md`.
