# Official Metadata Addon Mature Provider Model Research - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Gate Plan

| Gate | Command | When |
| --- | --- | --- |
| Workstream JSON | `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json` | OMAPMR-010 and closeout |
| Reference status | `git -C <repo-ref path> rev-parse --short HEAD` | OMAPMR-010 |
| Research anchors | `rg <pattern> repo-ref/<repo>` | OMAPMR-020 and OMAPMR-030 |
| Format | `cargo fmt --all -- --check` | Before commits and closeout |
| Diff hygiene | `git diff --check` | Before commits and closeout |

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | OMAPMR-010 setup | Added `repo-ref/` to `.gitignore`; cloned Jellyfin core sparse reference, Jellyfin TVDB/AniDB/AniList plugin references, and Kodi TMDB Python scraper reference. GitHub API search was rate-limited, so known official repository URLs were validated with `git ls-remote`. Validated `WORKSTREAM.json`, confirmed reference HEADs (`jellyfin` 498d265, `jellyfin-plugin-tvdb` 5c4592f, `jellyfin-plugin-anidb` 456ded6, `jellyfin-plugin-anilist` 0d973c4, `kodi-metadata-themoviedb-python` 285bc75), confirmed `repo-ref/` is ignored, and ran `git diff --check`. | Pass |
| 2026-05-25 | OMAPMR-020 Jellyfin core provider model | Inspected Jellyfin core provider interfaces, provider ordering, refresh state, remote search deduplication, metadata merge, locked field handling, and image pipeline. Recorded source anchors and Nako implications in `FINDINGS.md`. | Pass |
| 2026-05-25 | OMAPMR-030 plugin and scraper model | Inspected Jellyfin TVDB/AniDB/AniList plugin provider code, cache/rate-limit/API client policy, external ID lookup, image providers, and Kodi TMDB Python scraper matching/artwork/result-shaping heuristics. Recorded source anchors and Nako implications in `FINDINGS.md`. | Pass |

## Notes

- Reference repositories are ignored and must stay read-only.
- Emby is intentionally not cloned in the initial set because public source
  quality and freshness are uncertain compared with Jellyfin.
- No release or live provider smoke gates are required for this research lane.
