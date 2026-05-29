# Official Metadata Addon Provider Architecture Deepening - Milestones

Status: Complete
Last updated: 2026-05-25

## M0 - Scope Frozen

- Workstream docs exist and agree.
- The five refactor candidates are represented as explicit tasks.
- Non-goals keep release publishing and live smoke out of this lane.

## M1 - Provider Descriptor And Assembly Depth

- Provider descriptors own more provider-local facts than the central config
  Module.
- Adding a provider requires fewer central edits and has a clearer descriptor
  path.
- Provider ready adapters, disabled/unavailable diagnostics, and health facts
  are derived from one provider assembly.

## M2 - Shared Search Policy And Typed Outcomes

- TMDB and Bangumi no longer duplicate the same search-enrichment policy.
- Raw provider payload parsing remains provider-local.
- Provider outcomes are typed facts before they become redaction-safe public
  text.

## M3 - Rendered Page Support Seam

- Browser-worker support has a deep rendered-page runtime Module.
- Douban uses the support Seam without owning worker HTTP details.
- `browser_worker` provider identity is either justified by real metadata
  semantics or removed/split with docs.

## M4 - Integrated Gates

- Targeted metadata scraper tests pass for all touched Modules.
- `cargo nextest run -p nako-metadata-scraper --no-fail-fast` passes or any
  external blocker is recorded.
- `cargo fmt --all -- --check` passes.
- `git diff --check` passes.

## M5 - Closeout Ready

- EVIDENCE_AND_GATES.md has fresh command evidence.
- TODO.md task handoff states are final.
- HANDOFF.md records remaining risks and follow-ons.
- WORKSTREAM.json is updated to `complete` or a follow-on is split.

Status: Complete. OMAPAD-080 closed the lane with final package, format, JSON,
and diff hygiene evidence; no architecture follow-on was split.
