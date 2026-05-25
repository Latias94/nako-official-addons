# Official Notification Bridge Provider Extensibility - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

ONBPE-020 is complete. `nako-notification-bridge` now has a
provider-local registry seam that owns provider diagnostics, configuration
status, send-path count, multi-send fail-closed error construction,
selected-provider output, and generic attempt-history mapping.

Existing behavior is preserved for ACK-only mode, HTTP webhook, Discord
webhook, safe templates, bounded provider attempt history, aggregate
configuration status, and redaction-safe health/diagnostics.

## Active Task

- Task ID: ONBPE-030
- Owner: codex
- Files: `crates/nako-notification-bridge/src`, `addons/notification-bridge`, `docs`
- Validation: `cargo nextest run -p nako-notification-bridge test_send provider diagnostics --no-fail-fast`; `cargo fmt --package nako-notification-bridge --check`; `git diff --check`
- Status: NEEDS_CONTEXT
- Review: The test-send path must fail closed for zero providers, multiple
  providers, invalid provider config, and invalid enabled-provider templates.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Use a notification-bridge-local provider registry seam, not a generic
  cross-addon provider framework.
- Keep exactly one configured send path as the default fail-closed rule.
- Treat safe provider test-send as the next vertical slice after registry
  extraction.
- ONBPE-020 kept route behavior stable and added focused provider registry
  tests rather than changing public HTTP response shapes.

## Blockers

- None for ONBPE-030.
- Concrete new provider choice is intentionally deferred until the registry seam
  shows the extension cost.

## Next Recommended Action

- Execute ONBPE-030 by adding a redaction-safe provider test-send path through
  the new registry seam.
