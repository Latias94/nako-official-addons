# Official Notification Bridge Provider Extensibility

Status: Closed
Last updated: 2026-05-25

## Why This Lane Exists

`nako-notification-bridge` now has a useful first provider surface: ACK-only
mode, HTTP webhook, Discord webhook, safe templates, bounded attempt history,
and redaction-safe health/diagnostics. The next risk is structural: adding more
notification providers directly in `routes.rs` will multiply branching,
diagnostics, and test-send behavior across provider-specific code.

## Relevant Authority

- ADRs:
  - `../nako/docs/adr/0003-http-addons-before-in-process-plugins.md`
  - `../nako/docs/adr/0015-capability-scoped-http-addons-and-automation-providers.md`
  - `../nako/docs/adr/0020-jellyfin-like-sidecar-addons-with-scoped-api-access.md`
- Existing docs:
  - `crates/nako-notification-bridge/README.md`
  - `addons/notification-bridge/README.md`
  - `README.md`
- Related workstreams:
  - `docs/workstreams/official-metadata-addon-fearless-refactor`
  - `docs/workstreams/official-addon-contract-smoke-provider-hardening`
  - `docs/workstreams/official-addons-cross-repo-fearless-refactor`

## Problem

Provider orchestration currently lives in route handlers. That was acceptable
for two providers, but it makes these follow-ons harder than necessary:

- adding Telegram, Home Assistant, email, or other adapters;
- exposing a safe provider test-send command for operators;
- keeping provider diagnostics consistent and redaction-safe;
- enforcing exactly one configured send path without route-level provider
  knowledge.

## Target State

When this lane closes:

- notification providers are represented by a small provider registry/catalog
  seam that owns provider identity, safe diagnostics, send-path readiness, and
  provider-specific send execution;
- route handlers delegate provider selection, fail-closed multi-provider
  checks, attempt-history recording, and output shaping to that seam;
- operators get a redaction-safe test-send path that proves the configured
  provider can receive a synthetic notification without raw secrets or webhook
  URLs in responses;
- at least one additional provider or provider-ready extension point is
  demonstrably cheap to add;
- docs and smoke instructions teach the resulting operator workflow.

## In Scope

- Refactor `nako-notification-bridge` provider orchestration behind a
  provider-local registry/seam.
- Preserve the existing HTTP webhook and Discord webhook behavior.
- Add a redaction-safe test-send endpoint or resource path for configured
  provider sends.
- Add one provider-extension proof if it remains small after the registry seam
  lands.
- Update operator docs and changelog.

## Out Of Scope

- Publishing crates or container images.
- Changing Addon Protocol runtime compatibility.
- Persisting provider attempt history.
- Provider retry queues, scheduler ownership, or host process supervision.
- Storing raw webhook URLs, secrets, headers, message bodies, or raw event
  payload values in diagnostics.

## Starting Assumptions

| Assumption | Confidence | Evidence | Consequence if wrong |
| --- | --- | --- | --- |
| Two providers are enough pressure to justify a registry seam before adding more. | High | `routes.rs` currently knows both provider clients, configs, diagnostics, send-path count, and attempt-history recording. | If the seam is too abstract, keep it provider-local and avoid cross-addon generalization. |
| Test-send is the highest-value operator feature after diagnostics. | Medium | Live provider smoke exists but is local-only; operators still need a sidecar-owned way to prove configured delivery. | If host protocol constraints make this awkward, split test-send into a follow-on and land registry first. |
| Exactly one provider send path remains the right default. | High | Current code and docs fail closed for multiple send paths to avoid duplicated retry fan-out. | If multi-send becomes a product requirement, it needs a separate retry/idempotency design. |
| No ADR is needed for a provider-local registry. | Medium | Metadata scraper already uses a provider registry seam inside this repo. | If the seam starts changing Addon Protocol or install boundaries, propose an ADR/follow-on. |

## Architecture Direction

Keep the abstraction local to `nako-notification-bridge` first. The provider
registry should not try to become a universal addon provider framework. It
should make the current sidecar easier to reason about:

- a provider descriptor owns id, status, send-path readiness, and diagnostics;
- a provider runtime owns sending one safe notification shape;
- route handlers ask the registry for configured providers and execute the
  selected path;
- attempt-history records are created only for actual send attempts and
  failures, not disabled ACK-only providers;
- health and diagnostics are derived from registry facts, not duplicate route
  logic.

This mirrors the successful metadata scraper registry direction while staying
smaller because notification providers share a narrower event shape.

## Closeout Condition

This lane can close when:

- the provider registry/test-send target state is implemented or deliberately
  split into a narrower follow-on;
- existing provider behavior is preserved by tests;
- redaction-safety tests cover health, diagnostics, test-send responses, and
  attempt history;
- targeted `nako-notification-bridge` gates pass;
- docs reflect the shipped operator workflow.

## Closeout Summary

Closed on 2026-05-25.

Delivered:

- provider-local registry facts for diagnostics, configuration status,
  selected-provider output, fail-closed multi-provider checks, and
  attempt-history mapping;
- sidecar-local `POST /providers/test-send` for a single configured provider;
- default-disabled Telegram provider proof with fixture-backed
  `library.scanned` and test-send delivery;
- redaction-safe health, diagnostics, provider output, attempt history, docs,
  and tests for HTTP webhook, Discord webhook, and Telegram.

Residual follow-on:

- Provider send execution still requires one explicit route-helper call per
  provider client. That is acceptable for three small providers, but a
  provider-runtime catalog can be split out if the next provider would repeat
  the same route wiring.
