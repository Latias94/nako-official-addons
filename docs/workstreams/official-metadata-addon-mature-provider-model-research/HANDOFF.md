# Official Metadata Addon Mature Provider Model Research - Handoff

Status: Complete
Last updated: 2026-05-29

## Current State

This research lane is complete. It compared `nako-metadata-scraper` against
mature metadata provider systems before the next fearless refactor.

Status refresh on 2026-05-29: the original resolver and external ID capability
follow-on has been completed, and field-policy fusion is now baseline
architecture. This handoff no longer recommends reopening that P0 lane.

Reference repositories were cloned under ignored `repo-ref/` paths:

- `repo-ref/jellyfin`
- `repo-ref/jellyfin-plugin-tvdb`
- `repo-ref/jellyfin-plugin-anidb`
- `repo-ref/jellyfin-plugin-anilist`
- `repo-ref/kodi-metadata-themoviedb-python`

OMAPMR-010 through OMAPMR-050 are complete. `repo-ref/` is ignored and external
source is not part of the commit surface.

## Next Task

No next task in this lane.

Recommended follow-on:

- Design host policy context with Nako core before coding sidecar filtering for
  provider order, requested fields, locked-field summaries, or refresh intent.
- Split an artwork source pipeline only when more artwork kinds, local-first
  behaviour, or provider-specific artwork priority need it.
- Add actual cache/throttle execution state only if provider limits make it
  necessary; provider operation intent is already explicit.
- Keep host-owned responsibilities out of the sidecar: refresh state, locked
  fields, local metadata, local artwork priority, and final field merge policy.

## Risks

- Copying Jellyfin's host-owned library semantics into the addon sidecar would
  over-couple Nako core and official addons.
- Kodi scraper patterns are useful for parser drift, but not as a direct Rust
  architecture template.
- Emby is not part of the initial reference set because public source freshness
  is uncertain.
- Resolver/fusion regressions must preserve provenance; merged candidates must
  not hide which provider supplied which field or external ID.

## Validation

OMAPMR-010 passed with `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json`, reference repo `rev-parse --short HEAD` checks, and `git diff --check`.
OMAPMR-020 passed by recording Jellyfin core provider model source anchors in `FINDINGS.md`.
OMAPMR-030 passed by recording Jellyfin plugin and Kodi scraper source anchors in `FINDINGS.md`.
OMAPMR-040 passed by recording current `nako-metadata-scraper` source anchors in `FINDINGS.md` and ranked recommendations in `REFACTOR_CANDIDATES.md`.
OMAPMR-050 passed with `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json`, `cargo fmt --all -- --check`, `git diff --check`, and `git diff --name-status fbd8546..HEAD` confirming no production crate file changes.
