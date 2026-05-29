# Official Metadata Addon Provider Hardening — Handoff

Status: Complete
Last updated: 2026-05-23

## Current State

The lane is closed. Two proof slices landed: TMDB and Bangumi now accept
proxy-aware provider config, health diagnostics report that policy without
leaking proxy URLs, and ranking now uses original/sort title variants when the
primary surface title differs.

## Active Task

- Task ID: OMPH-040
- Owner: planner
- Files: `docs/workstreams/official-metadata-addon-provider-hardening`
- Validation: `verify-rust-workstream` and `review-workstream`
- Status: COMPLETE
- Review: Passed
- Evidence: `docs/workstreams/official-metadata-addon-provider-hardening/EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Provider network policy belongs in the HTTP runtime seam.
- CheckTMDB remains a reference for operator workarounds only.
- Browser automation and Douban stay in separate lanes.
- Ranking may use `original_title` and `sort_title` from the patch surface,
  not only the primary title.

## Blockers

- None.

## Next Recommended Action

- Split follow-on work for provider breadth, localized coverage, or artwork
  selection nuance if those remain worth pursuing.

## Follow-ons

- TMDB/Bangumi provider breadth and localization
- artwork selection nuance
- alias expansion and broader title matching
