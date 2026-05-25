# Official Notification Bridge Provider Extensibility - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

ONBPE-020 and ONBPE-030 are complete. `nako-notification-bridge` now has a
provider-local registry seam plus sidecar-local `POST /providers/test-send`.
The test-send path sends a synthetic redaction-safe notification through the
single configured provider and returns only safe delivery status.

Existing behavior is preserved for ACK-only mode, HTTP webhook, Discord
webhook, safe templates, bounded provider attempt history, aggregate
configuration status, and redaction-safe health/diagnostics. Test-send fails
closed for no provider send path, multiple provider send paths, invalid provider
configuration, and invalid enabled-provider templates.

## Active Task

- Task ID: ONBPE-040
- Owner: codex
- Files: `crates/nako-notification-bridge/src`, `addons/notification-bridge`, `docs`
- Validation: `cargo nextest run -p nako-notification-bridge --no-fail-fast`; `cargo clippy -p nako-notification-bridge --all-targets -- -D warnings`; `cargo fmt --package nako-notification-bridge --check`; `git diff --check`
- Status: NEEDS_CONTEXT
- Review: Do not add a low-quality provider just to increase count. Keep
  secrets redaction-safe and avoid live-network default tests.
- Evidence: To be recorded in `EVIDENCE_AND_GATES.md`.

## Decisions Since Last Update

- Use a notification-bridge-local provider registry seam, not a generic
  cross-addon provider framework.
- Keep exactly one configured send path as the default fail-closed rule.
- Treat safe provider test-send as the next vertical slice after registry
  extraction.
- ONBPE-020 kept route behavior stable and added focused provider registry
  tests rather than changing public HTTP response shapes.
- ONBPE-030 kept test-send sidecar-local rather than declaring it as an Addon
  Protocol resource.

## Blockers

- None for ONBPE-040.
- Concrete new provider choice is intentionally deferred until the registry seam
  shows the extension cost.

## Next Recommended Action

- Execute ONBPE-040 by proving the registry makes a provider extension cheap,
  or split a concrete provider into a follow-on if it is no longer small.
