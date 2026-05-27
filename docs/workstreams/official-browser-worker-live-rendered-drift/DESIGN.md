# Official Browser Worker Live Rendered Drift

## Why This Lane Exists

Rendered providers can keep their Rust parser tests green while the live page no
longer renders, the wait selector moves, or a proxy/age gate starts failing.
After the render runtime hardening lane, the worker has a stable execution
boundary and redaction-safe failure taxonomy. This lane adds an operator-run
health harness for that boundary.

## Relevant Authority

- Related workstreams:
  - `docs/workstreams/official-browser-worker-render-runtime-hardening`
  - `docs/workstreams/official-metadata-addon-provider-live-drift-checks`
  - `docs/workstreams/official-metadata-addon-av-ops-presets-and-drift`

## Problem

The browser worker has local smoke coverage, and Rust providers have opt-in live
provider drift coverage, but there is no narrow check that says whether
rendered-page execution itself can still open a live site, satisfy a provider
selector, and return useful HTML/text without leaking target details in logs.

## Target State

- Browser Worker has a default offline render drift suite backed by its local
  fixture.
- Operators can add live cases through an explicit environment gate and JSON
  case list.
- Reports are redaction-safe: case id, source, booleans, byte counts, HTTP
  status, `safe_error_code`, and `failure_kind` only.
- Default tests and CI do not require network access.

## In Scope

- Add a reusable Browser Worker render drift module.
- Add a `npm run live:render-drift` harness that starts an ephemeral worker and
  runs fixture/live cases.
- Cover parser/report redaction and the fixture render path with tests.
- Document live case configuration and validation gates.

## Out Of Scope

- Scheduling continuous monitoring.
- Encoding provider-specific parsing rules inside the worker.
- Hardcoding live adult-site sample URLs in source or docs.
- Replacing Rust provider-level field drift checks.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Render drift should measure worker execution, not metadata parsing. | High | Runtime hardening lane keeps parsing in Rust providers. | This lane would need to move back into provider tests. |
| Live drift must be opt-in and redaction-safe. | High | Existing live provider drift and failure taxonomy docs use explicit env gates. | Default CI or logs could become flaky or sensitive. |
| JSON case config is acceptable for operator-run checks. | Medium | Render request contract is already structured JSON. | We may add a shorthand parser later. |

## Architecture Direction

Keep the harness inside `addons/browser-worker` because it probes the worker HTTP
contract directly. The module owns case normalization, fixture/live selection,
redaction-safe health reports, and exit-code policy. Provider-specific page URLs,
selectors, headers, cookies, and actions remain operator-provided inputs and are
never emitted in reports.

## Closeout Condition

This lane can close when the harness exists, tests cover the fixture and
redaction behavior, README documents the opt-in live path, and gates pass.
