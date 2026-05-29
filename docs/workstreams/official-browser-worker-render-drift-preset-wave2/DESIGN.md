# Official Browser Worker Render Drift Preset Wave2

## Why This Lane Exists

The first provider-owned render drift preset lane covered Douban, JavBus, and
JavLibrary. Several rendered AV providers still required hand-written Browser
Worker JSON even though their provider modules already own URL construction and
search/detail selectors.

## Relevant Authority

- Related workstreams:
  - `docs/workstreams/official-browser-worker-provider-render-drift-presets`
  - `docs/workstreams/official-browser-worker-live-rendered-drift`
  - `docs/workstreams/official-metadata-addon-av-provider-wave4`

## Problem

DMM, MGStage, XCity, AirAV, and AVSox are rendered providers, but Browser Worker
live drift cases for them were not provider-owned. That would force operators
to keep URL templates and selectors in shell JSON, where they can drift from
the Rust implementation.

## Target State

- DMM and MGStage expose provider-owned render drift cases.
- Generic `RenderedSearchAvSite` providers can generate search drift cases from
  their site definitions.
- XCity, AirAV, and AVSox are included through the generic path.
- Generated cases still omit cookies, headers, and session keys while preserving
  safe `proxy_policy`.

## In Scope

- Add DMM search preset.
- Add MGStage detail preset with a provider-specific default sample.
- Add generic rendered-search AV preset generation.
- Wire XCity, AirAV, and AVSox into the preset collector.
- Update tests, docs, and workstream evidence.

## Out Of Scope

- Running live external checks by default.
- Adding presets for non-rendered providers.
- Adding IMDb before an IMDb provider or rendered recipe exists.
- Emitting secrets in generated cases.

## Architecture Direction

Keep independent provider presets in their provider modules. For providers that
already use `RenderedSearchAvSite`, add one generic preset helper beside the
generic search implementation so it shares the same search URL builder and
selector assumptions.

## Closeout Condition

This lane can close when wave2 providers are included in generated CLI cases,
Browser Worker can parse those cases, tests cover the JSON shape, docs mention
the expanded provider set, and gates pass.
