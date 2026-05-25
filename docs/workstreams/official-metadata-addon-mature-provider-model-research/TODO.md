# Official Metadata Addon Mature Provider Model Research - TODO

Status: Active
Last updated: 2026-05-25

## M0 - Research Lane Setup

- [x] OMAPMR-010 [owner=planner] [deps=none] [scope=.gitignore,repo-ref,docs/workstreams/official-metadata-addon-mature-provider-model-research]
  Goal: Open the research workstream, ignore `repo-ref/`, and clone the selected reference repositories.
  Validation: `git status --short`; `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json`; reference repo `git rev-parse --short HEAD` commands.
  Review: Confirm reference repositories are ignored and no external source is staged.
  Evidence: `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json`; `git -C repo-ref/jellyfin rev-parse --short HEAD`; `git -C repo-ref/jellyfin-plugin-tvdb rev-parse --short HEAD`; `git -C repo-ref/jellyfin-plugin-anidb rev-parse --short HEAD`; `git -C repo-ref/jellyfin-plugin-anilist rev-parse --short HEAD`; `git -C repo-ref/kodi-metadata-themoviedb-python rev-parse --short HEAD`; `git diff --check`.
  Handoff: DONE. Research lane opened, `repo-ref/` is ignored, and selected reference repositories are available for OMAPMR-020 and OMAPMR-030.

## M1 - Jellyfin Core Provider Model

- [ ] OMAPMR-020 [owner=codex] [deps=OMAPMR-010] [scope=repo-ref/jellyfin,docs/workstreams/official-metadata-addon-mature-provider-model-research/FINDINGS.md]
  Goal: Extract Jellyfin core provider concepts relevant to Nako: provider interfaces, metadata/image/local metadata roles, refresh/order semantics, and host-owned responsibilities.
  Validation: source anchors recorded in `FINDINGS.md`.
  Review: Separate patterns that Nako sidecar can own from patterns that must remain Nako core concerns.
  Evidence: pending.
  Handoff: TODO.

## M2 - Plugin And Scraper Model

- [ ] OMAPMR-030 [owner=codex] [deps=OMAPMR-010] [scope=repo-ref/jellyfin-plugin-*,repo-ref/kodi-metadata-themoviedb-python,docs/workstreams/official-metadata-addon-mature-provider-model-research/FINDINGS.md]
  Goal: Compare Jellyfin plugins and Kodi scraper implementation patterns for provider config, lookup flow, mapping, images, parser drift, and operational behaviour.
  Validation: source anchors recorded in `FINDINGS.md`.
  Review: Avoid copying ecosystem-specific plugin mechanics that do not fit Nako Addon Protocol.
  Evidence: pending.
  Handoff: TODO.

## M3 - Local Architecture Comparison

- [ ] OMAPMR-040 [owner=codex] [deps=OMAPMR-020,OMAPMR-030] [scope=crates/nako-metadata-scraper,docs/workstreams/official-metadata-addon-mature-provider-model-research/FINDINGS.md,docs/workstreams/official-metadata-addon-mature-provider-model-research/REFACTOR_CANDIDATES.md]
  Goal: Compare mature-system patterns against current `nako-metadata-scraper` and rank refactor candidates.
  Validation: `REFACTOR_CANDIDATES.md` contains ranked recommendations with risks, affected modules, and suggested gates.
  Review: Confirm candidates increase locality and leverage without smuggling host responsibilities into the sidecar.
  Evidence: pending.
  Handoff: TODO.

## M4 - Close Or Split

- [ ] OMAPMR-050 [owner=planner] [deps=OMAPMR-040] [scope=docs/workstreams/official-metadata-addon-mature-provider-model-research]
  Goal: Close the research lane or split a concrete implementation workstream.
  Validation: `cargo fmt --all -- --check`; `git diff --check`; `python -m json.tool docs/workstreams/official-metadata-addon-mature-provider-model-research/WORKSTREAM.json`.
  Review: Confirm no production code changed in this research lane unless explicitly justified.
  Evidence: pending.
  Handoff: TODO.
