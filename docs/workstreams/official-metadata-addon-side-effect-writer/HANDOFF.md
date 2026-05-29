# Official Metadata Addon Side Effect Writer - Handoff

Status: Complete
Last updated: 2026-05-23

## Current State

The lane is complete. Existing official addon lanes are complete: architecture
refactor, TMDB production baseline, Bangumi baseline, result-quality ranking,
release prep, and this side-effect writer lane. Nako core has already
implemented Addon Token, Addon Side Effect intake, metadata protected writes,
artwork candidate proposals, and NFO Library File Write paths.

OMASE-030 is complete. The Addon now has a disabled-by-default Nako runtime
config, a fake-transport-testable outbound client for `access-check` and
`side-effects`, and an explicit `payload.writeback` path that submits the
selected metadata candidate as a `metadata_write` side effect when runtime
writes are enabled.

OMASE-040 is complete. Provider image facts now surface as typed artwork
candidates, `artwork_write` uses the Nako-owned side-effect path, and browser
worker / TMDB / Bangumi tests cover the new artwork candidate identity and
request shapes.

## Follow-On

The Bulk Metadata Scrape / Addon Task discussion continues in the dedicated
`official-metadata-addon-bulk-task-design` lane.

## Known Constraints

- Do not depend on private `../nako` server/core crates from this addon
  workspace.
- Keep ordinary `/metadata` calls suggestion-only unless an explicit
  `writeback` object is present.
- Do not add hidden background jobs for Bulk Metadata Scrape.
- Do not expose raw Addon Tokens, provider tokens, local paths, source
  locators, or raw provider payloads.

## Next Likely Phase

No remaining work is tracked in this lane. Follow-on work now lives in the
bulk-task design lane and any host-side task runtime workstreams it may spawn.
