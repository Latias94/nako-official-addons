# Official Notification Bridge Provider Extensibility - TODO

Status: Active
Last updated: 2026-05-25

Task IDs use the `ONBPE` prefix.

## M0 - Scope And Evidence Freeze

- [x] ONBPE-010 [owner=planner] [deps=none] [scope=docs/workstreams/official-notification-bridge-provider-extensibility]
  Goal: Open the lane, freeze provider-extensibility scope, and choose the
  first vertical proof.
  Validation: DESIGN.md, TODO.md, MILESTONES.md, EVIDENCE_AND_GATES.md,
  WORKSTREAM.json, and HANDOFF.md agree.
  Review: Confirm this lane does not duplicate release-readiness or metadata
  provider workstreams.
  Evidence: this workstream directory.
  Result: DONE 2026-05-25.
  Handoff: Continue with ONBPE-020.

## M1 - Provider Registry Seam

- [x] ONBPE-020 [owner=codex] [deps=ONBPE-010] [scope=crates/nako-notification-bridge/src]
  Goal: Move notification provider identity, status, send-path readiness,
  diagnostics, selected-provider enforcement, and attempt-history mapping out of
  route-local branching into a provider-local registry/seam.
  Validation: `cargo nextest run -p nako-notification-bridge provider health diagnostics --no-fail-fast`;
  `cargo fmt --package nako-notification-bridge --check`; `git diff --check`.
  Review: Preserve current HTTP webhook and Discord webhook response shapes,
  safe diagnostics, and fail-closed multi-provider behavior. Avoid a generic
  cross-addon framework.
  Evidence: provider registry tests, route tests, and EVIDENCE_AND_GATES.md.
  Result: DONE 2026-05-25.
  Evidence: Added `provider_registry.rs` with provider diagnostics,
  configuration status, selected-provider output, and generic attempt-history
  mapping. Targeted nextest passed 18/18; package nextest passed 34/34; fmt,
  clippy, and diff checks passed.
  Handoff: Continue with ONBPE-030 safe provider test-send.

## M2 - Safe Provider Test-Send

- [x] ONBPE-030 [owner=codex] [deps=ONBPE-020] [scope=crates/nako-notification-bridge/src,addons/notification-bridge,docs]
  Goal: Add a redaction-safe provider test-send path that sends a synthetic
  notification through the single configured provider and reports safe delivery
  status without echoing secrets, webhook URLs, template text, or raw payload
  values.
  Validation: `cargo nextest run -p nako-notification-bridge test_send provider diagnostics --no-fail-fast`;
  `cargo fmt --package nako-notification-bridge --check`; `git diff --check`.
  Review: The test-send path must fail closed for zero providers, multiple
  providers, invalid provider config, and invalid enabled-provider templates.
  Evidence: route tests, smoke/docs updates, and EVIDENCE_AND_GATES.md.
  Result: DONE 2026-05-25.
  Evidence: Added `POST /providers/test-send`, success and fail-closed route
  tests, provider registry preflight checks, and operator docs. Targeted
  nextest passed 19/19; package nextest passed 39/39; fmt, clippy, and diff
  checks passed.
  Handoff: Continue with ONBPE-040 provider extension proof.

## M3 - Provider Extension Proof

- [x] ONBPE-040 [owner=codex] [deps=ONBPE-020,ONBPE-030] [scope=crates/nako-notification-bridge/src,docs]
  Goal: Prove the registry makes another provider or provider-ready extension
  cheap. Prefer a small provider with fixture-backed tests only if the
  configuration and payload contract are clear; otherwise record the extension
  point and split concrete provider implementation into a follow-on.
  Validation: `cargo nextest run -p nako-notification-bridge --no-fail-fast`;
  `cargo clippy -p nako-notification-bridge --all-targets -- -D warnings`;
  `cargo fmt --package nako-notification-bridge --check`; `git diff --check`.
  Review: Do not add a low-quality provider just to increase count. Keep
  secrets redaction-safe and avoid live-network default tests.
  Evidence: provider proof tests or follow-on decision in HANDOFF.md.
  Result: DONE 2026-05-25.
  Evidence: Added default-disabled `telegram` provider with fixture-backed
  `library.scanned`, `POST /providers/test-send`, health, diagnostics, config,
  and redaction tests. Targeted nextest passed 14/14; package nextest passed
  44/44; fmt, clippy, and diff checks passed.
  Handoff: Continue with ONBPE-050 closeout.

## M4 - Closeout

- [ ] ONBPE-050 [owner=planner] [deps=ONBPE-030,ONBPE-040] [scope=docs/workstreams/official-notification-bridge-provider-extensibility]
  Goal: Verify final evidence, record residual risks, and close the lane or
  split any remaining provider-specific work.
  Validation: verify-rust-workstream records fresh final gate evidence.
  Review: review-workstream has no blocking findings.
  Evidence: EVIDENCE_AND_GATES.md, WORKSTREAM.json, HANDOFF.md.
  Handoff: Summarize next provider recommendations and release-readiness impact.
