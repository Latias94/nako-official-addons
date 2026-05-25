# Official Notification Bridge Provider Extensibility - Handoff

Status: Closed
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

- Task ID: none
- Status: CLOSED
- Evidence: `EVIDENCE_AND_GATES.md` records final workspace nextest, package
  clippy, package fmt, diff check, and review gate.

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

- None.

## Follow-Ons

- Consider a provider-runtime catalog if the next provider repeats the same
  route-helper send wiring.
- Keep live Telegram/Discord/HTTP provider verification opt-in and
  operator-local; default CI should remain fixture-backed and secret-free.

## Closeout

The lane is closed. The shipped release-readiness surface is a default-disabled
HTTP webhook, Discord webhook, and Telegram provider set with exactly-one
provider enforcement, safe test-send, bounded safe attempt history, and
redaction-safe health/diagnostics.
