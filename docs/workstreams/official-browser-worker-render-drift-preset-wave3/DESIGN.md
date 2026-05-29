# Official Browser Worker Render Drift Preset Wave3

## Why This Lane Exists

Wave2 covered DMM, MGStage, XCity, AirAV, and AVSox. Several rendered providers
still required hand-written Browser Worker live drift JSON even though their Rust
modules already own URL construction and rendered selectors.

## Relevant Authority

- `docs/workstreams/official-browser-worker-live-rendered-drift`
- `docs/workstreams/official-browser-worker-provider-render-drift-presets`
- `docs/workstreams/official-browser-worker-render-drift-preset-wave2`
- `repo-ref/mdcx/mdcx/crawlers/javdb_new.py`
- `repo-ref/mdcx/mdcx/crawlers/fc2.py`
- `repo-ref/mdcx/mdcx/crawlers/fc2ppvdb.py`

## Problem

JavDB, FC2, FC2PPVDB, Caribbean, 1Pondo, and 10Musume use rendered HTML paths
but are not yet represented in generated drift cases. Operators can test these
providers only by writing JSON manually, which duplicates provider URL rules and
increases the chance of checking a URL shape that the scraper itself never uses.

## Target State

- JavDB emits a provider-owned search drift case.
- FC2 and FC2PPVDB emit direct detail drift cases for FC2 route samples.
- Official uncensored sites emit reusable detail drift cases from
  `OfficialUncensoredSite`.
- Generated cases preserve safe `proxy_policy` and timeout configuration while
  omitting session keys, cookies, and headers.
- Provider-specific sample environment variables allow each route to use a
  realistic sample without relying on one global AV number.

## In Scope

- Add provider-owned cases for JavDB, FC2, FC2PPVDB, Caribbean, 1Pondo, and
  10Musume.
- Add sample env vars and route-aware defaults for censored, FC2, and uncensored
  paths.
- Extend render drift unit coverage and generated CLI/parser smoke coverage.
- Update README and workstream evidence.

## Out Of Scope

- Adding new metadata providers.
- Changing parser field extraction semantics.
- Running external live drift by default.
- Emitting secrets in drift case JSON.

## Architecture Direction

Keep provider-specific URL decisions inside provider modules. Shared rendered
provider families should expose one generic helper in their shared module:
`RenderedSearchAvSite` already owns search drift, and `OfficialUncensoredSite`
should own official uncensored detail drift.

## Closeout Condition

This lane can close when generated CLI cases include all remaining rendered AV
provider presets, Browser Worker can parse the expanded set, focused Rust and
Browser Worker gates pass, docs are updated, and the change is committed.
