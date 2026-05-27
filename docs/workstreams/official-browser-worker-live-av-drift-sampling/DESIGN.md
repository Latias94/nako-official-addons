# Official Browser Worker Live AV Drift Sampling

## Why This Lane Exists

Generated Browser Worker render drift cases now cover all existing rendered AV
providers. The next risk is whether those generated cases survive real network,
proxy, age-gate, regional, and selector behavior.

## Relevant Authority

- `docs/workstreams/official-browser-worker-live-rendered-drift`
- `docs/workstreams/official-browser-worker-provider-render-drift-presets`
- `docs/workstreams/official-browser-worker-render-drift-preset-wave2`
- `docs/workstreams/official-browser-worker-render-drift-preset-wave3`
- `repo-ref/mdcx/mdcx/config/v1.py`
- `repo-ref/mdcx/mdcx/controllers/main_window/main_window.py`

## Problem

Offline tests prove JSON shape, but they do not prove real provider pages still
match selectors, region behavior, and proxy expectations. Mature AV scrapers
need routine drift checks that reveal whether failures are selector drift,
operator action, timeout, response-size policy, or proxy/network class.

## Target State

- Run a redaction-safe live sample across generated rendered AV cases.
- Record per-case status, failure kind, safe error code, byte sizes, and proxy
  policy without storing target URLs or secrets.
- If results expose obvious generated-case defects, fix the provider preset.
- If results expose environmental/provider access issues, document the next
  operational follow-up instead of baking secrets or site-specific cookies into
  generated cases.

## In Scope

- Generate live drift cases from enabled rendered AV providers.
- Run Browser Worker live drift through the configured local proxy.
- Preserve only redaction-safe evidence.
- Update docs/workstream with findings and any code fixes needed by the sample.

## Out Of Scope

- Storing raw URLs, page text, cookies, headers, proxy URLs, or credentials in
  evidence.
- Solving every external site block in one pass.
- Adding new providers.
- Moving metadata parsing into Browser Worker.

## Architecture Direction

The Browser Worker remains the execution boundary and returns redaction-safe
health reports. Provider modules own URL and selector presets. This lane should
improve observability and preset correctness without making live site behavior
part of default CI.

## Closeout Condition

This lane can close when live drift sampling has been run or clearly blocked,
findings are recorded, any clear preset defects are fixed and tested, and gates
pass.
