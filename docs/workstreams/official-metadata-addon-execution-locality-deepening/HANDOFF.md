# Official Metadata Addon Execution Locality Deepening - Handoff

Status: Complete
Last updated: 2026-05-27

## Current State

The lane is opened as a fearless-refactor follow-on after code review found residual shallow Seams in
otherwise closed architecture lanes.

Completed:

- OMAELD-010 froze the target state and gates.
- OMAELD-020 removed the Bulk provider execution JSON tunnel. Bulk now builds a typed
  ProviderRunPolicy overlay from payload policy, cooldown-disabled provider ids, and per-item budget,
  then calls a typed runtime scrape path. The old payload mutation helper was deleted.
- OMAELD-030 moved rendered-page proxy/session configuration facts into provider catalog entries.
  `Config` now delegates those checks to `ProviderRegistry`, removing the central
  `ProviderConfigKind` match over every rendered provider.
- OMAELD-040 moved render drift sample/case selection into provider catalog descriptors. The
  render drift runner now filters enabled provider entries, resolves descriptor samples, sorts by
  descriptor order, and serializes cases without hard-coded provider branches.

Remaining:

- None for this lane.

## Closed Task

- Task ID: OMAELD-050
- Owner: codex
- Files: `crates/nako-metadata-scraper`, `docs/workstreams/official-metadata-addon-execution-locality-deepening`
- Validation: `cargo nextest run -p nako-metadata-scraper --no-fail-fast`
- Status: DONE
- Review: No blocking findings. Default field-provider preferences remain central and can become a future field-policy locality lane.
- Evidence: `EVIDENCE_AND_GATES.md`

## Decisions Since Last Update

- Use fearless refactoring for internal-only shapes.
- Do not keep compatibility wrappers for obsolete internal payload mutation.
- Keep public scrape request parsing intentional until tests prove it is unused.
- Keep `provider_execution_policy` parsing for public scrape requests; remove only Bulk's internal
  use of that JSON tunnel.
- Provider rendered-page support diagnostics belong to provider catalog facts, not central Config
  enum matching.
- Provider render drift sample, fallback, order, and builder facts belong to provider catalog
  descriptors; the runner owns lookup/filter/sort/serialization.

## Blockers

- None.

## Next Recommended Action

- Commit this lane after user confirmation.
- Consider a separate field-policy locality lane for `DEFAULT_FIELD_PROVIDER_PREFERENCES` when provider field ownership becomes the next priority.
