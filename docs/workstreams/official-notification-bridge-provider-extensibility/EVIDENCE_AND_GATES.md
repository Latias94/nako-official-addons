# Official Notification Bridge Provider Extensibility - Evidence And Gates

Status: Active
Last updated: 2026-05-25

## Smallest Current Repro

```powershell
cargo nextest run -p nako-notification-bridge provider health diagnostics --no-fail-fast
```

This should stay green while provider orchestration moves behind a provider
registry seam.

## Gate Set

### Targeted Iteration Gate

```powershell
cargo nextest run -p nako-notification-bridge provider health diagnostics --no-fail-fast
cargo fmt --package nako-notification-bridge --check
git diff --check
```

### Test-Send Gate

```powershell
cargo nextest run -p nako-notification-bridge test_send provider diagnostics --no-fail-fast
```

### Package Gate

```powershell
cargo nextest run -p nako-notification-bridge --no-fail-fast
cargo clippy -p nako-notification-bridge --all-targets -- -D warnings
```

### Broader Closeout Gate

```powershell
cargo nextest run --workspace --no-fail-fast
```

Use the package gate instead if unrelated workspace release-readiness blockers
or external services make the full workspace gate inappropriate; record the
reason here.

### Review Gate

Run `review-workstream` before accepting task or lane completion. Record
blocking findings, missing gates, and residual risks here or link to the review
note.

## Redaction Gate

Any changed health, diagnostics, test-send, provider output, attempt-history,
smoke, or docs example must not expose:

- provider URLs;
- Discord webhook paths or tokens;
- Telegram bot tokens or chat ids;
- shared secrets or secret header values;
- raw template text;
- raw event payload values;
- internal fixture host/port details when they are not part of the public
  contract.

## Evidence Anchors

- `docs/workstreams/official-notification-bridge-provider-extensibility/DESIGN.md`
- `docs/workstreams/official-notification-bridge-provider-extensibility/TODO.md`
- `docs/workstreams/official-notification-bridge-provider-extensibility/MILESTONES.md`
- `crates/nako-notification-bridge/src/routes.rs`
- `crates/nako-notification-bridge/src/http_webhook.rs`
- `crates/nako-notification-bridge/src/discord_webhook.rs`
- `crates/nako-notification-bridge/src/telegram.rs`
- `crates/nako-notification-bridge/src/attempt_history.rs`
- `crates/nako-notification-bridge/README.md`
- `addons/notification-bridge/README.md`

## Evidence Log

| Date | Task | Evidence | Result |
| --- | --- | --- | --- |
| 2026-05-25 | ONBPE-010 | Opened provider-extensibility workstream and selected registry seam as first vertical proof. | Pass |
| 2026-05-25 | ONBPE-020 | `cargo nextest run -p nako-notification-bridge provider health diagnostics --no-fail-fast` | Pass: 18 passed, 16 skipped |
| 2026-05-25 | ONBPE-020 | `cargo nextest run -p nako-notification-bridge --no-fail-fast` | Pass: 34 passed |
| 2026-05-25 | ONBPE-020 | `cargo clippy -p nako-notification-bridge --all-targets -- -D warnings` | Pass |
| 2026-05-25 | ONBPE-020 | `cargo fmt --package nako-notification-bridge --check`; `git diff --check` | Pass |
| 2026-05-25 | ONBPE-020 | `cargo nextest run --workspace --no-fail-fast` | Skipped: changed Rust scope is limited to `nako-notification-bridge`; package nextest plus clippy/fmt/diff gates prove the task-local behavior and compile surface. |
| 2026-05-25 | ONBPE-030 | `cargo nextest run -p nako-notification-bridge test_send provider diagnostics --no-fail-fast` | Pass: 19 passed, 20 skipped |
| 2026-05-25 | ONBPE-030 | `cargo nextest run -p nako-notification-bridge --no-fail-fast` | Pass: 39 passed |
| 2026-05-25 | ONBPE-030 | `cargo clippy -p nako-notification-bridge --all-targets -- -D warnings` | Pass |
| 2026-05-25 | ONBPE-030 | `cargo fmt --package nako-notification-bridge --check`; `git diff --check` | Pass |
| 2026-05-25 | ONBPE-030 | Review gate | Pass: no blocking workstream-compliance or code-quality findings. Residual live-provider behavior remains covered by opt-in local smoke, not default CI. |
| 2026-05-25 | ONBPE-030 | `cargo nextest run --workspace --no-fail-fast` | Skipped: changed Rust scope is limited to `nako-notification-bridge`; package nextest plus clippy/fmt/diff gates prove the task-local behavior and compile surface. |
| 2026-05-25 | ONBPE-040 | `cargo nextest run -p nako-notification-bridge telegram diagnostics health --no-fail-fast` | Pass: 14 passed, 30 skipped |
| 2026-05-25 | ONBPE-040 | `cargo nextest run -p nako-notification-bridge --no-fail-fast` | Pass: 44 passed |
| 2026-05-25 | ONBPE-040 | `cargo clippy -p nako-notification-bridge --all-targets -- -D warnings` | Pass |
| 2026-05-25 | ONBPE-040 | `cargo fmt --package nako-notification-bridge --check`; `git diff --check` | Pass |
| 2026-05-25 | ONBPE-040 | Review gate | Pass: no blocking workstream-compliance or code-quality findings. Residual live Telegram delivery remains operator-local, fixture-backed by default. |
| 2026-05-25 | ONBPE-040 | `cargo nextest run --workspace --no-fail-fast` | Skipped: changed Rust scope is limited to `nako-notification-bridge`; package nextest plus clippy/fmt/diff gates prove the task-local behavior and compile surface. |

## Notes

Fresh verification is required before marking a task, Codex goal, or lane
complete.
