# Official Notification Bridge Provider Extensibility - Milestones

Status: Active
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem and target state are explicit.
- Non-goals are explicit.
- Related ADRs/docs/workstreams are linked.
- First proof target is chosen.

Primary evidence:

- `docs/workstreams/official-notification-bridge-provider-extensibility/DESIGN.md`
- `docs/workstreams/official-notification-bridge-provider-extensibility/TODO.md`

## M1 - Provider Registry Seam

Exit criteria:

- Provider identity, readiness, diagnostics, and send-path counting are derived
  through a notification provider seam.
- Route handlers stop duplicating provider-specific orchestration details.
- Existing HTTP webhook and Discord webhook behavior is preserved.
- Disabled ACK-only providers still do not create attempt-history records.

Primary gates:

- `cargo nextest run -p nako-notification-bridge provider health diagnostics --no-fail-fast`
- `cargo fmt --package nako-notification-bridge --check`
- `git diff --check`

## M2 - Safe Provider Test-Send

Exit criteria:

- A test-send path exists for the single configured provider.
- It reports only redaction-safe status and failure facts.
- It fails closed for zero providers, multiple providers, invalid provider
  config, and invalid enabled-provider templates.
- Operator docs explain how to use it without live CI secrets.

Primary gates:

- `cargo nextest run -p nako-notification-bridge test_send provider diagnostics --no-fail-fast`
- `cargo fmt --package nako-notification-bridge --check`
- `git diff --check`

## M3 - Provider Extension Proof

Exit criteria:

- The provider seam makes a provider-extension proof cheap, or the concrete
  provider is split into a documented follow-on with reasons.
- No low-quality provider is added just to increase provider count.
- Redaction-safety tests remain explicit.

Primary gates:

- `cargo nextest run -p nako-notification-bridge --no-fail-fast`
- `cargo clippy -p nako-notification-bridge --all-targets -- -D warnings`
- `cargo fmt --package nako-notification-bridge --check`
- `git diff --check`

## M4 - Closeout

Exit criteria:

- Gate set is recorded with fresh evidence.
- Remaining work is completed, deferred, or split into follow-ons.
- `WORKSTREAM.json` and `HANDOFF.md` reflect the actual continuation state.
