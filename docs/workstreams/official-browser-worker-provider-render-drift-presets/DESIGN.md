# Official Browser Worker Provider Render Drift Presets

## Why This Lane Exists

The Browser Worker live drift harness can execute arbitrary JSON cases, but
operators should not need to hand-write provider URLs, selectors, and actions
for providers that already know those details. This lane moves live case
construction into metadata providers while keeping execution in Browser Worker.

## Relevant Authority

- Related workstreams:
  - `docs/workstreams/official-browser-worker-live-rendered-drift`
  - `docs/workstreams/official-browser-worker-render-runtime-hardening`
  - `docs/workstreams/official-metadata-addon-provider-live-drift-checks`

## Problem

Manual Browser Worker live cases are flexible but easy to drift from provider
implementation. If a provider changes its search URL, detail URL, selector, or
age-gate action, a hand-written live case can keep checking the wrong thing.

## Target State

- Provider modules own their Browser Worker live render case presets.
- Metadata scraper can print Browser Worker-compatible case JSON through a CLI
  command.
- Presets are generated only for enabled providers.
- Secrets such as cookies are not emitted in generated cases.
- Browser Worker remains the executor and does not learn provider parsing logic.

## In Scope

- Add provider-owned presets for Douban search, JavBus detail, and JavLibrary
  search.
- Add `nako-metadata-scraper render-drift-cases`.
- Document how to pipe generated JSON into Browser Worker
  `NAKO_BROWSER_WORKER_LIVE_RENDER_DRIFT_CASES`.
- Add tests for JSON shape, enablement, and secret redaction.

## Out Of Scope

- Running external live checks in default CI.
- Exposing preset URLs in `/health`.
- Adding IMDb before an IMDb provider or rendered recipe is committed.
- Moving provider parsing into Browser Worker.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Provider modules should own URL/selector/action presets. | High | The Rust providers already own render intents and parsing selectors. | Browser Worker would duplicate provider knowledge. |
| CLI output is safer than health diagnostics for URL-bearing cases. | High | Existing health diagnostics are redaction-safe and avoid URLs. | A route can be added later with explicit operator intent. |
| Enabled providers are the right default preset scope. | Medium | Provider enablement already controls operational use. | We can add an include-disabled flag later. |

## Architecture Direction

Add a small `providers::render_drift` module that defines Browser
Worker-compatible case structs and gathers cases from enabled providers. Each
provider exposes a crate-local `render_drift_case` helper for its own URL,
selector, timeout, and actions. `main.rs` handles `render-drift-cases` before
initializing tracing so stdout remains pure JSON for shell piping.

## Closeout Condition

This lane can close when the CLI emits Browser Worker-compatible cases for
enabled providers, tests cover shape and redaction, docs show the handoff to
Browser Worker, and gates pass.
