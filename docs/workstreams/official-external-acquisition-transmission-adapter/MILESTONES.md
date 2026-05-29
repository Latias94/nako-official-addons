# Official External Acquisition Transmission Adapter - Milestones

Status: Active
Last updated: 2026-05-29

## M0 - Lane Open

Exit criteria:

- Workstream docs exist and agree.
- Transmission is scoped as the first production profile only.
- Non-goals preserve qBittorrent, aria2, cloud-drive, browser raw URL
  submission, Addon Manager lifecycle, and Managed Import promotion boundaries.

## M1 - Profile And Secret Policy

Exit criteria:

- [x] Transmission profile config is opt-in.
- [x] Debug output and diagnostics redact credentials and session details.
- [x] Checked-in manifest/config schema advertises shape without embedding secrets.
- [x] Fixture remains the default local profile.

## M2 - RPC Client Harness

Exit criteria:

- Fake RPC tests cover session-id retry and core methods.
- Client errors expose safe categories only.
- The transport boundary is testable without a real daemon.

## M3 - Enqueue

Exit criteria:

- Enqueue materializes once for the approved action context.
- Supported material enqueues to Transmission.
- Duplicate add behavior maps to an idempotent/AlreadyExists response where
  possible.
- `runner_job_ref` uses `transmission:<hash_string>`.
- Unsupported link types fail safely.

## M4 - Status And Controls

Exit criteria:

- Query status maps Transmission state/progress to addon response state.
- Cancel, pause, and resume operate only on runner job refs.
- No status/control path rematerializes target links.

## M5 - Integration And Smoke

Exit criteria:

- Route and health diagnostics expose safe profile readiness.
- Full package tests pass.
- Local smoke remains fixture-only by default.
- Optional live Transmission smoke instructions are documented but not required
  for CI.

## M6 - Closeout

Exit criteria:

- Evidence gates are fresh.
- Residual risks and follow-ons are explicit.
- Workstream status moves to complete/closed.
- The next goal item can proceed to Android ACFH-090.
