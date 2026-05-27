# Official Metadata Addon Execution Locality Deepening

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

The previous metadata architecture lanes closed real work, but the current code still has three
high-leverage shallow Seams:

- Bulk Metadata Scrape still tunnels provider execution policy through request JSON.
- Provider catalog ownership still leaves provider-specific facts in central config/registry files.
- Render drift case generation still keeps sample routing and provider case selection in one central
  Module.

The user has explicitly chosen fearless refactoring: remove obsolete code, avoid compatibility shims
for internal-only shapes, and prefer the correct architecture over minimal local edits.

## Relevant Authority

- ADRs:
  - `../nako/docs/adr/0034-package-addon-capabilities-into-sidecar-suites.md`
  - `../nako/docs/adr/0042-external-casting-protocol-adapters.md`
- Existing docs:
  - `../nako/CONTEXT.md`
  - `README.md`
  - `addons/metadata-scraper/README.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-scraper-architecture-deepening`
  - `docs/workstreams/official-metadata-addon-provider-architecture-deepening`
  - `docs/workstreams/official-metadata-addon-bulk-task-design`
  - `docs/workstreams/official-browser-worker-provider-render-drift-presets`

## Problem

The addon is internally typed in several places, but some residual paths still leak Implementation
facts across Module Seams. The result is high edit fan-out when changing provider execution, adding a
provider, or adding render drift coverage.

## Target State

- Bulk Metadata Scrape passes typed provider execution facts to the scrape runtime instead of
  mutating JSON payload.
- Provider-specific catalog/config/field facts move toward provider-owned descriptors, with central
  registry files acting as composition/query Modules.
- Render drift sample/case facts move toward provider-owned descriptors, with the runner owning only
  sample lookup, filtering, and serialization.
- Internal compatibility wrappers are removed when they exist only to preserve obsolete internal
  paths.

## In Scope

- `crates/nako-metadata-scraper/src/engine`
- `crates/nako-metadata-scraper/src/providers`
- `crates/nako-metadata-scraper/src/config.rs` when provider-locality cleanup requires it
- `docs/workstreams/official-metadata-addon-execution-locality-deepening`

## Out Of Scope

- Publishing `v0.1.0-alpha.2`.
- Adding new metadata providers.
- Changing the Addon Protocol crate.
- Nako core scan scheduler changes unless a typed addon fact is demonstrably required.
- Notification bridge and Chromecast follow-ons.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| The Bulk provider execution JSON tunnel is internal-only and can be removed without Addon Protocol changes. | High | `bulk.rs` builds the policy before calling `scrape_outcome`; `provider_execution.rs` parses it from payload. | If external callers rely on that payload field, keep request parsing for single scrape but stop Bulk from using it internally. |
| Provider catalog and render drift follow-ons can be split after the Bulk slice. | High | Existing workstreams closed adjacent lanes, but `rg` still shows central residuals. | If the first slice reveals deeper coupling, split narrower follow-ons before touching provider catalog. |
| Targeted metadata-scraper gates are sufficient for task iteration. | High | Changes are within one crate and existing nextest filters cover bulk/provider/render_drift. | Broaden to workspace gate before closeout or if shared crates change. |

## Architecture Direction

Prefer Deep Modules with explicit internal facts:

- Provider Execution Module owns execution policy, suppression, budget, and report facts.
- Bulk Metadata Scrape owns batch planning and summary state, not scrape payload mutation.
- Runtime owns request parsing for the public scrape path and a typed internal entrypoint for Bulk.
- Provider modules own provider-specific catalog and render drift facts.
- Central registries compose provider-owned facts and answer queries.

This keeps Interface depth proportional to the Implementation complexity hidden behind it and
improves Locality for future provider work.

## Closeout Condition

This lane can close when:

- the three residual Seams are either removed or split into narrower follow-ons with evidence,
- targeted and package gates pass,
- workstream docs record the shipped behavior,
- and no obsolete internal compatibility code remains in the touched paths.

Closeout status: complete on 2026-05-27. Bulk execution, rendered-page support diagnostics, and
render drift case generation now route through typed or provider-owned facts. The remaining central
field-provider preference tables in `ProviderRegistry` are a separate field-policy ownership topic,
not part of this lane's three target Seams.
