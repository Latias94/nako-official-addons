# Milestones

Status: Complete
Last updated: 2026-05-26

## M1 - Lane Opened

Status: Complete

Exit criteria:

- Workstream docs exist and match the current repository state.
- Task order protects the protocol boundary and GPL reference-only guardrail.
- First executable task is clear.

## M2 - Structured AV Facts

Status: Complete

Exit criteria:

- Existing AV providers emit structured AV facts without losing legacy tags.
- Candidate JSON includes non-empty AV facts.
- Field-source evidence can name AV fields selected by provider policy.
- Tests cover single-provider and merged-provider selection.

## M3 - Browser Worker Proxy/Wait/Session Contract

Status: Complete

Exit criteria:

- `/health` reports only redaction-safe proxy configuration booleans.
- `/extract` and `/render` accept validated optional controls.
- Crawlee/Playwright receives proxy configuration when env config is present.
- Tests cover request validation and no-secret response behavior.

## M4 - Provider Expansion Wave 1

Status: Complete

Exit criteria:

- One high-value AV fallback provider is disabled by default and route-aware.
- Config, registry, manifest, and docs expose the provider.
- Synthetic fixtures prove search/detail mapping, external IDs, artwork, and AV facts.

## M5 - Verification And Closeout

Status: Complete

Exit criteria:

- Focused and package gates pass or failures are documented with cause.
- Workstream evidence records command output summaries.
- Follow-up provider/protocol work is explicitly split.
- Final commit(s) contain only intended changes.
