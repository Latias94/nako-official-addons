# Official Metadata Addon Mature Provider Model Research - Milestones

Status: Complete
Last updated: 2026-05-29

## M0 - Reference Base Ready

- `repo-ref/` is ignored.
- Selected reference repositories are cloned or skipped with reasons.
- Workstream docs exist and agree.

## M1 - Mature Core Patterns Extracted

- Jellyfin core provider interfaces and host-owned metadata responsibilities are
  summarized with source anchors.
- Findings distinguish provider-side mechanics from library-management
  semantics.

## M2 - Plugin And Scraper Patterns Extracted

- Jellyfin plugin patterns are summarized with source anchors.
- Kodi scraper patterns are summarized only where parser/site-drift lessons
  apply to Nako.

## M3 - Refactor Candidates Ranked

- Current `nako-metadata-scraper` architecture is compared against findings.
- Follow-on candidates include problem, proposed change, affected files, gates,
  risk, and recommendation strength.

## M4 - Research Closed Or Split

- The lane is closed with no code changes, or a concrete implementation
  workstream is split.
- Evidence and handoff record exact next action.

Closeout result:

- Research lane closed with no production code changes.
- The concrete follow-on was completed by
  `official-metadata-addon-provider-fact-resolver`: provider fact resolver plus
  external ID capability catalog are now baseline architecture.
- Later field-policy workstreams added request/default provider field policy and
  provider-owned default field preferences.
