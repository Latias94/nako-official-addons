# Official Notification Bridge Provider Extensibility - Handoff

Status: Active
Last updated: 2026-05-25

## Current State

ONBPE-020, ONBPE-030, and ONBPE-040 are complete.
`nako-notification-bridge` now has a provider-local registry seam,
sidecar-local `POST /providers/test-send`, and a default-disabled Telegram
provider proof. The test-send path sends a synthetic redaction-safe
notification through the single configured provider and returns only safe
delivery status.

Existing behavior is preserved for ACK-only mode, HTTP webhook, Discord
webhook, Telegram `sendMessage`, safe templates, bounded provider attempt
history, aggregate configuration status, and redaction-safe
health/diagnostics. Test-send fails closed for no provider send path, multiple
provider send paths, invalid provider configuration, and invalid
enabled-provider templates.

## Active Task

- Task ID: ONBPE-050
- Owner: planner
- Files: `docs/workstreams/official-notification-bridge-provider-extensibility`
- Validation: verify final gate evidence and review notes before closing.
- Status: NEEDS_CONTEXT
- Review: Close the lane or split any residual provider-runtime generalization
  into a follow-on.
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
- ONBPE-040 chose Telegram as the provider extension proof because its Bot API
  `sendMessage` contract is small, useful, and fixture-testable without live
  secrets.
- Telegram is default-disabled and participates in the same exactly-one
  provider send path fail-closed rule.

## Blockers

- None for ONBPE-050.
- Remaining potential provider-runtime generalization is a follow-on decision,
  not a blocker for this lane.

## Next Recommended Action

- Execute ONBPE-050 closeout: verify final evidence, record residual risks, and
  decide whether provider-runtime catalog generalization belongs in a follow-on.
