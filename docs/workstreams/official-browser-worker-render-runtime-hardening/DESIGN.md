# Official Browser Worker Render Runtime Hardening

Status: Complete
Last updated: 2026-05-27

## Why This Lane Exists

`addons/browser-worker` started as a deterministic Crawlee/Playwright proof for
rendered metadata pages. It is now the execution boundary for multiple rendered
providers and future rendered recipes. The current `extract.mjs` module works,
but its interface is becoming too shallow: option parsing, proxy/session/header
policy, page actions, Crawlee lifecycle, DOM extraction, and error handling all
live in one implementation.

## Target State

- `POST /render` and legacy `POST /extract` remain compatible.
- Render request parsing is a dedicated Render Intent contract module.
- Browser execution is behind a deeper Render Runtime module with a small
  interface and a Crawlee/Playwright adapter implementation.
- Safety policy is explicit: URL validation, render timeout, action/header
  budgets, and response size limits are centralized.
- Render failures are redaction-safe and typed enough for the Rust sidecar to
  distinguish retryable, operator-action, and permanent failure classes.
- Tests exercise the contract and runtime seam without requiring every case to
  spin up a full browser.

## In Scope

- `addons/browser-worker/src` refactor.
- Browser-worker tests and smoke coverage.
- README updates for new safety/error semantics.
- Rust sidecar compatibility only where `/render` response error facts need to
  be preserved.

## Out Of Scope

- Moving metadata parsing into the browser worker.
- Changing rendered provider selectors in Rust.
- Adding new rendered providers.
- Removing `POST /extract` until callers have migrated.
- Live third-party website smoke as a required closeout gate.

## Architecture Direction

Keep the boundary that previous workstreams established: Playwright/Crawlee stay
inside `addons/browser-worker`; provider-specific metadata interpretation stays
inside `nako-metadata-scraper`.

The deepening target is:

- `render-contract.mjs`: parse and validate Render Intent, including aliases and
  redaction-safe invalid-request errors.
- `render-safety.mjs`: own URL policy, size budgets, timeout defaults, and
  bounded request controls.
- `render-runtime.mjs`: expose a small render interface and own browser page
  execution flow.
- `crawlee-render-adapter.mjs`: concrete adapter for Crawlee/Playwright.
- `extract.mjs`: compatibility facade during migration.

The deletion test should improve: deleting `extract.mjs` should not delete the
contract vocabulary, safety policy, and runtime adapter all at once.

## Risks

- Over-splitting can make a small Node worker harder to navigate. New modules
  must earn their keep by owning a real interface.
- Crawlee lifecycle reuse should not be introduced blindly if it makes tests
  flaky. Start with a deeper runtime interface, then optimize pooling only when
  the interface can hide it.
- URL policy must keep local deterministic tests possible while letting
  production operators choose stricter network rules.
