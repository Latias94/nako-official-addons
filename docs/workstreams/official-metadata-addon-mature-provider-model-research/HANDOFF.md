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

Start OMAPMR-020:

- inspect Jellyfin core metadata provider interfaces and provider ordering;
- identify host-owned metadata responsibilities that should not be moved into
  the Nako sidecar;
- record findings in `FINDINGS.md` with source anchors.

## Risks

- Copying Jellyfin's host-owned library semantics into the addon sidecar would
  over-couple Nako core and official addons.
- Kodi scraper patterns are useful for parser drift, but not as a direct Rust
  architecture template.
- Emby is not part of the initial reference set because public source freshness
  is uncertain.

## Validation

OMAPMR-010 passed with `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json`, reference repo `rev-parse --short HEAD` checks, and `git diff --check`.
