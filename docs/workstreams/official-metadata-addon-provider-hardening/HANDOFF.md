# Official Metadata Addon Provider Hardening — Handoff

Status: Active
Last updated: 2026-05-23

## Current State

The lane is opened and scoped. Two proof slices are done: TMDB and Bangumi now
accept proxy-aware provider config, health diagnostics report that policy
without leaking proxy URLs, and ranking now uses original/sort title variants
when the primary surface title differs. The lane is ready for closeout review
or a narrower provider-quality follow-on.

## Active Task

- Task ID: OMPH-040
- Owner: planner
- Files: `docs/workstreams/official-metadata-addon-provider-hardening`
- Validation: `verify-rust-workstream` and `review-workstream`
- Status: NEEDS_CONTEXT
- Review: Pending
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

- Run closeout review and verification, then either close the lane or split any
  remaining provider-quality breadth into a narrower follow-on.
